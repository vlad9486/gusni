extern crate gusni;
extern crate serde;
extern crate serde_json;
extern crate bincode;
extern crate rand;

use std::path::Path;
use std::{fs::File, io::Write, time::SystemTime, thread, sync::{Arc, mpsc}};

use gusni::{
    core::{Buffer, Report, Eye, WaveLengthFactory, WaveLengthTrimmedFactory},
    tree::Sphere,
    light::CustomMaterial,
};

use serde::{Serialize, Deserialize};
use bincode::serialize;

fn main() {
    let scene: Arc<Vec<Sphere<CustomMaterial, f64>>> = Arc::new(serde_json::from_str(include_str!("../scene.json")).unwrap());
    let eye: Arc<Eye<f64>> = Arc::new(serde_json::from_str(include_str!("../eye.json")).unwrap());

    let (sender, receiver) = mpsc::channel();
    let threads_number = 8;
    let horizontal_resolution = 256 * 16;
    let vertical_resolution = 256 * 9;
    let threads = (0..threads_number)
        .map(|i| {
            let eye = eye.clone();
            let scene = scene.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let mut buffer = Buffer::new(
                    horizontal_resolution,
                    vertical_resolution,
                    None,
                    WaveLengthTrimmedFactory,
                );
                let start = SystemTime::now();
                let report = Report {
                    id: i,
                    interval: 0x1000,
                    sender: &sender,
                };
                buffer.trace(&mut rng, &eye, scene.as_ref(), None, Some(report));
                let traced = SystemTime::now();
                let duration = traced.duration_since(start).unwrap();
                let ray_per_pixel = WaveLengthTrimmedFactory.resolution();
                let pixels = horizontal_resolution * vertical_resolution;
                let rays = (ray_per_pixel as u128) * (pixels as u128);
                let per_ray = duration.as_nanos() / rays;
                println!(
                    "thread: {:?}, tracing time: {:?}, {:?}, {:?}",
                    i, duration, rays, per_ray,
                );
                buffer
            })
        })
        .collect::<Vec<_>>();

    thread::spawn(move || {
        let ray_per_pixel = WaveLengthTrimmedFactory.resolution();
        let total_pixels = horizontal_resolution * vertical_resolution;
        let mut labels = (0..threads_number)
            .map(|_| (SystemTime::now(), 0))
            .collect::<Vec<_>>();
        for progress in receiver {
            let &(ref time, ref pixels) = &labels[progress.id];
            let new_time = SystemTime::now();
            let new_pixels = progress.index;
            let delta_time = new_time.duration_since(time.clone()).unwrap();
            let delta_pixels = new_pixels - pixels.clone();
            let speed = ((delta_pixels * ray_per_pixel) as f64) / (delta_time.as_micros() as f64);
            println!(
                "rays per second: {:08.8}, progress: {:0.8}",
                speed * 1_000_000.0,
                (new_pixels as f64) / (total_pixels as f64),
            );
            labels[progress.id] = (new_time, new_pixels);
        }
    });

    let start = SystemTime::now();
    let buffer = threads
        .into_iter()
        .fold(None, |a, handle| {
            let sample = handle.join().unwrap();
            match a {
                None => Some(sample),
                Some(s) => Some(s + sample),
            }
        })
        .unwrap();

    store_tga(&buffer, "target/demo.tga");
    let written = SystemTime::now();
    println!("total time: {:?}", written.duration_since(start).unwrap());
}

fn store_tga<P, F>(buffer: &Buffer<F>, path: P)
where
    P: AsRef<Path>,
    F: WaveLengthFactory,
{
    #[derive(Serialize, Deserialize)]
    pub struct TgaHeader {
        id_length: u8,
        color_map_type: u8,
        image_type: u8,
        color_map: [u8; 5],
        x_origin: u16,
        y_origin: u16,
        width: u16,
        height: u16,
        pixel_depth: u8,
        image_descriptor: u8,
    }

    impl TgaHeader {
        pub fn rgb(width: usize, height: usize) -> Self {
            TgaHeader {
                id_length: 0,
                color_map_type: 0,
                image_type: 2,
                color_map: [0, 0, 0, 0, 0],
                x_origin: 0,
                y_origin: 0,
                width: width as u16,
                height: height as u16,
                pixel_depth: 24,
                image_descriptor: 0,
            }
        }
    }

    let image_header = TgaHeader::rgb(buffer.width(), buffer.height());
    let mut file = File::create(path).unwrap();
    file.write(serialize(&image_header).unwrap().as_slice())
        .unwrap();
    let mut b = Vec::with_capacity(buffer.width() * buffer.height() * 3);
    b.resize(buffer.width() * buffer.height() * 3, 0);
    buffer.write(3072.0, true, b.as_mut());
    file.write(b.as_ref()).unwrap();
}

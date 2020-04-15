extern crate gusni;
extern crate serde;
extern crate serde_json;
extern crate bincode;
extern crate rand;

use std::path::Path;
use std::{fs::File, io::Write, time::SystemTime, thread, sync::Arc};

use gusni::{
    core::{Buffer, Eye, WaveLengthFactory, WaveLengthTrimmedFactory},
    tree::Sphere,
    light::CustomMaterial,
};

use serde::{Serialize, Deserialize};
use bincode::serialize;

fn main() {
    let scene: Arc<Vec<Sphere<CustomMaterial, f64>>> = Arc::new(serde_json::from_str(include_str!("scene.json")).unwrap());
    let eye: Arc<Eye<f64>> = Arc::new(serde_json::from_str(include_str!("eye.json")).unwrap());

    let threads = (0..8)
        .map(|i| {
            let eye = eye.clone();
            let scene = scene.clone();
            thread::spawn(move || {
                let horizontal_resolution = 1920;
                let vertical_resolution = 1080;
                let mut rng = rand::thread_rng();
                let mut buffer = Buffer::new(
                    horizontal_resolution,
                    vertical_resolution,
                    WaveLengthTrimmedFactory,
                );
                let start = SystemTime::now();
                let sample_count = 1;
                for _ in 0..sample_count {
                    buffer.trace(&mut rng, &eye, scene.as_ref());
                }
                let traced = SystemTime::now();
                let duration = traced.duration_since(start).unwrap();
                let ray_per_pixel = sample_count * WaveLengthTrimmedFactory.resolution();
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

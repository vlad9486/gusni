extern crate gusni;
extern crate serde;
extern crate bincode;
extern crate rand;

use std::path::Path;
use std::{fs::File, io::Write, time::SystemTime, thread, sync::Arc};

use gusni::{
    core::{Buffer, Eye, V3, WaveLengthFactory, WaveLengthTrimmedFactory},
    tree::Sphere,
    light::CustomMaterial,
};

use serde::{Serialize, Deserialize};
use bincode::serialize;

fn main() {
    use self::CustomMaterial::{
        DiffuseWhite, Glass, DiffuseGreen, DiffuseBlue, SemiMirrorRed, Light,
    };

    let scene = {
        let r = 100000.0;

        let zp = Sphere::new(V3::new(0.0, 0.0, -r + 10.0), r, DiffuseBlue);
        let zn = Sphere::new(V3::new(0.0, 0.0, -r - 20.0), r, DiffuseWhite);
        let yp = Sphere::new(V3::new(0.0, r + 10.0, 0.0), r, DiffuseWhite);
        let yn = Sphere::new(V3::new(0.0, -r - 10.0, 0.0), r, DiffuseWhite);
        let xp = Sphere::new(V3::new(r + 10.0, 0.0, 0.0), r, DiffuseGreen);
        let xn = Sphere::new(V3::new(-r - 10.0, 0.0, 0.0), r, DiffuseGreen);

        let a = Sphere::new(V3::new(-4.0, -6.0, 3.0), 4.0, Glass(false));
        let b = Sphere::new(V3::new(-4.0, -6.0, 3.0), 3.99, Glass(true));
        let c = Sphere::new(V3::new(4.0, -7.5, 6.0), 2.5, SemiMirrorRed);

        let source = Sphere::new(V3::new(0.0, 1000.0 + 9.98, 0.0), 1000.0, Light);

        Arc::new(vec![zp, zn, yp, yn, xp, xn, a, b, c, source])
    };

    let eye = Arc::new(Eye {
        position: V3::new(0.0, 0.0, -19.0),

        forward: V3::new(0.0, 0.0, 1.0),
        right: V3::new(1.0, 0.0, 0.0),
        up: V3::new(0.0, 1.0, 0.0),

        width: 1.6,
        height: 0.9,
        distance: 0.8,
    });

    let threads = (0..8)
        .map(|i| {
            let eye = eye.clone();
            let scene = scene.clone();
            thread::spawn(move || {
                let horizontal_resolution = 640;
                let vertical_resolution = 360;
                let mut rng = rand::thread_rng();
                let mut buffer =
                    Buffer::new(
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
    buffer.write(64.0, true, b.as_mut());
    file.write(b.as_ref()).unwrap();
}

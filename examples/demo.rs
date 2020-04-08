extern crate gusni;
extern crate serde;
extern crate bincode;
extern crate generic_array;
extern crate rand;

use std::path::Path;
use std::{
    fs::File,
    io::{Read, Write},
    time::SystemTime,
};

use gusni::{Sample, Eye, Size, V3, Sphere, Beam, Material, Density};

use serde::{Serialize, Deserialize};
use num::Float;
use bincode::{serialize, deserialize};
use generic_array::{ArrayLength, typenum::U12};

fn main() {
    let scene = {
        let gray = Beam::<_, U12>::red() + Beam::green() + Beam::blue();

        let rg = Material::<U12, f32>::new(
            Beam::default(),
            &Beam::red() * 0.5,
            &gray * 0.5,
            Beam::default(),
            Beam::default(),
        );
        let d = Material::<U12, f32>::new(
            Beam::default(),
            Beam::blue(),
            Beam::default(),
            Beam::default(),
            Beam::default(),
        );
        let e_w = Material::new(
            gray.clone(),
            Beam::default(),
            Beam::default(),
            Beam::default(),
            Beam::default(),
        );
        let d_gray = Material::new(
            Beam::default(),
            gray.clone(),
            Beam::default(),
            Beam::default(),
            Beam::default(),
        );

        let r = 100000.0;
        let zp = Sphere::new(V3::new(0.0, 0.0, -r + 20.0), r, d.clone());
        let zn = Sphere::new(V3::new(0.0, 0.0, -r - 10.0), r, d_gray.clone());
        let yp = Sphere::new(V3::new(0.0, r + 10.0, 0.0), r, d_gray.clone());
        let yn = Sphere::new(V3::new(0.0, -r - 10.0, 0.0), r, d_gray.clone());
        let xp = Sphere::new(V3::new(r + 10.0, 0.0, 0.0), r, d_gray.clone());
        let xn = Sphere::new(V3::new(-r - 10.0, 0.0, 0.0), r, d_gray.clone());

        let a = Sphere::new(V3::new(-0.9, 0.0, 0.0), 1.0, rg.clone());
        let b = Sphere::new(V3::new(1.5, 1.0, 0.5), 1.5, rg.clone());

        let source = Sphere::new(V3::new(0.0, 1000.0 + 9.98, -4.0), 1000.0, e_w.clone());

        vec![zp, zn, yp, yn, xp, xn, a, b, source]
    };

    let size = Size {
        horizontal_count: 1920,
        vertical_count: 1080,
    };

    let eye = Eye {
        position: V3::new(0.0, 0.0, -9.0),
        forward: V3::new(0.0, 0.0, 1.0),
        right: V3::new(1.0, 0.0, 0.0),
        up: V3::new(0.0, 1.0, 0.0),

        width: 1.6,
        height: 0.9,
        distance: 1.2,
    };

    let mut rng = rand::thread_rng();
    let mut sample = Sample::new(size);
    let start = SystemTime::now();
    for _ in 0..16 {
        sample.trace(&mut rng, &eye, &scene);
    }
    let traced = SystemTime::now();
    store_tga(&sample, "target/demo.tga");
    let written = SystemTime::now();
    println!("tracing time: {:?}", traced.duration_since(start).unwrap());
    println!("writing time: {:?}", written.duration_since(traced).unwrap());
}

pub fn load<P, N, C>(path: P) -> Option<Sample<N, C>>
where
    P: AsRef<Path>,
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
    for<'de> Sample<N, C>: Deserialize<'de>,
{
    match File::open(path) {
        Ok(mut file) => {
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            Some(deserialize(data.as_slice()).unwrap())
        },
        Err(_) => None,
    }
}

pub fn store<P, N, C>(sample: &Sample<N, C>, path: P)
where
    P: AsRef<Path>,
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
    Sample<N, C>: Serialize,
{
    let image_encoded: Vec<u8> = serialize(sample).unwrap();
    let mut file = File::create(path).unwrap();
    file.write(image_encoded.as_slice()).unwrap();
}

fn store_tga<P, N, C>(sample: &Sample<N, C>, path: P)
where
    P: AsRef<Path>,
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
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
        pub fn rgb(width: u32, height: u32) -> Self {
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

    let size = sample.size();
    let image_header = TgaHeader::rgb(size.horizontal_count, size.vertical_count);
    let mut file = File::create(path).unwrap();
    file.write(serialize(&image_header).unwrap().as_slice())
        .unwrap();
    file.write(sample.bitmap().as_slice()).unwrap();
}

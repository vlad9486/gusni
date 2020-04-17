use gusni::core::{Buffer, Progress, Report, WaveLengthTrimmedFactory, Eye, Scene};
use std::{
    path::PathBuf,
    thread,
    sync::{mpsc, Arc, Mutex},
};

struct TracerContext {
    handle: Option<thread::JoinHandle<()>>,
    terminate_sender: mpsc::Sender<()>,
}

impl TracerContext {
    pub fn start<E, R, S>(
        id: usize,
        width: usize,
        height: usize,
        eye: E,
        scene: R,
        parent: Arc<Mutex<Buffer<WaveLengthTrimmedFactory>>>,
        progress_sender: mpsc::Sender<Progress>,
    ) -> Self
    where
        E: 'static + Send + AsRef<Eye<f64>>,
        R: 'static + Send + AsRef<S>,
        S: Scene<f64>,
    {
        let (terminate_sender, terminate_receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let factory = WaveLengthTrimmedFactory;
            let mut buffer = Buffer::new(width, height, None, factory);
            println!("starting {}", id);
            loop {
                let complete = buffer.trace(
                    &mut rng,
                    eye.as_ref(),
                    scene.as_ref(),
                    Some(&terminate_receiver),
                    Some(Report {
                        id: id,
                        interval: 0x1000,
                        sender: &progress_sender,
                    })
                );
                if !complete {
                    println!("stopping {}", id);
                    break;
                } else {
                    println!("writing {}", id);
                    *parent.lock().unwrap() += &mut buffer;
                }
            }
        });
        TracerContext {
            handle: Some(handle),
            terminate_sender: terminate_sender,
        }
    }

    pub fn stop(&mut self) {
        let _ = self.handle.take().map(|handle| {
            self.terminate_sender.send(()).unwrap();
            handle.join().unwrap()
        });
    }
}

pub struct Tracer {
    contexts: Vec<TracerContext>,
    progress_receiver: thread::JoinHandle<()>,
    buffer: Arc<Mutex<Buffer<WaveLengthTrimmedFactory>>>,
}

impl Tracer {
    pub fn start(
        width: usize,
        height: usize,
        threads: usize,
        scene_file: PathBuf,
        eye_file: PathBuf,
        state_file: Option<PathBuf>,
    ) -> Self {
        use std::{
            fs,
            fs::File,
            io::Read,
            mem,
        };
        use byteorder::{LittleEndian, ByteOrder};
        use gusni::{
            tree::Sphere,
            light::CustomMaterial,
        };

        let buffer = if let Some(state_file) = state_file {
            let mut size = [0; mem::size_of::<(u64, u64)>()];
            let mut state_file = File::open(state_file).unwrap();
            state_file.read(size.as_mut()).unwrap();
            let width = LittleEndian::read_u64(&size[0x00..0x08]) as usize;
            let height = LittleEndian::read_u64(&size[0x08..0x10]) as usize;
    
            let s = mem::size_of::<f64>();
            let capacity = width * height * 3 * s;
            let mut byte_buffer = Vec::with_capacity(capacity);
            byte_buffer.resize(capacity, 0);
            state_file.read(byte_buffer.as_mut()).unwrap();

            let mut data = Vec::with_capacity(width * height * 3);
            data.resize(capacity, 0.0);

            for i in 0..(width * height * 3) {
                let f = LittleEndian::read_f64(&byte_buffer[(i * s)..((i + 1) * s)]);
                data[i] = f;
            }

            Buffer::new(width, height, Some(data), WaveLengthTrimmedFactory)
        } else {
            Buffer::new(width, height, None, WaveLengthTrimmedFactory)
        };
        let buffer = Arc::new(Mutex::new(buffer));

        let scene_json = fs::read_to_string(scene_file.as_path()).unwrap();
        let scene: Arc<Vec<Sphere<CustomMaterial, f64>>> = 
            Arc::new(serde_json::from_str(scene_json.as_str()).unwrap());
        let eye_json = fs::read_to_string(eye_file.as_path()).unwrap();
        let eye: Arc<Eye<f64>> = 
            Arc::new(serde_json::from_str(eye_json.as_str()).unwrap());

        let (progress_sender, progress_receiver) = mpsc::channel();

        let contexts = (0..threads)
            .map(|thread_id| {
                let eye = eye.clone();
                let scene = scene.clone();
                TracerContext::start(
                    thread_id,
                    width,
                    height,
                    eye,
                    scene,
                    buffer.clone(),
                    progress_sender.clone(),
                )
            })
            .collect();

        Tracer {
            contexts: contexts,
            progress_receiver: thread::spawn(move || {
                progress_receiver.into_iter().for_each(|progress| {
                    println!("{:?}", progress);
                })
            }),
            buffer: buffer,
        }
    }

    pub fn image(&self, scale: f64, tga_file: PathBuf) {
        use std::{
            fs::File,
            io::Write,
        };
        use byteorder::{LittleEndian, ByteOrder};

        let buffer = self.buffer.lock().unwrap();

        let mut tga_header = [0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 0];
        LittleEndian::write_u16(&mut tga_header[12..14], buffer.width() as u16);
        LittleEndian::write_u16(&mut tga_header[14..16], buffer.height() as u16);

        let mut tga_buffer = Vec::new();
        tga_buffer.resize(3 * buffer.width() * buffer.height(), 0);
        buffer.write(scale, true, tga_buffer.as_mut());

        let mut tga_file = File::create(tga_file).unwrap();
        tga_file.write_all(tga_header.as_ref()).unwrap();
        tga_file.write_all(tga_buffer.as_ref()).unwrap();
    }

    pub fn stop(self, state_file: Option<PathBuf>) {
        use std::{
            fs::File,
            io::Write,
            mem,
        };
        use byteorder::{LittleEndian, ByteOrder};

        self
            .contexts
            .into_iter()
            .for_each(|mut context| {
                context.stop();
            });
        self.progress_receiver.join().unwrap();
        if let Some(state_file) = state_file {
            let buffer = self.buffer.lock().unwrap();

            let mut size = [0; mem::size_of::<(u64, u64)>()];
            LittleEndian::write_u64(&mut size[0x00..0x08], buffer.width() as u64);
            LittleEndian::write_u64(&mut size[0x08..0x10], buffer.height() as u64);
            let mut state_file = File::create(state_file).unwrap();
            state_file.write_all(size.as_ref()).unwrap();

            let s = mem::size_of::<f64>();
            let capacity = s * buffer.data().len();
            let mut byte_buffer = Vec::with_capacity(capacity);
            byte_buffer.resize(capacity, 0);
            for i in 0..buffer.data().len() {
                let f = buffer.data()[i];
                LittleEndian::write_f64(&mut byte_buffer[(i * s)..((i + 1) * s)], f);
            }
            state_file.write_all(byte_buffer.as_ref()).unwrap();
        }
    }
}

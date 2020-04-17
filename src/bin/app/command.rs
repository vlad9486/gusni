use std::{
    fmt,
    ffi::OsString,
    num::{ParseIntError, ParseFloatError},
    path::PathBuf,
};

pub enum Command {
    Start {
        width: usize,
        height: usize,
        threads: usize,
        scene_file: PathBuf,
        eye_file: PathBuf,
        state_file: Option<PathBuf>,
    },
    Image {
        scale: f64,
        tga_file: PathBuf,
    },
    Stop {
        state_file: Option<PathBuf>,
    },
}

pub enum Exception {
    Exit,
    Error(Error),
}

pub enum Error {
    Empty,
    Unrecognized(String),
    TraceWrongWidth(Option<ParseIntError>),
    TraceWrongHeight(Option<ParseIntError>),
    TraceWrongThreads(Option<ParseIntError>),
    TraceWrongSceneFile,
    TraceWrongEyeFile,
    ImageWrongScale(Option<ParseFloatError>),
    ImageWrongTgaFile,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Error::Empty => write!(f, "command is empty"),
            &Error::Unrecognized(ref s) => write!(f, "command \'{}\' is unrecognized", s),
            _ => unimplemented!(),
        }
    }
}

impl Command {
    pub fn recognize(s: &str) -> Result<Self, Exception> {
        let mut s = s.split(" ");
        let d = s.next().ok_or(Exception::Error(Error::Empty))?;
        match d {
            "exit" => Err(Exception::Exit),
            "start" => {
                let width = s.next()
                    .ok_or(Exception::Error(Error::TraceWrongWidth(None)))?
                    .parse()
                    .map_err(|e| Exception::Error(Error::TraceWrongWidth(Some(e))))?;
                let height = s.next()
                    .ok_or(Exception::Error(Error::TraceWrongHeight(None)))?
                    .parse()
                    .map_err(|e| Exception::Error(Error::TraceWrongHeight(Some(e))))?;
                let threads = s.next()
                    .ok_or(Exception::Error(Error::TraceWrongThreads(None)))?
                    .parse()
                    .map_err(|e| Exception::Error(Error::TraceWrongThreads(Some(e))))?;
                let scene_file = s.next()
                    .ok_or(Exception::Error(Error::TraceWrongSceneFile))?;
                let eye_file = s.next()
                    .ok_or(Exception::Error(Error::TraceWrongEyeFile))?;
                let state_file = s.next();
                Ok(Command::Start {
                    width: width,
                    height: height,
                    threads: threads,
                    scene_file: PathBuf::from(OsString::from(scene_file)),
                    eye_file: PathBuf::from(OsString::from(eye_file)),
                    state_file: state_file.map(|s| PathBuf::from(OsString::from(s))),
                })
            },
            "image" => {
                let scale = s.next()
                    .ok_or(Exception::Error(Error::ImageWrongScale(None)))?
                    .parse()
                    .map_err(|e| Exception::Error(Error::ImageWrongScale(Some(e))))?;
                let file = s.next()
                    .ok_or(Exception::Error(Error::ImageWrongTgaFile))?;
                Ok(Command::Image {
                    scale: scale,
                    tga_file: PathBuf::from(OsString::from(file)),
                })
            },
            "stop" => {
                let file = s.next();
                Ok(Command::Stop {
                    state_file: file.map(|s| PathBuf::from(OsString::from(s))),
                })
            },
            s => Err(Exception::Error(Error::Unrecognized(s.to_owned()))),
        }
    }
}

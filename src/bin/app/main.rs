#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod tracer;
mod command;

fn main() {
    use self::{
        tracer::Tracer,
        command::{Command, Exception},
    };
    use std::io;

    let mut context = None;
    let mut s = String::new();
    loop {
        s.clear();
        io::stdin().read_line(&mut s).unwrap();
        match Command::recognize(s.as_str().trim_end_matches("\n")) {
            Err(Exception::Exit) => break,
            Err(Exception::Error(e)) => eprintln!("command parsing error: {}", e),
            Ok(Command::Start {
                width: width,
                height: height,
                threads: threads,
                scene_file: scene_file,
                eye_file: eye_file,
                state_file: state_file,
            }) => context = Some(Tracer::start(width, height, threads, scene_file, eye_file, state_file)),
            Ok(Command::Image {
                scale: scale,
                tga_file: tga_file,
            }) => {
                if let Some(context) = context.as_ref() {
                    context.image(scale, tga_file)
                }
            },
            Ok(Command::Stop {
                state_file: state_file,
            }) => if let Some(context) = context.take() {
                Tracer::stop(context, state_file)
            },
        };
    }
}

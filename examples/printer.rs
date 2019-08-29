use std::{process::exit, error::Error};

fn main() {
    if let Some(path) = std::env::args().nth(1) {
        match oas3::from_path(path) {
            Ok(spec) => {
                println!("{}", oas3::to_yaml(&spec).unwrap());
            }
            Err(err) => {
                eprintln!("error: {}", &err);

                let mut cause = err.source();
                while let Some(err) = cause {
                    eprintln!("caused by: {}", err.to_string());
                    cause = err.source();
                }

                exit(1);
            }
        }
    }
}

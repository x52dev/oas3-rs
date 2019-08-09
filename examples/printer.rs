use std::{io::Write, process::exit};

use failure::Fail;

fn main() {
    if let Some(path) = std::env::args().nth(1) {
        match oas3::from_path(path) {
            Ok(spec) => {
                /*for (path, op) in spec.paths {
                    println!("{}", path);
                    println!("{:#?}", op);
                }
                for (name, definition) in spec.definitions {
                    println!("{}", name);
                    println!("{:#?}", definition);
                }*/
                println!("{}", oas3::to_json(&spec).unwrap());
            }
            Err(e) => {
                let stderr = &mut ::std::io::stderr();
                let errmsg = "Error writing to stderr";

                writeln!(stderr, "error: {}", e).expect(errmsg);
                for cause in Fail::iter_chain(&e) {
                    writeln!(
                        stderr,
                        "caused by: {} {}",
                        cause.name().unwrap_or("Error"),
                        cause
                    )
                    .expect(errmsg);
                }

                // The backtrace is not always generated. Try to run this example
                // with `RUST_BACKTRACE=1`.
                if let Some(backtrace) = e.cause().and_then(|cause| cause.backtrace()) {
                    writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
                }

                exit(1);
            }
        }
    }
}

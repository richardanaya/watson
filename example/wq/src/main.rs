use colored::*;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use watson::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut buffer = Vec::new();
    if args.len() < 2 {
        for i in io::stdin().lock().bytes() {
            buffer.push(i?);
        }
    } else {
        if args.len() < 2 {
            println!("first arg should be a file: wq <test.wasm>");
            return Ok(());
        }
        let mut f = File::open(&args[1])?;
        f.read_to_end(&mut buffer)?;
    }

    match watson::parse(&buffer) {
        Ok(p) => {
            let json_string = match serde_json::to_string(&p) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Error: failed to serialize");
                    process::exit(1);
                }
            };
            if args.len() == 3 {
                let mut f = File::create(&args[2])?;
                f.write_all(&json_string.as_bytes())?;
            }  else {
                println!("{}",json_string)
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e.red());
            process::exit(1);
        }
    };
    Ok(())
}

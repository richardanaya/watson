use colored::*;
use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut redirect = false;
    if args.len() < 2 {
        redirect = true;
    }
    if args.len() == 2 {
        let mut buffer = Vec::new();
        if redirect {
            for i in io::stdin().lock().bytes() {
                buffer.push(i?);
            }
        } else {
            buffer = fs::read(&args[1])?;
        }
        match watson::parse(&buffer) {
            Ok(p) => {
                let json_string = match serde_json::to_string(&p.to_owned()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("Error: failed to serialize");
                        process::exit(1);
                    }
                };
                println!("{}", json_string)
            }
            Err(e) => {
                eprintln!("Error: {}", e.red());
                process::exit(1);
            }
        };
    } else if args.len() == 3 {
        let json = fs::read_to_string(&args[1])?;
        let p: watson::Program = match serde_json::from_str(&json) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Error: failed to deserialize");
                process::exit(1);
            }
        };
        fs::write(&args[2], p.to_bytes())?;
    } else {
        println!("wq <help> for help");
    }
    Ok(())
}

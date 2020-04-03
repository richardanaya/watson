use colored::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use watson::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("first arg should be a file: wq <test.wasm>");
        return Ok(());
    }
    let mut f = File::open(&args[1])?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    match Program::parse(&buffer) {
        Ok(p) => println!("{}", p.to_json().unwrap()),
        Err(e) => {
            eprintln!("Error: {}", e.red());
            process::exit(1);
        }
    };
    Ok(())
}

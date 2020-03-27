use std::env;
use std::fs::File;
use std::io::prelude::*;
use watson::*;

fn section_id_string(_s: &Section) -> String {
    "unknown".to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("first arg should be a file");
        return Ok(());
    }
    let mut f = File::open(&args[1])?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let program = Program::load(&buffer)?;
    println!("Program: {}", &args[1]);
    for s in program.sections.iter() {
        println!("  - Section[{}]", section_id_string(&s));
    }
    Ok(())
}

use std::env;
use std::fs::File;
use std::io::prelude::*;
use watson::*;

fn print_type_section(s:&TypeSection) {
    println!("- Types");
    println!("  {:?}",s.data);
}

fn print_function_section(s:&FunctionSection) {
    println!("- Functions");
    for i in 0..s.function_types.len() {
        println!("  - function {} type: {:?}",i,s.function_types[i]);
    }
}

fn print_export_section(s:&ExportSection) {
    println!("- Exports");
    println!("  {:?}",s.data);
}

fn print_code_section(s:&CodeSection) {
    println!("- Code");
    for i in 0..s.function_bodies.len() {
        println!("  - function {} body: {:?}",i,s.function_bodies[i]);
    }
}

fn print_unknown_section(s:&UnknownSection) {
    println!("- UnknownSection[{}]",s.id);
    println!("  {:?}",s.data);
}

fn print_section(s: &Section) {
    match s {
        Section::Type(s) => print_type_section(&s),
        Section::Function(s) => print_function_section(&s),
        Section::Export(s) => print_export_section(&s),
        Section::Code(s) => print_code_section(&s),
        Section::Unknown(s) => print_unknown_section(&s),
    }
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
        print_section(&s);
    }
    Ok(())
}

use std::env;
use std::fs::File;
use std::io::prelude::*;
use watson::*;

fn print_type_section(s: &TypeSection) {
    println!("- Types");
    for i in 0..s.types.len() {
        match &s.types[i] {
            WasmType::Function(f) => {
                println!("  - Type 0: function(inputs{:?}) -> outputs{:?}", f.inputs, f.outputs);
            }
        }
    }
}

fn print_function_section(s: &FunctionSection) {
    println!("- Functions");
    for i in 0..s.function_types.len() {
        println!("  - Function {}: Type {:?}", i, s.function_types[i]);
    }
}

fn print_export_section(s: &ExportSection) {
    println!("- Exports");
    for i in 0..s.exports.len() {
        match &s.exports[i] {
            WasmExport::Function(f) => {
                println!("  - {:?} Function {}", f.name, f.index);
            }
            WasmExport::Memory(f) => {
                println!("  - {:?} Memory {}", f.name, f.index);
            }
        }
    }
}

fn print_memory_section(s: &MemorySection) {
    println!("- Memories");
    for i in 0..s.memories.len() {
        println!("  - Memory {}: min {} max {}",i, s.memories[i].min_pages, s.memories[i].max_pages);
    }
}

fn print_code_section(s: &CodeSection) {
    println!("- Code");
    for i in 0..s.code_blocks.len() {
        println!("  - Function {}: locals{:?} code{:?}", i, s.code_blocks[i].locals, s.code_blocks[i].code);
    }
}

fn print_unknown_section(s: &UnknownSection) {
    println!("- UnknownSection[{}]", s.id);
    println!("  {:?}", s.data);
}

fn print_section(s: &Section) {
    match s {
        Section::Type(s) => print_type_section(&s),
        Section::Function(s) => print_function_section(&s),
        Section::Export(s) => print_export_section(&s),
        Section::Code(s) => print_code_section(&s),
        Section::Memory(s) => print_memory_section(&s),
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

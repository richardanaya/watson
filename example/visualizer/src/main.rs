use colored::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use watson::*;

fn print_type_section(s: &TypeSection) {
    println!("  [{}]", "Type".purple());
    for i in 0..s.types.len() {
        match &s.types[i] {
            WasmType::Function(f) => {
                println!("  0: fn(inputs{:?}) -> outputs{:?}", f.inputs, f.outputs);
            }
        }
    }
}

fn print_function_section(s: &FunctionSection) {
    println!("  [{}]", "Function".purple());
    for i in 0..s.function_types.len() {
        println!("  {}: type[{:?}]", i, s.function_types[i]);
    }
}

fn print_export_section(s: &ExportSection) {
    println!("  [{}]", "Export".purple());
    for i in 0..s.exports.len() {
        match &s.exports[i] {
            WasmExport::Function(f) => {
                println!("  {:?} function[{}]", f.name, f.index);
            }
            WasmExport::Memory(f) => {
                println!("  {:?} memory[{}]", f.name, f.index);
            }
            WasmExport::Global(f) => {
                println!("  {:?} global[{}]", f.name, f.index);
            }
        }
    }
}

fn print_memory_section(s: &MemorySection) {
    println!("  [{}]", "Memory".purple());
    for i in 0..s.memories.len() {
        println!(
            "  {}: min {} max {}",
            i,
            s.memories[i].min_pages,
            &match s.memories[i].max_pages {
                Some(m) => m.to_string(),
                None => "".to_string(),
            }
        );
    }
}

fn print_code_section(s: &CodeSection) {
    println!("  [{}]", "Code".purple());
    for i in 0..s.code_blocks.len() {
        println!(
            "  {}: locals{:?} code{:?}",
            i, s.code_blocks[i].locals, s.code_blocks[i].code
        );
    }
}

fn print_unknown_section(s: &UnknownSection) {
    println!("  [{}:{}]", "Unknown".purple(), s.id);
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
    println!("{} {{", &args[1].green());
    for s in program.sections.iter() {
        print_section(&s);
    }
    println!("}}");
    Ok(())
}

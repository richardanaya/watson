use colored::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use watson::*;

fn print_type_section(s: &TypeSection) {
    println!("[{}]", "Type Section".purple());
    for i in 0..s.types.len() {
        match &s.types[i] {
            WasmType::Function(f) => {
                println!("{}: fn(inputs{:?}) -> outputs{:?}", i, f.inputs, f.outputs);
            }
        }
    }
}

fn print_function_section(s: &FunctionSection) {
    println!("[{}]", "Function Section".purple());
    for i in 0..s.function_types.len() {
        println!("{}: type[{:?}]", i, s.function_types[i]);
    }
}

fn print_export_section(s: &ExportSectionView) {
    println!("[{}]", "Export Section".purple());
    for i in 0..s.exports.len() {
        match &s.exports[i] {
            WasmExportView::Function(f) => {
                println!("{:?} function[{}]", f.name, f.index);
            }
            WasmExportView::Memory(f) => {
                println!("{:?} memory[{}]", f.name, f.index);
            }
            WasmExportView::Global(f) => {
                println!("{:?} global[{}]", f.name, f.index);
            }

            WasmExportView::Table(f) => {
                println!("{:?} table[{}]", f.name, f.index);
            }
        }
    }
}

fn print_import_section(s: &ImportSectionView) {
    println!("[{}]", "Import Section".purple());
    for i in 0..s.imports.len() {
        match &s.imports[i] {
            WasmImportView::Function(f) => {
                println!("{:?}.{:?} fn type[{}]", f.module_name, f.name, f.type_index);
            }
            WasmImportView::Memory(f) => {
                if f.max_pages.is_some() {
                    println!(
                        "{:?}.{:?} memory min {} max {}",
                        f.module_name,
                        f.name,
                        f.min_pages,
                        f.max_pages.unwrap()
                    );
                } else {
                    println!(
                        "{:?}.{:?} memory min {}",
                        f.module_name, f.name, f.min_pages
                    );
                }
            }
            WasmImportView::Table(f) => {
                if f.max.is_some() {
                    println!(
                        "{:?}.{:?} table \"ANYFUNC\" min {} max {}",
                        f.module_name,
                        f.name,
                        f.min,
                        f.max.unwrap()
                    );
                } else {
                    println!(
                        "{:?}.{:?} table \"ANYFUNC\" min {}",
                        f.module_name, f.name, f.min
                    );
                }
            }
            WasmImportView::Global(f) => {
                if f.is_mutable {
                    println!(
                        "{:?}.{:?} global mut {:?}",
                        f.module_name, f.name, f.value_type
                    );
                } else {
                    println!(
                        "{:?}.{:?} iglobal mm {:?}",
                        f.module_name, f.name, f.value_type
                    );
                }
            }
        }
    }
}

fn print_table_section(s: &TableSection) {
    println!("[{}]", "Table Section".purple());
    for (i, t) in s.tables.iter().enumerate() {
        if t.max.is_some() {
            println!("{:?}: ANYREF min {:?} max {:?}", i, t.min, t.max.unwrap());
        } else {
            println!("{:?}: ANYREF min {:?}", i, t.min);
        }
    }
}

fn print_global_section(s: &GlobalSection) {
    println!("[{}]", "Global Section".purple());
    for i in 0..s.globals.len() {
        let g = &s.globals[i];
        if g.is_mutable {
            println!(
                "{}: mut {:?} expr: {:?}",
                i, g.value_type, g.value_expression
            );
        } else {
            println!(
                "{}: imm {:?} expr: {:?}",
                i, g.value_type, g.value_expression
            );
        }
    }
}

fn print_memory_section(s: &MemorySection) {
    println!("[{}]", "Memory Section".purple());
    for i in 0..s.memories.len() {
        println!(
            "{}: min {} max {}",
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
    println!("[{}]", "Code Section".purple());
    for i in 0..s.code_blocks.len() {
        println!(
            "{}: locals{:?} code{:?}",
            i,
            s.code_blocks[i]
                .locals
                .iter()
                .map(|x| (x.0, x.1))
                .collect::<Vec<(u32, ValueType)>>(),
            s.code_blocks[i].code_expression
        );
    }
}

fn print_data_section(s: &DataSectionView) {
    println!("[{}]", "Data Section".purple());
    for (i, d) in s.data_blocks.iter().enumerate() {
        println!(
            "{}: memory[{:?}] offset_expression{:?} data{:?}",
            i, d.memory, d.offset_expression, d.data,
        );
    }
}

fn print_element_section(s: &ElementSection) {
    println!("[{}]", "Element Section".purple());
    for (i, d) in s.elements.iter().enumerate() {
        println!(
            "{}: table[{:?}] expression{:?} functions{:?}",
            i, d.table, d.value_expression, d.functions,
        );
    }
}

fn print_custom_section(s: &CustomSectionView) {
    println!("[{}]", "Custom Section".purple());
    println!("{}  data{:?}", s.name, s.data,);
}

fn print_start_section(s: &StartSection) {
    println!("[{}]", "Start Section".purple());
    println!("{:?}", s.start_function);
}

fn print_section(s: &SectionView) {
    match s {
        SectionView::Type(s) => print_type_section(&s),
        SectionView::Function(s) => print_function_section(&s),
        SectionView::Export(s) => print_export_section(&s),
        SectionView::Code(s) => print_code_section(&s),
        SectionView::Memory(s) => print_memory_section(&s),
        SectionView::Start(s) => print_start_section(&s),
        SectionView::Import(s) => print_import_section(&s),
        SectionView::Table(s) => print_table_section(&s),
        SectionView::Global(s) => print_global_section(&s),
        SectionView::Data(s) => print_data_section(&s),
        SectionView::Custom(s) => print_custom_section(&s),
        SectionView::Element(s) => print_element_section(&s),
    }
}

fn print_program(program: &ProgramView) {
    for s in program.sections.iter() {
        print_section(&s);
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

    match watson::parse_web_assembly(&buffer) {
        Ok(p) => print_program(&p),
        Err(e) => {
            println!("{}", e.red());
        }
    };
    Ok(())
}

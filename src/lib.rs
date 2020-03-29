#![no_std]
#[macro_use]
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use webassembly::*;

fn tag(tag: &[u8]) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), String> + '_ {
    move |input: &[u8]| {
        if tag.len() > input.len() {
            return Err("trying to tag too many bytes".to_string());
        }
        for i in 0..tag.len() {
            if tag[i] != input[i] {
                return Err("did not match tag".to_string());
            }
        }
        Ok((&input[tag.len()..], &input[..tag.len()]))
    }
}

fn take(num: usize) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), String> {
    move |input: &[u8]| {
        if num > input.len() {
            return Err("trying to take too many bytes".to_string());
        }
        Ok((&input[num..], &input[..num]))
    }
}

fn many0<'a, T>(
    f: impl Fn(&'a [u8]) -> Result<(&'a [u8], T), String>,
) -> impl Fn(&'a [u8]) -> Result<(&'a [u8], Vec<T>), String> {
    move |input: &[u8]| {
        let mut v = vec![];
        let mut ip = input;
        loop {
            match f(ip) {
                Ok((input, item)) => {
                    v.push(item);
                    ip = input;
                }
                Err(e) => {
                    if ip.len() == 0 {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok((ip, v))
    }
}

pub struct FunctionType {
    pub inputs: Vec<u8>,
    pub outputs: Vec<u8>,
}

pub enum WasmType {
    Function(FunctionType),
}

pub struct TypeSection {
    pub types: Vec<WasmType>,
}

pub struct FunctionSection {
    pub function_types: Vec<u32>,
}

pub struct CodeBlock {
    pub locals: Vec<u8>,
    pub code: Vec<u8>,
}

pub struct CodeSection {
    pub code_blocks: Vec<CodeBlock>,
}

pub struct FunctionExport {
    pub name: String,
    pub index: usize,
}

pub struct MemoryExport {
    pub name: String,
    pub index: usize,
}

pub struct GlobalExport {
    pub name: String,
    pub index: usize,
}

pub enum WasmExport {
    Function(FunctionExport),
    Memory(MemoryExport),
    Global(GlobalExport),
}

pub struct ExportSection {
    pub exports: Vec<WasmExport>,
}

pub struct UnknownSection {
    pub id: u8,
    pub data: Vec<u8>,
}

pub struct WasmMemory {
    pub min_pages: u32,
    pub max_pages: Option<u32>,
}

pub struct MemorySection {
    pub memories: Vec<WasmMemory>,
}

pub enum Section {
    Type(TypeSection),
    Function(FunctionSection),
    Code(CodeSection),
    Export(ExportSection),
    Memory(MemorySection),
    Unknown(UnknownSection),
}

pub struct Program {
    pub sections: Vec<Section>,
}

fn wasm_u32(input: &[u8]) -> Result<(&[u8], u32), String> {
    let (i, byte_count) = match input.try_extract_u32(0) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i))
}

fn section(input: &[u8]) -> Result<(&[u8], Section), String> {
    let (input, id) = take(1)(input)?;
    let (input, section_length) = wasm_u32(input)?;

    match id[0] {
        SECTION_TYPE => {
            let (input, num_types) = wasm_u32(input)?;
            let mut types = vec![];
            let mut ip = input;
            for _ in 0..num_types {
                let (input, wasm_type) = take(1)(input)?;
                types.push(match wasm_type[0] {
                    FUNC => {
                        let (input, num_inputs) = wasm_u32(input)?;
                        let (input, inputs) = take(num_inputs as usize)(input)?;
                        let (input, num_outputs) = wasm_u32(input)?;
                        let (input, outputs) = take(num_outputs as usize)(input)?;
                        ip = input;
                        WasmType::Function(FunctionType {
                            inputs: inputs.to_vec(),
                            outputs: outputs.to_vec(),
                        })
                    }
                    _ => return Err("unknown type".to_string()),
                });
            }
            Ok((ip, Section::Type(TypeSection { types })))
        }
        SECTION_FUNCTION => {
            let (input, num_funcs) = wasm_u32(input)?;
            let mut function_types = vec![];
            let mut ip = input;
            for _ in 0..num_funcs {
                let (input, index) = wasm_u32(ip)?;
                ip = input;
                function_types.push(index);
            }
            Ok((ip, Section::Function(FunctionSection { function_types })))
        }
        SECTION_EXPORT => {
            let (input, num_exports) = wasm_u32(input)?;
            let mut exports = vec![];
            let mut ip = input;
            for _ in 0..num_exports {
                let (input, num_chars) = wasm_u32(ip)?;
                let (input, chars) = take(num_chars as usize)(input)?;
                let name = match alloc::str::from_utf8(chars) {
                    Ok(b) => b.to_string(),
                    Err(_) => return Err("could not parse export name as utf8".to_string()),
                };
                let (input, export_type) = take(1)(input)?;
                let (input, export_index) = wasm_u32(input)?;
                ip = input;
                exports.push(match export_type[0] {
                    DESC_FUNCTION => WasmExport::Function(FunctionExport {
                        name,
                        index: export_index as usize,
                    }),
                    DESC_MEMORY => WasmExport::Memory(MemoryExport {
                        name,
                        index: export_index as usize,
                    }),
                    DESC_GLOBAL => WasmExport::Global(GlobalExport {
                        name,
                        index: export_index as usize,
                    }),
                    _ => return Err("unknown export".to_string()),
                });
            }
            Ok((ip, Section::Export(ExportSection { exports: exports })))
        }
        SECTION_CODE => {
            let (input, num_funcs) = wasm_u32(input)?;
            let mut code_blocks = vec![];
            let mut ip = input;
            for _ in 0..num_funcs {
                let (input, num_op_codes) = wasm_u32(input)?;
                let (num_locals, byte_count) = match input.try_extract_u32(0) {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_string()),
                };
                let (input, _) = take(byte_count as usize)(input)?;
                let (input, locals) = take(num_locals as usize)(input)?;
                let (input, op_codes) =
                    take(num_op_codes as usize - 1 - byte_count as usize)(input)?;
                let (input, _) = tag(&[END])(input)?;
                ip = input;
                code_blocks.push(CodeBlock {
                    locals: locals.to_vec(),
                    code: op_codes.to_vec(),
                });
            }
            Ok((ip, Section::Code(CodeSection { code_blocks })))
        }
        SECTION_MEMORY => {
            let (input, num_mems) = wasm_u32(input)?;
            let mut memories = vec![];
            let mut ip = input;
            for _ in 0..num_mems {
                let (input, mem_type) = take(1)(input)?;
                match mem_type[0] {
                    LIMIT_MIN_MAX => {
                        let (input, min_pages) = wasm_u32(input)?;
                        let (input, max_pages) = wasm_u32(input)?;
                        ip = input;
                        memories.push(WasmMemory {
                            min_pages,
                            max_pages: Some(max_pages),
                        });
                    }
                    LIMIT_MIN => {
                        let (input, min_pages) = wasm_u32(input)?;
                        ip = input;
                        memories.push(WasmMemory {
                            min_pages,
                            max_pages: None,
                        });
                    }
                    _ => {
                        return Err("unhandled memory type".to_string());
                    }
                }
            }
            Ok((ip, Section::Memory(MemorySection { memories })))
        }
        _ => {
            let (input, data) = take(section_length as usize)(input)?;
            Ok((
                input,
                Section::Unknown(UnknownSection {
                    id: id[0],
                    data: data.to_vec(),
                }),
            ))
        }
    }
}

fn wasm_module(input: &[u8]) -> Result<Program, String> {
    let (input, _) = tag(MAGIC_NUMBER)(input)?;
    let (input, _) = tag(VERSION_1)(input)?;
    let (_, sections) = many0(section)(input)?;
    Ok(Program { sections })
}

impl Program {
    pub fn load(input: &[u8]) -> Result<Program, String> {
        wasm_module(input)
    }

    pub fn find_exported_function<'a>(&'a self, name: &str) -> Result<&'a FunctionExport, String> {
        let result = self.sections.iter().find(|x| matches!(Section::Export, x));
        if let Some(Section::Export(export_section)) = result {
            let result = self.sections.iter().find(|x| matches!(Section::Code, x));
            if let Some(Section::Code(_)) = result {
                let result = export_section.exports.iter().find(|x| {
                    if let WasmExport::Function(f) = x {
                        f.name == name
                    } else {
                        false
                    }
                });
                let main_export = match result {
                    Some(WasmExport::Function(f)) => f,
                    _ => {
                        let mut e = "could not find ".to_string();
                        e.push_str(name);
                        return Err(e);
                    }
                };
                Ok(main_export)
            } else {
                Err("could find code section".to_string())
            }
        } else {
            Err("could find export section".to_string())
        }
    }

    pub fn find_code_block<'a>(&'a self, index: usize) -> Result<&'a CodeBlock, String> {
        let result = self.sections.iter().find(|x| matches!(Section::Export, x));
        let result = self.sections.iter().find(|x| matches!(Section::Code, x));
        if let Some(Section::Code(code_section)) = result {
            if index >= code_section.code_blocks.len() {
                Err("invalid code block index".to_string())
            } else {
                Ok(&code_section.code_blocks[index])
            }
        } else {
            Err("could find code section".to_string())
        }
    }
}

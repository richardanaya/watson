#![no_std]
#[macro_use]
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use webassembly::*;

pub trait WasmValueTypes {
    fn try_to_value_types(self) -> Result<Vec<ValueType>, String>;
}

pub trait WasmValueType {
    fn try_to_value_type(self) -> Result<ValueType, String>;
}

#[derive(Clone)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl alloc::string::ToString for ValueType {
    fn to_string(&self) -> String {
        match self {
            ValueType::I32 => "I32".to_string(),
            ValueType::I64 => "I64".to_string(),
            ValueType::F32 => "F32".to_string(),
            ValueType::F64 => "F64".to_string(),
        }
    }
}

impl WasmValueTypes for Vec<u8> {
    fn try_to_value_types(self) -> Result<Vec<ValueType>, String> {
        let mut r = vec![];
        for v in self {
            r.push(v.try_to_value_type()?);
        }
        Ok(r)
    }
}

impl WasmValueType for u8 {
    fn try_to_value_type(self) -> Result<ValueType, String> {
        match self {
            I32 => Ok(ValueType::I32),
            I64 => Ok(ValueType::I64),
            F32 => Ok(ValueType::F32),
            F64 => Ok(ValueType::F64),
            _ => Err("could not convert data type".to_string()),
        }
    }
}

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

fn many_n<'a, T>(
    n: usize,
    f: impl Fn(&'a [u8]) -> Result<(&'a [u8], T), String>,
) -> impl Fn(&'a [u8]) -> Result<(&'a [u8], Vec<T>), String> {
    move |input: &[u8]| {
        let mut v = vec![];
        let mut ip = input;
        loop {
            if n == v.len() {
                break;
            }
            match f(ip) {
                Ok((input, item)) => {
                    v.push(item);
                    ip = input;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok((ip, v))
    }
}

fn many<'a, T>(
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
    pub inputs: Vec<ValueType>,
    pub outputs: Vec<ValueType>,
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
    pub locals: Vec<(u32, ValueType)>,
    pub code: Vec<u8>,
}

pub struct CodeSection {
    pub code_blocks: Vec<CodeBlock>,
}

pub struct Export {
    pub name: String,
    pub index: usize,
}

pub enum WasmExport {
    Function(Export),
    Table(Export),
    Memory(Export),
    Global(Export),
}

pub struct ExportSection {
    pub exports: Vec<WasmExport>,
}

pub struct FunctionImport {
    pub module_name: String,
    pub name: String,
    pub type_index: usize,
}

pub struct GlobalImport {
    pub module_name: String,
    pub name: String,
    pub value_type: ValueType,
    pub is_mutable: bool,
}

pub struct MemoryImport {
    pub module_name: String,
    pub name: String,
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

pub struct TableImport {
    pub module_name: String,
    pub name: String,
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

pub enum WasmImport {
    Function(FunctionImport),
    Global(GlobalImport),
    Memory(MemoryImport),
    Table(TableImport),
}

pub struct ImportSection {
    pub imports: Vec<WasmImport>,
}

pub struct UnknownSection {
    pub id: u8,
    pub data: Vec<u8>,
}

pub struct WasmMemory {
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

pub struct MemorySection {
    pub memories: Vec<WasmMemory>,
}

pub struct StartSection {
    pub start_function: usize,
}

pub struct Global {
    pub value_type: ValueType,
    pub is_mutable: bool,
    pub expression: Vec<u8>,
}

pub struct GlobalSection {
    pub globals: Vec<Global>,
}

pub struct Table {
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

pub struct TableSection {
    pub tables: Vec<Table>,
}

pub struct DataBlock {
    pub memory: usize,
    pub offset_expression: Vec<u8>,
    pub data: Vec<u8>,
}

pub struct DataSection {
    pub data_blocks: Vec<DataBlock>,
}

pub enum Section {
    Type(TypeSection),
    Function(FunctionSection),
    Code(CodeSection),
    Export(ExportSection),
    Import(ImportSection),
    Memory(MemorySection),
    Start(StartSection),
    Unknown(UnknownSection),
    Global(GlobalSection),
    Table(TableSection),
    Data(DataSection),
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

fn wasm_i32(input: &[u8]) -> Result<(&[u8], i32, &[u8]), String> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_i32(0) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_i64(input: &[u8]) -> Result<(&[u8], i64, &[u8]), String> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_i64(0) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_f32(input: &[u8]) -> Result<(&[u8], f32, &[u8]), String> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_f32(0) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_f64(input: &[u8]) -> Result<(&[u8], f64, &[u8]), String> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_f64(0) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_string(input: &[u8]) -> Result<(&[u8], String), String> {
    let (input, num_chars) = wasm_u32(input)?;
    let (input, chars) = take(num_chars as usize)(input)?;
    let s = match alloc::str::from_utf8(chars) {
        Ok(b) => b.to_string(),
        Err(_) => return Err("could not parse utf8 string".to_string()),
    };
    Ok((input, s))
}

fn wasm_global_type(input: &[u8]) -> Result<(&[u8], ValueType, bool), String> {
    let (input, global_value_type) = take(1)(input)?;
    let (input, global_type) = take(1)(input)?;
    Ok((
        input,
        global_value_type[0].try_to_value_type()?,
        global_type[0] == MUTABLE,
    ))
}

fn wasm_limit(input: &[u8]) -> Result<(&[u8], usize, Option<usize>), String> {
    let (input, mem_type) = take(1)(input)?;
    match mem_type[0] {
        LIMIT_MIN_MAX => {
            let (input, min) = wasm_u32(input)?;
            let (input, max) = wasm_u32(input)?;
            Ok((input, min as usize, Some(max as usize)))
        }
        LIMIT_MIN => {
            let (input, min) = wasm_u32(input)?;
            Ok((input, min as usize, None))
        }
        _ => Err("unhandled memory type".to_string()),
    }
}

fn wasm_expression(input: &[u8]) -> Result<(&[u8], Vec<u8>), String> {
    let mut bytes = vec![];
    let mut ip = input;
    loop {
        let (input, op) = take(1)(ip)?;
        match op[0] {
            END => {
                ip = input;
                break;
            }
            I32_CONST => {
                bytes.push(op[0]);
                let (input, _, data) = wasm_i32(input)?;
                bytes.extend(data);
                ip = input;
            }
            I64_CONST => {
                bytes.push(op[0]);
                let (input, _, data) = wasm_i64(input)?;
                bytes.extend(data);
                ip = input;
            }

            F32_CONST => {
                bytes.push(op[0]);
                let (input, _, data) = wasm_f32(input)?;
                bytes.extend(data);
                ip = input;
            }

            F64_CONST => {
                bytes.push(op[0]);
                let (input, _, data) = wasm_f64(input)?;
                bytes.extend(data);
                ip = input;
            }
            _ => return Err("unknown expression".to_string()),
        }
    }
    Ok((ip, bytes))
}

fn section(input: &[u8]) -> Result<(&[u8], Section), String> {
    let (input, id) = take(1)(input)?;
    let (input, section_length) = wasm_u32(input)?;

    match id[0] {
        SECTION_TYPE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, wasm_type) = take(1)(input)?;
                match wasm_type[0] {
                    FUNC => {
                        let (input, num_inputs) = wasm_u32(input)?;
                        let (input, inputs) = take(num_inputs as usize)(input)?;
                        let (input, num_outputs) = wasm_u32(input)?;
                        let (input, outputs) = take(num_outputs as usize)(input)?;
                        Ok((
                            input,
                            WasmType::Function(FunctionType {
                                inputs: inputs.to_vec().try_to_value_types()?,
                                outputs: outputs.to_vec().try_to_value_types()?,
                            }),
                        ))
                    }
                    _ => Err("unknown type".to_string()),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Type(TypeSection { types: items })))
        }
        SECTION_FUNCTION => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| wasm_u32(input));
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                Section::Function(FunctionSection {
                    function_types: items,
                }),
            ))
        }
        SECTION_START => {
            let (input, start_function) = wasm_u32(input)?;
            Ok((
                input,
                Section::Start(StartSection {
                    start_function: start_function as usize,
                }),
            ))
        }
        SECTION_EXPORT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, name) = wasm_string(input)?;
                let (input, export_type) = take(1)(input)?;
                let (input, export_index) = wasm_u32(input)?;
                match export_type[0] {
                    DESC_FUNCTION => Ok((
                        input,
                        WasmExport::Function(Export {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_MEMORY => Ok((
                        input,
                        WasmExport::Memory(Export {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_GLOBAL => Ok((
                        input,
                        WasmExport::Global(Export {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_TABLE => Ok((
                        input,
                        WasmExport::Table(Export {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    _ => Err("unknown export".to_string()),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Export(ExportSection { exports: items })))
        }
        SECTION_CODE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, num_op_codes) = wasm_u32(input)?;

                let mut total_bytes = 0;
                let (num_local_vecs, byte_count) = match input.try_extract_u32(0) {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_string()),
                };
                total_bytes += byte_count;
                let (input, _) = take(byte_count as usize)(input)?;
                let mut ip2 = input;
                let mut local_vectors = vec![];
                for _ in 0..num_local_vecs {
                    let (num_locals, byte_count) = match input.try_extract_u32(0) {
                        Ok(r) => r,
                        Err(e) => return Err(e.to_string()),
                    };
                    let (input, _) = take(byte_count as usize)(input)?;
                    total_bytes += byte_count;

                    let (input, local_type) = take(1 as usize)(input)?;
                    total_bytes += 1;
                    local_vectors.push((num_locals, local_type[0].try_to_value_type()?));
                    ip2 = input;
                }
                let input = ip2;
                let (input, op_codes) =
                    take(num_op_codes as usize - 1 - total_bytes as usize)(input)?;
                let (input, _) = tag(&[END])(input)?;
                Ok((
                    input,
                    CodeBlock {
                        locals: local_vectors.to_vec(),
                        code: op_codes.to_vec(),
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Code(CodeSection { code_blocks: items })))
        }
        SECTION_IMPORT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, module_name) = wasm_string(input)?;
                let (input, name) = wasm_string(input)?;
                let (input, import_type) = take(1)(input)?;
                match import_type[0] {
                    DESC_FUNCTION => {
                        let (input, type_index) = wasm_u32(input)?;
                        Ok((
                            input,
                            WasmImport::Function(FunctionImport {
                                module_name,
                                name,
                                type_index: type_index as usize,
                            }),
                        ))
                    }
                    DESC_GLOBAL => {
                        let (input, min_pages, max_pages) = wasm_limit(input)?;
                        Ok((
                            input,
                            WasmImport::Memory(MemoryImport {
                                module_name,
                                name,
                                min_pages,
                                max_pages,
                            }),
                        ))
                    }
                    DESC_TABLE => {
                        let (input, element_type) = take(1)(input)?;
                        let (input, min, max) = wasm_limit(input)?;
                        Ok((
                            input,
                            WasmImport::Table(TableImport {
                                module_name,
                                name,
                                element_type: element_type[0],
                                min,
                                max,
                            }),
                        ))
                    }
                    DESC_MEMORY => {
                        let (input, value_type, is_mutable) = wasm_global_type(input)?;
                        Ok((
                            input,
                            WasmImport::Global(GlobalImport {
                                module_name,
                                name,
                                value_type,
                                is_mutable,
                            }),
                        ))
                    }
                    _ => Err("unknown export".to_string()),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Import(ImportSection { imports: items })))
        }
        SECTION_GLOBAL => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, value_type, is_mutable) = wasm_global_type(input)?;
                let (input, expression) = wasm_expression(input)?;
                Ok((
                    input,
                    Global {
                        value_type,
                        is_mutable,
                        expression,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Global(GlobalSection { globals: items })))
        }
        SECTION_TABLE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, element_type) = take(1)(input)?;
                if element_type[0] == ANYFUNC {
                    let (input, min, max) = wasm_limit(input)?;
                    Ok((
                        input,
                        Table {
                            element_type: element_type[0],
                            min,
                            max,
                        },
                    ))
                } else {
                    Err("unknown table type".to_string())
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Table(TableSection { tables: items })))
        }
        SECTION_DATA => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, mem_index) = wasm_u32(input)?;
                let (input, offset_expression) = wasm_expression(input)?;
                let (input, data_len) = wasm_u32(input)?;
                let (input, data) = take(data_len as usize)(input)?;
                Ok((
                    input,
                    DataBlock {
                        memory: mem_index as usize,
                        offset_expression,
                        data: data.to_vec(),
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Data(DataSection { data_blocks: items })))
        }
        SECTION_MEMORY => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, min, max) = wasm_limit(input)?;
                Ok((
                    input,
                    WasmMemory {
                        min_pages: min,
                        max_pages: max,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, Section::Memory(MemorySection { memories: items })))
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
    let (_, sections) = many(section)(input)?;
    Ok(Program { sections })
}

impl Program {
    pub fn parse(input: &[u8]) -> Result<Program, String> {
        wasm_module(input)
    }

    pub fn find_exported_function<'a>(&'a self, name: &str) -> Result<&'a Export, String> {
        let result = self.sections.iter().find(|x| {
            if let Section::Export(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(Section::Export(export_section)) = result {
            let result = self.sections.iter().find(|x| {
                if let Section::Code(_) = x {
                    true
                } else {
                    false
                }
            });
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
        let result = self.sections.iter().find(|x| {
            if let Section::Code(_) = x {
                true
            } else {
                false
            }
        });
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

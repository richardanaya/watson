#![no_std]
extern crate nom;
#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use nom::bytes::complete::{tag,take};
use nom::multi::many0;
use webassembly::*;
use alloc::string::String;
use crate::alloc::string::ToString;
use nom::IResult;

pub struct WasmFunctionType {
    pub inputs:Vec<u8>,
    pub outputs:Vec<u8>,
}

pub enum WasmType {
    Function(WasmFunctionType)
}

pub struct TypeSection {
    pub id:u8,
    pub data:Vec<u8>,
}

pub struct FunctionSection {
    pub function_types:Vec<u32>,
}

pub struct CodeSection {
    pub function_bodies:Vec<Vec<u8>>,
}

pub struct FunctionExport {
    pub name:String,
    pub index: u32,
}

pub struct MemoryExport {
    pub name:String,
    pub index: u32,
}

pub enum WasmExport {
    Function(FunctionExport),
    Memory(MemoryExport),
}

pub struct ExportSection {
    pub exports:Vec<WasmExport>,
}

pub struct UnknownSection {
    pub id:u8,
    pub data:Vec<u8>,
}

pub enum Section {
    Type(TypeSection),
    Function(FunctionSection),
    Code(CodeSection),
    Export(ExportSection),
    Unknown(UnknownSection),
}

pub struct Program {
    pub sections: Vec<Section>,
}


fn wasm_u32(input: &[u8]) -> IResult<&[u8], u32> {
    let (i,byte_count) = input.try_extract_u32(0).unwrap();
    let (input, _) = take(byte_count)(input)?;
    Ok((input,i))
}

fn section(input: &[u8]) -> IResult<&[u8], Section> {
    let (input, id) = take(1u8)(input)?;
    let (input,section_length) = wasm_u32(input)?;
    
    match id[0] {
        SECTION_TYPE => {
            let (input, data) = take(section_length)(input)?;
            Ok((input, Section::Type(TypeSection { id:id[0], data:data.to_vec() })))
        },
        SECTION_FUNCTION => {
            let (mut input,num_funcs) = wasm_u32(input)?;
            let mut function_types = vec![];
            let mut ip = input;
            for i in 0..num_funcs {
                let (input,index) = wasm_u32(ip)?;
                ip = input;
                function_types.push(index);
            }
            Ok((ip, Section::Function(FunctionSection {function_types })))
        },
        SECTION_EXPORT => {
            let (input,num_exports) = wasm_u32(input)?;
            let mut exports = vec![];
            let mut ip = input;
            for i in 0..num_exports {
                let (input,num_chars) = wasm_u32(ip)?;
                let (input,chars) = take(num_chars)(input)?;
                let name = alloc::str::from_utf8(chars).unwrap().to_string();
                let (input,export_type) = take(1u8)(input)?;
                let (input,export_index) = wasm_u32(input)?;
                ip = input;
                exports.push(match export_type[0]{
                    DESC_FUNCTION => WasmExport::Function(FunctionExport{name,index:export_index}),
                    DESC_MEMORY => WasmExport::Memory(MemoryExport{name,index:export_index}),
                    _ => panic!("unknown export")
                });
            }
            Ok((ip, Section::Export(ExportSection {exports:exports })))
        },
        SECTION_CODE => {
            let (mut input,num_funcs) = wasm_u32(input)?;
            let mut function_bodies = vec![];
            let mut ip = input;
            for i in 0..num_funcs {
                let (input,num_op_codes) = wasm_u32(ip)?;
                let (input,op_codes) = take(num_op_codes)(input)?;
                ip = input;
                function_bodies.push(op_codes.to_vec());
            }
            Ok((ip, Section::Code(CodeSection {function_bodies })))
        },
        _ => {
            let (input, data) = take(section_length)(input)?;
            Ok((input, Section::Unknown(UnknownSection { id:id[0], data:data.to_vec() })))
        }
    }
}

fn wasm_module(input: &[u8]) -> IResult<&[u8], Program> {
    let (input, _) = tag(MAGIC_NUMBER)(input)?;
    let (input, _) = tag(VERSION_1)(input)?;
    let (input, sections) = many0(section)(input)?;
    Ok((input, Program { sections }))
}

impl Program {
    pub fn load(input: &[u8]) -> Result<Program, &'static str> {
        let result = wasm_module(input);
        match result {
            Ok((_, program)) => Ok(program),
            Err(_) => Err("failed to parse"),
        }
    }
}

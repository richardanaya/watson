#![no_std]
extern crate nom;
extern crate alloc;
use alloc::vec::Vec;
use nom::bytes::complete::{tag,take};
use nom::multi::many0;
use webassembly::*;

use nom::IResult;

pub struct TypeSection {
    pub id:u8,
    pub data:Vec<u8>,
}

pub struct FunctionSection {
    pub id:u8,
    pub data:Vec<u8>,
}

pub struct CodeSection {
    pub id:u8,
    pub data:Vec<u8>,
}

pub struct ExportSection {
    pub id:u8,
    pub data:Vec<u8>,
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

fn section(input: &[u8]) -> IResult<&[u8], Section> {
    let (input, id) = take(1u8)(input)?;
    let (section_length,section_length_count) = input.try_extract_u32(0).unwrap();
    let (input, _) = take(section_length_count)(input)?;
    let (input, data) = take(section_length)(input)?;
    match id[0] {
        SECTION_TYPE => Ok((input, Section::Type(TypeSection { id:id[0], data:data.to_vec() }))),
        SECTION_FUNCTION => Ok((input, Section::Function(FunctionSection { id:id[0], data:data.to_vec() }))),
        SECTION_EXPORT => Ok((input, Section::Export(ExportSection { id:id[0], data:data.to_vec() }))),
        SECTION_CODE => Ok((input, Section::Code(CodeSection { id:id[0], data:data.to_vec() }))),
        _ => Ok((input, Section::Unknown(UnknownSection { id:id[0], data:data.to_vec() })))
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

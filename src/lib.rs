#![no_std]
#[macro_use]
extern crate nom;
#[macro_use]
extern crate alloc;
use crate::alloc::string::ToString;
use alloc::vec::Vec;
use nom::bytes::complete::tag;
use webassembly::*;

use nom::IResult;

pub enum Section {
    UnknownSection,
}

pub struct Program {
    pub sections: Vec<Section>,
}

fn wasm_module(input: &[u8]) -> IResult<&[u8], Program> {
    let (input, _) = tag(MAGIC_NUMBER)(input)?;
    let (input, _) = tag(VERSION_1)(input)?;
    Ok((input, Program { sections: vec![] }))
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

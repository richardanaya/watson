#![no_std]
#[macro_use]
extern crate alloc;
extern crate serde;
use crate::core::*;
use crate::parser::*;

mod compiler;
mod core;
mod parser;
mod util;

pub fn parse<'p>(input: &'p [u8]) -> Result<ProgramView<'p>, &'static str> {
    wasm_module(input)
}

/// # Safety
///
/// This is an attempt to make this library compatible with c eventually
#[no_mangle]
#[cfg(feature = "c_extern")]
pub unsafe fn c_parse_web_assembly(ptr_wasm_bytes: *mut u8, len: usize) -> Program {
    let wasm_bytes = Vec::from_raw_parts(ptr_wasm_bytes, len, len);
    wasm_module(&wasm_bytes).unwrap().to_owned()
}

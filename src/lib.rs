#![no_std]
#[macro_use]
extern crate alloc;
extern crate serde;
extern crate webassembly;

mod compiler;
mod core;
mod interpreter;
mod parser;
mod util;
#[cfg(test)]
mod spec_tests;

pub use crate::core::common::*;
pub use crate::core::view::*;
use crate::core::wast::Wast;
pub use crate::core::Instruction;
pub use crate::core::Program;
pub use crate::core::ProgramView;
pub use crate::interpreter::*;

pub fn parse<'p>(input: &'p [u8]) -> Result<core::ProgramView<'p>, &'static str> {
    parser::wasm::wasm_module(input)
}

pub fn parse_wast<'p>(input: &'p [u8]) -> Result<Wast, &'static str> {
    parser::wast::wast_file(input)
}

/// # Safety
///
/// This is an attempt to make this library compatible with c eventually
#[no_mangle]
#[cfg(feature = "c_extern")]
pub unsafe fn c_parse_web_assembly(ptr_wasm_bytes: *mut u8, len: usize) -> core::Program {
    let wasm_bytes = Vec::from_raw_parts(ptr_wasm_bytes, len, len);
    parser::wasm_module(&wasm_bytes).unwrap().to_owned()
}

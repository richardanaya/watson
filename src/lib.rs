#![no_std]
#[macro_use]
extern crate alloc;
extern crate serde;

mod compiler;
mod core;
mod interpreter;
mod parser;
mod util;

pub fn parse<'p>(input: &'p [u8]) -> Result<core::ProgramView<'p>, &'static str> {
    parser::wasm_module(input)
}

pub fn create_interpreter(p: core::Program) -> interpreter::Interpreter {
    interpreter::Interpreter::from_program(p)
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

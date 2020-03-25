#![no_std]
#[macro_use]
extern crate nom;
#[macro_use]
extern crate alloc;
use crate::alloc::string::ToString;

use alloc::vec::Vec;

use nom::{
    IResult,
};

#[no_mangle]
fn malloc(size: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

extern "C" {
    fn _log(msg:*const u8);
}

fn log(msg:&str){
    let mut s = msg.to_string();
    s.push_str("\0");
    unsafe { _log(s.as_ptr()) }
}

struct WasmProgram {}

impl WasmProgram {
    fn execute(&self, _fn_name:&str,_args:Vec<usize>) -> f64 {
        0.0
    }
}

fn wasm_module(input: &[u8]) -> IResult<&[u8], WasmProgram> {
    Ok((input, WasmProgram{}))
}

#[no_mangle]
fn run(ptr: *mut u8, len: usize) -> f64 {
    let wasm_bytes = unsafe {
        Vec::from_raw_parts(ptr, len, len)
    };

    log("starting interpreter");
    if let Ok((_,program)) = wasm_module(&wasm_bytes) {
        program.execute("main",vec![0,0])
    } else {
        log("something bad happened");
        panic!("fail")
    }
}

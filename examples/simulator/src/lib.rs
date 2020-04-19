extern crate alloc;
use alloc::string::{String,ToString};
use alloc::vec::Vec;
use watson::*;

extern "C" {
    fn _log(msg: *const u8);
}

fn log(msg: &str) {
    let mut s = msg.to_string();
    s.push_str("\0");
    unsafe { _log(s.as_ptr()) }
}

#[no_mangle]
fn malloc(size: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}


#[no_mangle]
fn load(ptr: *mut u8, len: usize) {
    log("starting interpreter");
    let wasm_bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    match watson::parse(&wasm_bytes) {
        Ok(p) => {
            let mut s = globals::get::<Simulator>();
            let prog = p.to_owned();
            s.program_string = serde_json::to_string(&prog).unwrap();
            s.program_string.push_str("\0");
            let mut interpreter = Interpreter::new(prog).unwrap();
            match interpreter.call("main", &[]) {
                Ok(executor) => {
                    log("called main function");
                    s.execution = Some(executor);
                },
                Err(e) => {
                    log("could not call main function");
                }
            }
            
        },
        Err(e) => {
            log(e);
        }
    }
}

struct Simulator {
    execution:Option<WasmExecution<Program>>,
    program_string:String,
    interpreter_string:String,
  }
  
impl Default for Simulator {
    fn default() -> Self {
        Simulator {
            execution:None,
            program_string: "".to_string(),
            interpreter_string: "".to_string(),
        }
    }
}

#[no_mangle]
fn get_program() ->  *const u8 {
    let s = globals::get::<Simulator>();
    return s.program_string.as_ptr();
}

#[no_mangle]
fn get_interpreter() ->  *const u8 {
    let mut s = globals::get::<Simulator>();
    s.interpreter_string = serde_json::to_string(&s.execution).unwrap();
    s.interpreter_string.push_str("\0");
    return s.interpreter_string.as_ptr();
}

#[no_mangle]
fn next_instruction() {
    let mut s = globals::get::<Simulator>();
    let execution_unit = match s.execution.as_mut().unwrap().next_unit() {
        Ok(r)=>r,
        Err(e)=>{
            log(e);
            return;
        }
   };
    let response = match execution_unit {
        // if an import is called, figure out what to do
        ExecutionUnit::CallImport(x) => {
            log(&x.name);
            ExecutionResponse::DoNothing
        }
        // if there's nothing left to do, break out of loop
        ExecutionUnit::Complete(v) => {
            log("PROGRAM COMPLETE!");
            return;
        },
        // handle other execution units with default behavior
        mut x @ _ => match x.evaluate() {
             Ok(r)=>r,
             Err(e)=>{
                 log(e);
                 panic!("");
             }
        },
    };
    match s.execution.as_mut().unwrap().execute(response) {
        Ok(r)=>r,
        Err(e)=>{
            log(e);
            return;
        }
   }
}
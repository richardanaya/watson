use crate::core::*;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub struct Interpreter<'a> {
    memory: Vec<u8>,
    program: Box<dyn InterpretableProgram + 'a>,
}

pub struct WasmValue;

impl WasmValue {
    pub fn to_i32(&self) -> i32 {
        0
    }
}

pub struct ImportCall {
    pub name: String,
    pub params: Vec<WasmValue>,
}

pub struct ExecutionResponse;
pub enum ExecutionUnit {
    CallImport(ImportCall),
    Complete,
}

pub trait InterpretableProgram {}

impl InterpretableProgram for ProgramView<'_> {}

impl<'a> Interpreter<'a> {
    pub fn new(p: impl InterpretableProgram + 'a) -> Self {
        Interpreter {
            memory: Vec::new(),
            program: Box::new(p),
        }
    }
    pub fn call(&mut self, name: &str, params: &[ValueType]) {}

    pub fn next(&self) -> ExecutionUnit {
        ExecutionUnit::Complete
    }

    pub fn execute(&mut self, _: ExecutionResponse) {}

    pub fn memory(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}

impl ExecutionUnit {
    pub fn evaluate(&mut self) -> ExecutionResponse {
        ExecutionResponse
    }
}

use crate::core::*;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub struct Interpreter<'a> {
    memory: Vec<u8>,
    program: Box<dyn InterpretableProgram + 'a>,
}

pub struct WasmValue;

pub trait ToWasmValue {
    fn to_wasm_value(&self) -> WasmValue;
}

impl ToWasmValue for i32 {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue
    }
}

impl WasmValue {
    pub fn to_i32(&self) -> i32 {
        0
    }
}

pub struct ImportCall {
    pub name: String,
    pub params: Vec<WasmValue>,
}

pub enum ExecutionResponse {
    DoNothing,
    Values(Vec<WasmValue>),
}

pub enum ExecutionUnit {
    CallImport(ImportCall),
    Complete(Vec<WasmValue>),
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
    pub fn call(&mut self, name: &str, params: &[ValueType]) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn next(&self) -> Result<ExecutionUnit, &'static str> {
        Ok(ExecutionUnit::Complete(vec![]))
    }

    pub fn execute(&mut self, _: ExecutionResponse) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn memory(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.memory)
    }
}

impl ExecutionUnit {
    pub fn evaluate(&mut self) -> Result<ExecutionResponse,&'static str> {
        Ok(ExecutionResponse::DoNothing)
    }
}

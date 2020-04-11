use crate::core::*;
use alloc::boxed::Box;

pub struct Interpreter<'a> {
    program: Box<dyn InterpretableProgram + 'a>,
}

pub struct ImportCall;
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
            program: Box::new(p),
        }
    }
    pub fn call(&mut self, name: &str, params: &[ValueType]) {}

    pub fn next(&self) -> ExecutionUnit {
        ExecutionUnit::Complete
    }

    pub fn execute(&mut self, _: ExecutionResponse) {}
}

impl ExecutionUnit {
    pub fn evaluate(&mut self) -> ExecutionResponse {
        ExecutionResponse
    }
}

use crate::core::*;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

pub struct Interpreter<'a> {
    pub memory: Rc<RefCell<Vec<u8>>>,
    pub program: Rc<RefCell<dyn InterpretableProgram + 'a>>,
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

pub trait WasmExecutor {
    fn next(&self) -> Result<ExecutionUnit, &'static str>;
    fn execute(&mut self, _: ExecutionResponse) -> Result<(), &'static str>;
    fn memory(&mut self) -> Option<Rc<RefCell<Vec<u8>>>>;
}

impl<'a> Interpreter<'a> {
    pub fn new(p: impl InterpretableProgram + 'a) -> Self {
        Interpreter {
            memory: Rc::new(RefCell::new(Vec::new())),
            program: Rc::new(RefCell::new(p)),
        }
    }
    pub fn call(
        &mut self,
        name: &str,
        params: &[ValueType],
    ) -> Result<Box<(dyn WasmExecutor + 'a)>, &'static str> {
        Ok(Box::new(WasmExecution {
            memory: self.memory.clone(),
            program: self.program.clone(),
        }))
    }
}

struct WasmExecution<'a> {
    pub memory: Rc<RefCell<Vec<u8>>>,
    pub program: Rc<RefCell<dyn InterpretableProgram + 'a>>,
}

impl<'a> WasmExecutor for WasmExecution<'a> {
    fn next(&self) -> Result<ExecutionUnit, &'static str> {
        Ok(ExecutionUnit::Complete(vec![]))
    }

    fn execute(&mut self, _: ExecutionResponse) -> Result<(), &'static str> {
        Ok(())
    }

    fn memory(&mut self) -> Option<Rc<RefCell<Vec<u8>>>> {
        Some(self.memory.clone())
    }
}

impl ExecutionUnit {
    pub fn evaluate(&mut self) -> Result<ExecutionResponse, &'static str> {
        Ok(ExecutionResponse::DoNothing)
    }
}

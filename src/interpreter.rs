use crate::core::*;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

pub struct Interpreter<T>
where
    T: InterpretableProgram,
{
    pub memory: Rc<RefCell<Vec<u8>>>,
    pub program: Rc<RefCell<T>>,
}

#[derive(Clone, Debug)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

pub trait ToWasmValue {
    fn to_wasm_value(&self) -> WasmValue;
}

impl ToWasmValue for i32 {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue::I32(*self)
    }
}

impl ToWasmValue for i64 {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue::I64(*self)
    }
}

impl ToWasmValue for f32 {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue::F32(*self)
    }
}

impl ToWasmValue for f64 {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue::F64(*self)
    }
}

impl WasmValue {
    pub fn to_i32(&self) -> i32 {
        match self {
            WasmValue::I32(i) => *i,
            WasmValue::I64(i) => *i as i32,
            WasmValue::F32(i) => *i as i32,
            WasmValue::F64(i) => *i as i32,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            WasmValue::I32(i) => *i as i64,
            WasmValue::I64(i) => *i,
            WasmValue::F32(i) => *i as i64,
            WasmValue::F64(i) => *i as i64,
        }
    }

    pub fn to_f32(&self) -> f32 {
        match self {
            WasmValue::I32(i) => *i as f32,
            WasmValue::I64(i) => *i as f32,
            WasmValue::F32(i) => *i,
            WasmValue::F64(i) => *i as f32,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            WasmValue::I32(i) => *i as f64,
            WasmValue::I64(i) => *i as f64,
            WasmValue::F32(i) => *i as f64,
            WasmValue::F64(i) => *i,
        }
    }
}

#[derive(Debug)]
pub struct ImportCall {
    pub module_name: String,
    pub name: String,
    pub params: Vec<WasmValue>,
}

pub enum ExecutionResponse {
    DoNothing,
    AddValues(Vec<WasmValue>),
    ValueStackModification(fn(&mut Vec<WasmValue>)),
}

#[derive(Debug)]
pub enum ExecutionUnit {
    CallImport(ImportCall),
    BasicInstruction(Instruction),
    Complete(Vec<WasmValue>),
}

pub trait InterpretableProgram {
    fn load_data_into_memory(&self, mem: &mut Vec<u8>) -> Result<(), &'static str>;
    fn initial_memory_size(&self) -> usize;
    fn import_fn_details(&self, index: usize) -> Result<(&str, &str, usize), &'static str>;
    fn import_fn_count(&self) -> usize;
    fn fetch_export_fn_index(&self, name: &str) -> Result<(usize, usize), &'static str>;
    fn fetch_instruction<'a>(
        &'a self,
        position: &[usize],
    ) -> Result<Option<&'a Instruction>, &'static str>;
}

impl InterpretableProgram for ProgramView<'_> {
    fn import_fn_details(&self, index: usize) -> Result<(&str, &str, usize), &'static str> {
        for s in self.sections.iter() {
            if let SectionView::Import(import_section) = s {
                let l: Vec<_> = import_section
                    .imports
                    .iter()
                    .filter(|x| matches!(x, WasmImportView::Function(_)))
                    .collect();
                if index < l.len() {
                    if let WasmImportView::Function(x) = l[index] {
                        return Ok((x.module_name, x.name, 1));
                    }
                } else {
                    return Err("import does not exist with that index");
                }
            }
        }
        Err("import section does not exist")
    }

    fn load_data_into_memory(&self, mem: &mut Vec<u8>) -> Result<(), &'static str> {
        for s in self.sections.iter() {
            if let SectionView::Data(d) = s {
                for db in d.data_blocks.iter() {
                    match db.offset_expression[0] {
                        Instruction::I32Const(x) => {
                            let offset = x as usize;
                            for (i, b) in db.data.iter().enumerate() {
                                mem[offset + i] = *b;
                            }
                        }
                        _ => return Err("I don't know how to build this memory yet"),
                    }
                }
            }
        }
        Ok(())
    }

    fn initial_memory_size(&self) -> usize {
        for s in self.sections.iter() {
            if let SectionView::Memory(m) = s {
                if !m.memories.is_empty() {
                    return m.memories[0].min_pages * 1024;
                }
            }
        }
        0
    }

    fn import_fn_count(&self) -> usize {
        for s in self.sections.iter() {
            if let SectionView::Import(import_section) = s {
                let l: Vec<_> = import_section
                    .imports
                    .iter()
                    .filter(|x| matches!(x, WasmImportView::Function(_)))
                    .collect();
                return l.len();
            }
        }
        0
    }

    fn fetch_export_fn_index(&self, name: &str) -> Result<(usize, usize), &'static str> {
        let ct = self.import_fn_count();
        let result = self
            .sections
            .iter()
            .enumerate()
            .find(|(_, x)| matches!(x, SectionView::Code(_)));
        let code_section_idx = match result {
            Some((i, _)) => i,
            None => return Err("Code section did not exist"),
        };
        for s in self.sections.iter() {
            if let SectionView::Export(export_section) = s {
                for e in export_section.exports.iter() {
                    if let WasmExportView::Function(f) = e {
                        if f.name == name {
                            return Ok((code_section_idx, f.index - ct));
                        }
                    }
                }
            }
        }
        Err("could not find export section")
    }

    fn fetch_instruction<'a>(
        &'a self,
        position: &[usize],
    ) -> Result<Option<&'a Instruction>, &'static str> {
        if let SectionView::Code(code_section) = &self.sections[position[0]] {
            let b = &code_section.code_blocks[position[1]];
            // TODO: handle nesting
            if position.len() > 2 {
                let i = 2;
                let instruction_index = position[i];
                let len = &b.instructions.len();
                if instruction_index < *len {
                    Ok(Some(&b.instructions[instruction_index]))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Err("cannot find code section")
        }
    }
}

impl<T> Interpreter<T>
where
    T: InterpretableProgram,
{
    pub fn new(p: T) -> Result<Self, &'static str> {
        let mem_size = p.initial_memory_size();
        let mut mem = vec![0; mem_size];
        p.load_data_into_memory(&mut mem)?;
        Ok(Interpreter {
            memory: Rc::new(RefCell::new(mem)),
            program: Rc::new(RefCell::new(p)),
        })
    }
    pub fn call(
        &mut self,
        name: &str,
        params: &[WasmValue],
    ) -> Result<WasmExecution<T>, &'static str> {
        let (section_index, function_index) = self.program.borrow().fetch_export_fn_index(name)?;
        let import_fn_count = self.program.borrow().import_fn_count();
        Ok(WasmExecution {
            import_fn_count,
            value_stack: params.to_vec(),
            current_position: vec![section_index, function_index],
            memory: self.memory.clone(),
            program: self.program.clone(),
        })
    }
}

pub struct WasmExecution<T>
where
    T: InterpretableProgram,
{
    import_fn_count: usize,
    pub value_stack: Vec<WasmValue>,
    pub current_position: Vec<usize>,
    pub memory: Rc<RefCell<Vec<u8>>>,
    pub program: Rc<RefCell<T>>,
}

impl<T> WasmExecution<T>
where
    T: InterpretableProgram,
{
    pub fn next_unit(&mut self) -> Result<ExecutionUnit, &'static str> {
        let p = self.program.borrow();
        if self.current_position.len() == 2 {
            self.current_position.push(0);
        } else {
            let len = self.current_position.len() - 1;
            self.current_position[len] += 1;
        }
        let first_function_instruction = p.fetch_instruction(&self.current_position)?;
        if let Some(instruction) = first_function_instruction {
            let unit = match instruction {
                Instruction::Call(fn_index) => {
                    if *fn_index < self.import_fn_count {
                        let (module_name, name, param_ct) = p.import_fn_details(*fn_index)?;
                        let mut params = vec![];
                        for _ in 0..param_ct {
                            let p = match self.value_stack.pop() {
                                Some(p) => p,
                                None => return Err("ran out of values on value stack"),
                            };
                            params.push(p);
                        }
                        ExecutionUnit::CallImport(ImportCall {
                            module_name: module_name.to_string(),
                            name: name.to_string(),
                            params,
                        })
                    } else {
                        return Err("cannot call non-imports yet");
                    }
                }
                x @ Instruction::Drop | x @ Instruction::I32Const(_) => {
                    ExecutionUnit::BasicInstruction(x.clone())
                }
                _ => return Err("cannot interpret this instruction yet"),
            };
            Ok(unit)
        } else {
            // TODO: handle nested function call
            Ok(ExecutionUnit::Complete(vec![]))
        }
    }

    pub fn execute(&mut self, r: ExecutionResponse) -> Result<(), &'static str> {
        match r {
            ExecutionResponse::ValueStackModification(f) => f(&mut self.value_stack),
            ExecutionResponse::AddValues(mut v) => {
                while let Some(wv) = v.pop() {
                    self.value_stack.push(wv);
                }
            }
            ExecutionResponse::DoNothing => {}
        }
        Ok(())
    }

    pub fn memory(&mut self) -> Option<Rc<RefCell<Vec<u8>>>> {
        Some(self.memory.clone())
    }
}

impl ExecutionUnit {
    pub fn evaluate(&mut self) -> Result<ExecutionResponse, &'static str> {
        let response = match self {
            ExecutionUnit::BasicInstruction(i) => match i {
                Instruction::Drop => ExecutionResponse::ValueStackModification(|stack| {
                    stack.pop();
                }),
                Instruction::I32Const(v) => ExecutionResponse::AddValues(vec![v.to_wasm_value()]),
                _ => return Err("no default evaluation for basic instruction"),
            },
            _ => return Err("no default evaluation"),
        };
        Ok(response)
    }
}

use crate::core::*;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use spin::Mutex;

pub struct Interpreter<T>
where
    T: InterpretableProgram,
{
    pub memory: Arc<Mutex<Vec<u8>>>,
    pub program: Arc<Mutex<T>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

pub trait ToWasmValue {
    fn to_wasm_value(&self) -> WasmValue;
}

impl ToWasmValue for usize {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue::I32(*self as i32)
    }
}

impl ToWasmValue for u32 {
    fn to_wasm_value(&self) -> WasmValue {
        WasmValue::I32((*self as u32).try_into().unwrap())
    }
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
    ValueStackModification(fn(&mut Vec<WasmValue>) -> Result<(), &'static str>),
    GetRegister(u32),
    SetRegister(u32),
    ThrowError(&'static str),
    GetMemorySize,
}

#[derive(Debug)]
pub enum ExecutionUnit {
    CallImport(ImportCall),
    BasicInstruction(Instruction),
    Unreachable,
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
    fn create_locals(&self, position: &[usize]) -> Result<Vec<WasmValue>, &'static str>;
}

impl InterpretableProgram for Program {
    fn import_fn_details(&self, index: usize) -> Result<(&str, &str, usize), &'static str> {
        for s in self.sections.iter() {
            if let Section::Import(import_section) = s {
                let l: Vec<_> = import_section
                    .imports
                    .iter()
                    .filter(|x| matches!(x, WasmImport::Function(_)))
                    .collect();
                if index < l.len() {
                    if let WasmImport::Function(x) = l[index] {
                        return Ok((&x.module_name, &x.name, 1));
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
            if let Section::Data(d) = s {
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
            if let Section::Memory(m) = s {
                if !m.memories.is_empty() {
                    return m.memories[0].min_pages * 1024;
                }
            }
        }
        0
    }

    fn import_fn_count(&self) -> usize {
        for s in self.sections.iter() {
            if let Section::Import(import_section) = s {
                let l: Vec<_> = import_section
                    .imports
                    .iter()
                    .filter(|x| matches!(x, WasmImport::Function(_)))
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
            .find(|(_, x)| matches!(x, Section::Code(_)));
        let code_section_idx = match result {
            Some((i, _)) => i,
            None => return Err("Code section did not exist"),
        };
        for s in self.sections.iter() {
            if let Section::Export(export_section) = s {
                for e in export_section.exports.iter() {
                    if let WasmExport::Function(f) = e {
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
        if let Section::Code(code_section) = &self.sections[position[0]] {
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

    fn create_locals(&self, position: &[usize]) -> Result<Vec<WasmValue>, &'static str> {
        let mut locals = vec![];
        if let Section::Code(code_section) = &self.sections[position[0]] {
            let b = &code_section.code_blocks[position[1]];
            for l in b.locals.iter() {
                for _ in 0..l.count {
                    let v = match l.value_type {
                        ValueType::I32 => 0i32.to_wasm_value(),
                        ValueType::I64 => 0i64.to_wasm_value(),
                        ValueType::F32 => 0f32.to_wasm_value(),
                        ValueType::F64 => 0f64.to_wasm_value(),
                    };
                    locals.push(v);
                }
            }
        } else {
            return Err("cannot find code section");
        }
        Ok(locals)
    }
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

    fn create_locals(&self, position: &[usize]) -> Result<Vec<WasmValue>, &'static str> {
        let mut locals = vec![];
        if let SectionView::Code(code_section) = &self.sections[position[0]] {
            let b = &code_section.code_blocks[position[1]];
            for l in b.locals.iter() {
                for _ in 0..l.count {
                    let v = match l.value_type {
                        ValueType::I32 => 0i32.to_wasm_value(),
                        ValueType::I64 => 0i64.to_wasm_value(),
                        ValueType::F32 => 0f32.to_wasm_value(),
                        ValueType::F64 => 0f64.to_wasm_value(),
                    };
                    locals.push(v);
                }
            }
        } else {
            return Err("cannot find code section");
        }
        Ok(locals)
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
            memory: Arc::new(Mutex::new(mem)),
            program: Arc::new(Mutex::new(p)),
        })
    }
    pub fn call(
        &mut self,
        name: &str,
        params: &[WasmValue],
    ) -> Result<WasmExecution<T>, &'static str> {
        WasmExecution::new(name, params, self.program.clone(), self.memory.clone())
    }
}

#[derive(Deserialize, Serialize)]
pub struct WasmExecution<T>
where
    T: InterpretableProgram,
{
    #[serde(skip)]
    import_fn_count: usize,
    pub call_stack: (usize, Vec<WasmValue>),
    pub value_stack: Vec<WasmValue>,
    pub current_position: Vec<usize>,
    #[serde(skip)]
    pub memory: Arc<Mutex<Vec<u8>>>,
    #[serde(skip)]
    pub program: Arc<Mutex<T>>,
}

impl<T> WasmExecution<T>
where
    T: InterpretableProgram,
{
    pub fn new(
        name: &str,
        params: &[WasmValue],
        program: Arc<Mutex<T>>,
        memory: Arc<Mutex<Vec<u8>>>,
    ) -> Result<Self, &'static str> {
        let p = program.lock();
        let (section_index, function_index) = p.fetch_export_fn_index(name)?;
        let position = vec![section_index, function_index];
        let locals = p.create_locals(&position)?;
        let import_fn_count = p.import_fn_count();
        Ok(WasmExecution {
            call_stack: (function_index, locals),
            import_fn_count,
            value_stack: params.to_vec(),
            current_position: position,
            memory,
            program: program.clone(),
        })
    }

    pub fn next_unit(&mut self) -> Result<ExecutionUnit, &'static str> {
        let p = self.program.lock();
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
                    if ((*fn_index) as usize) < self.import_fn_count {
                        let (module_name, name, param_ct) =
                            p.import_fn_details((*fn_index) as usize)?;
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
                Instruction::Unreachable => ExecutionUnit::Unreachable,
                x => ExecutionUnit::BasicInstruction(x.clone()),
            };
            Ok(unit)
        } else {
            // TODO: handle nested function call
            Ok(ExecutionUnit::Complete(vec![]))
        }
    }

    pub fn execute(&mut self, r: ExecutionResponse) -> Result<(), &'static str> {
        match r {
            ExecutionResponse::GetMemorySize => self
                .value_stack
                .push(self.memory.lock().len().to_wasm_value()),
            ExecutionResponse::ValueStackModification(f) => f(&mut self.value_stack)?,
            ExecutionResponse::AddValues(mut v) => {
                while let Some(wv) = v.pop() {
                    self.value_stack.push(wv);
                }
            }
            ExecutionResponse::GetRegister(v) => {
                self.value_stack.push(self.call_stack.1[v as usize]);
            }
            ExecutionResponse::SetRegister(v) => {
                if let Some(p) = self.value_stack.pop() {
                    self.call_stack.1[v as usize] = p;
                } else {
                    return Err("can't set register because value stack is empty");
                }
            }
            ExecutionResponse::ThrowError(msg) => return Err(msg),
            ExecutionResponse::DoNothing => {}
        }
        Ok(())
    }

    pub fn memory(&mut self) -> Option<Arc<Mutex<Vec<u8>>>> {
        Some(self.memory.clone())
    }
}

impl ExecutionUnit {
    pub fn evaluate(&mut self) -> Result<ExecutionResponse, &'static str> {
        let response = match self {
            ExecutionUnit::Unreachable => ExecutionResponse::ThrowError("Reached unreachable"),
            ExecutionUnit::BasicInstruction(i) => match i {
                Instruction::Raw(b) => {
                    return Err("Cannot handle raw instruction.");
                }
                Instruction::Unreachable => {
                    return Err("Cannot handle unreachable.");
                }
                Instruction::Nop => ExecutionResponse::DoNothing,
                Instruction::Block(block_type, instructions) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::Loop(block_type, instructions) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::If(block_type, if_instructions, else_instructions) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::Br(i) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::BrIf(i) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::BrTable(labels, label_index) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::Return => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::Call(i) => {
                    return Err("Cannot handle call.");
                }
                Instruction::CallIndirect(i) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::Drop => ExecutionResponse::ValueStackModification(|stack| {
                    stack.pop();
                    Ok(())
                }),
                Instruction::Select => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::LocalGet(i) => ExecutionResponse::GetRegister(*i),
                Instruction::LocalSet(i) => ExecutionResponse::SetRegister(*i),
                Instruction::LocalTee(i) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::GlobalGet(i) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::GlobalSet(i) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Load(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Load(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Load(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Load8S(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Load8U(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Load16S(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Load16U(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load8S(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load8U(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load16S(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load16U(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load32S(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Load32U(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Store(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Store(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Store(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Store(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Store8(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Store16(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Store8(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Store16(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Store32(align, offset) => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::MemorySize => ExecutionResponse::GetMemorySize,
                Instruction::MemoryGrow => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Const(i) => ExecutionResponse::AddValues(vec![i.to_wasm_value()]),
                Instruction::I64Const(i) => ExecutionResponse::AddValues(vec![i.to_wasm_value()]),
                Instruction::F32Const(f) => ExecutionResponse::AddValues(vec![f.to_wasm_value()]),
                Instruction::F64Const(f) => ExecutionResponse::AddValues(vec![f.to_wasm_value()]),
                Instruction::I32Eqz => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Eq => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Ne => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32LtS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32LtU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32GtS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32GtU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32LeS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32LeU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32GeS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32GeU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Eqz => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Eq => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Ne => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64LtS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64LtU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64GtS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64GtU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64LeS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64LeU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64GeS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64GeU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Eq => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Ne => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Lt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Gt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Le => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Ge => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Eq => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Ne => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Lt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Gt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Le => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Ge => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Clz => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Ctz => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Popcnt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Add => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Sub => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Mul => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32DivS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32DivU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32RemS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32RemU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32And => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Or => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Xor => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Shl => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32ShrS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32ShrU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Rotl => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32Rotr => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Clz => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Ctz => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Popcnt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Add => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Sub => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Mul => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64DivS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64DivU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64RemS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64RemU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64And => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Or => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Xor => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Shl => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64ShrS => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64ShrU => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Rotl => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64Rotr => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Abs => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Neg => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Ceil => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Floor => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Trunc => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Nearest => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Sqrt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Add => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Sub => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Mul => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Div => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Min => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Max => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32Copysign => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Abs => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Neg => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Ceil => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Floor => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Trunc => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Nearest => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Sqrt => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Add => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Sub => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Mul => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Div => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Min => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Max => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64Copysign => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32wrapF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32TruncSF32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32TruncUF32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32TruncSF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32TruncUF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64ExtendSI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64ExtendUI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64TruncSF32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64TruncUF32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64TruncSF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64TruncUF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32ConvertSI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32ConvertUI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32ConvertSI64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32ConvertUI64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32DemoteF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64ConvertSI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64ConvertUI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64ConvertSI64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64ConvertUI64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64PromoteF32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I32ReinterpretF32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::I64ReinterpretF64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F32ReinterpretI32 => {
                    return Err("no default evaluation for basic instruction yet");
                }
                Instruction::F64ReinterpretI64 => {
                    return Err("no default evaluation for basic instruction yet");
                }
            },
            _ => return Err("no default evaluation"),
        };
        Ok(response)
    }
}

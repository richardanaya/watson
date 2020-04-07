use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use webassembly::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "params")]
#[repr(C)]
pub enum Instruction {
    Unreachable,
    Nop,
    Block(u8, Vec<Instruction>),
    Loop(u8, Vec<Instruction>),
    If(u8, Vec<Instruction>, Option<Vec<Instruction>>),
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Call(u32),
    CallIndirect(u32),
    Drop,
    Select,
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Load8S(u32, u32),
    I32Load8U(u32, u32),
    I32Load16S(u32, u32),
    I32Load16U(u32, u32),
    I64Load8S(u32, u32),
    I64Load8U(u32, u32),
    I64Load16S(u32, u32),
    I64Load16U(u32, u32),
    I64Load32S(u32, u32),
    I64Load32U(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    I32Store8(u32, u32),
    I32Store16(u32, u32),
    I64Store8(u32, u32),
    I64Store16(u32, u32),
    I64Store32(u32, u32),
    MemorySize,
    MemoryGrow,
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32wrapF64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

impl Instruction {
    pub fn extend_wasm_bytes(&self, v: &mut Vec<u8>) {
        match self {
            Instruction::Unreachable => {
                v.push(webassembly::UNREACHABLE);
            }
            Instruction::Nop => {
                v.push(webassembly::NOP);
            }
            Instruction::Block(block_type, instructions) => {
                v.push(webassembly::BLOCK);
                v.push(*block_type);
                for i in instructions.iter() {
                    i.extend_wasm_bytes(v);
                }
                v.push(webassembly::END);
            }
            Instruction::Loop(block_type, instructions) => {
                v.push(webassembly::LOOP);
                v.push(*block_type);
                for i in instructions.iter() {
                    i.extend_wasm_bytes(v);
                }
                v.push(webassembly::END);
            }
            Instruction::If(block_type, if_instructions, else_instructions) => {
                v.push(webassembly::IF);
                v.push(*block_type);
                for i in if_instructions.iter() {
                    i.extend_wasm_bytes(v);
                }
                if let Some(e) = else_instructions {
                    v.push(webassembly::ELSE);
                    for i in e.iter() {
                        i.extend_wasm_bytes(v);
                    }
                }
                v.push(webassembly::END);
            }
            Instruction::Br(i) => {
                v.push(webassembly::BR);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::BrIf(i) => {
                v.push(webassembly::BR_IF);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::BrTable(labels, label_index) => {
                v.push(webassembly::BR_TABLE);
                v.extend(labels.len().to_wasm_bytes());
                for l in labels.iter() {
                    v.extend(l.to_wasm_bytes());
                }
                v.extend(label_index.to_wasm_bytes());
            }
            Instruction::Return => {
                v.push(webassembly::RETURN);
            }
            Instruction::Call(i) => {
                v.push(webassembly::CALL);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::CallIndirect(i) => {
                v.push(webassembly::CALL_INDIRECT);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::Drop => {
                v.push(webassembly::DROP);
            }
            Instruction::Select => {
                v.push(webassembly::SELECT);
            }
            Instruction::LocalGet(i) => {
                v.push(webassembly::LOCAL_GET);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::LocalSet(i) => {
                v.push(webassembly::LOCAL_SET);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::LocalTee(i) => {
                v.push(webassembly::LOCAL_TEE);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::GlobalGet(i) => {
                v.push(webassembly::GLOBAL_GET);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::GlobalSet(i) => {
                v.push(webassembly::GLOBAL_SET);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::I32Load(align, offset) => {
                v.push(webassembly::I32_LOAD);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load(align, offset) => {
                v.push(webassembly::I64_LOAD);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::F32Load(align, offset) => {
                v.push(webassembly::F32_LOAD);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::F64Load(align, offset) => {
                v.push(webassembly::F64_LOAD);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Load8S(align, offset) => {
                v.push(webassembly::I32_LOAD8_S);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Load8U(align, offset) => {
                v.push(webassembly::I32_LOAD8_U);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Load16S(align, offset) => {
                v.push(webassembly::I32_LOAD16_S);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Load16U(align, offset) => {
                v.push(webassembly::I32_LOAD16_U);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load8S(align, offset) => {
                v.push(webassembly::I64_LOAD8_S);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load8U(align, offset) => {
                v.push(webassembly::I64_LOAD8_U);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load16S(align, offset) => {
                v.push(webassembly::I64_LOAD16_S);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load16U(align, offset) => {
                v.push(webassembly::I64_LOAD16_U);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load32S(align, offset) => {
                v.push(webassembly::I64_LOAD32_S);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Load32U(align, offset) => {
                v.push(webassembly::I64_LOAD32_U);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Store(align, offset) => {
                v.push(webassembly::I32_STORE);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Store(align, offset) => {
                v.push(webassembly::I64_STORE);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::F32Store(align, offset) => {
                v.push(webassembly::F32_STORE);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::F64Store(align, offset) => {
                v.push(webassembly::F64_STORE);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Store8(align, offset) => {
                v.push(webassembly::I32_STORE8);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I32Store16(align, offset) => {
                v.push(webassembly::I32_STORE16);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Store8(align, offset) => {
                v.push(webassembly::I64_STORE8);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Store16(align, offset) => {
                v.push(webassembly::I64_STORE16);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::I64Store32(align, offset) => {
                v.push(webassembly::I64_STORE32);
                v.extend(align.to_wasm_bytes());
                v.extend(offset.to_wasm_bytes());
            }
            Instruction::MemorySize => {
                v.push(webassembly::MEMORY_SIZE);
            }
            Instruction::MemoryGrow => {
                v.push(webassembly::MEMORY_GROW);
            }
            Instruction::I32Const(i) => {
                v.push(webassembly::I32_CONST);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::I64Const(i) => {
                v.push(webassembly::I64_CONST);
                v.extend(i.to_wasm_bytes());
            }
            Instruction::F32Const(f) => {
                v.push(webassembly::F32_CONST);
                v.extend(f.to_wasm_bytes());
            }
            Instruction::F64Const(f) => {
                v.push(webassembly::F64_CONST);
                v.extend(f.to_wasm_bytes());
            }
            Instruction::I32Eqz => {
                v.push(webassembly::I32_EQZ);
            }
            Instruction::I32Eq => {
                v.push(webassembly::I32_EQ);
            }
            Instruction::I32Ne => {
                v.push(webassembly::I32_NE);
            }
            Instruction::I32LtS => {
                v.push(webassembly::I32_LT_S);
            }
            Instruction::I32LtU => {
                v.push(webassembly::I32_LT_U);
            }
            Instruction::I32GtS => {
                v.push(webassembly::I32_GT_S);
            }
            Instruction::I32GtU => {
                v.push(webassembly::I32_GT_U);
            }
            Instruction::I32LeS => {
                v.push(webassembly::I32_LE_S);
            }
            Instruction::I32LeU => {
                v.push(webassembly::I32_LE_U);
            }
            Instruction::I32GeS => {
                v.push(webassembly::I32_GE_S);
            }
            Instruction::I32GeU => {
                v.push(webassembly::I32_GE_U);
            }
            Instruction::I64Eqz => {
                v.push(webassembly::I64_EQZ);
            }
            Instruction::I64Eq => {
                v.push(webassembly::I64_EQ);
            }
            Instruction::I64Ne => {
                v.push(webassembly::I64_NE);
            }
            Instruction::I64LtS => {
                v.push(webassembly::I64_LT_S);
            }
            Instruction::I64LtU => {
                v.push(webassembly::I64_LT_U);
            }
            Instruction::I64GtS => {
                v.push(webassembly::I64_GT_S);
            }
            Instruction::I64GtU => {
                v.push(webassembly::I64_GT_U);
            }
            Instruction::I64LeS => {
                v.push(webassembly::I64_LE_S);
            }
            Instruction::I64LeU => {
                v.push(webassembly::I64_LE_U);
            }
            Instruction::I64GeS => {
                v.push(webassembly::I64_GE_S);
            }
            Instruction::I64GeU => {
                v.push(webassembly::I64_GE_U);
            }
            Instruction::F32Eq => {
                v.push(webassembly::F32_EQ);
            }
            Instruction::F32Ne => {
                v.push(webassembly::F32_NE);
            }
            Instruction::F32Lt => {
                v.push(webassembly::F32_LT);
            }
            Instruction::F32Gt => {
                v.push(webassembly::F32_GT);
            }
            Instruction::F32Le => {
                v.push(webassembly::F32_LE);
            }
            Instruction::F32Ge => {
                v.push(webassembly::F32_GE);
            }
            Instruction::F64Eq => {
                v.push(webassembly::F64_EQ);
            }
            Instruction::F64Ne => {
                v.push(webassembly::F64_NE);
            }
            Instruction::F64Lt => {
                v.push(webassembly::F64_LT);
            }
            Instruction::F64Gt => {
                v.push(webassembly::F64_GT);
            }
            Instruction::F64Le => {
                v.push(webassembly::F64_LE);
            }
            Instruction::F64Ge => {
                v.push(webassembly::F64_GE);
            }
            Instruction::I32Clz => {
                v.push(webassembly::I32_CLZ);
            }
            Instruction::I32Ctz => {
                v.push(webassembly::I32_CTZ);
            }
            Instruction::I32Popcnt => {
                v.push(webassembly::I32_POPCNT);
            }
            Instruction::I32Add => {
                v.push(webassembly::I32_ADD);
            }
            Instruction::I32Sub => {
                v.push(webassembly::I32_SUB);
            }
            Instruction::I32Mul => {
                v.push(webassembly::I32_MUL);
            }
            Instruction::I32DivS => {
                v.push(webassembly::I32_DIV_S);
            }
            Instruction::I32DivU => {
                v.push(webassembly::I32_DIV_U);
            }
            Instruction::I32RemS => {
                v.push(webassembly::I32_REM_S);
            }
            Instruction::I32RemU => {
                v.push(webassembly::I32_REM_U);
            }
            Instruction::I32And => {
                v.push(webassembly::I32_AND);
            }
            Instruction::I32Or => {
                v.push(webassembly::I32_OR);
            }
            Instruction::I32Xor => {
                v.push(webassembly::I32_XOR);
            }
            Instruction::I32Shl => {
                v.push(webassembly::I32_SHL);
            }
            Instruction::I32ShrS => {
                v.push(webassembly::I32_SHR_S);
            }
            Instruction::I32ShrU => {
                v.push(webassembly::I32_SHR_U);
            }
            Instruction::I32Rotl => {
                v.push(webassembly::I32_ROTL);
            }
            Instruction::I32Rotr => {
                v.push(webassembly::I32_ROTR);
            }
            Instruction::I64Clz => {
                v.push(webassembly::I64_CLZ);
            }
            Instruction::I64Ctz => {
                v.push(webassembly::I64_CTZ);
            }
            Instruction::I64Popcnt => {
                v.push(webassembly::I64_POPCNT);
            }
            Instruction::I64Add => {
                v.push(webassembly::I64_ADD);
            }
            Instruction::I64Sub => {
                v.push(webassembly::I64_SUB);
            }
            Instruction::I64Mul => {
                v.push(webassembly::I64_MUL);
            }
            Instruction::I64DivS => {
                v.push(webassembly::I64_DIV_S);
            }
            Instruction::I64DivU => {
                v.push(webassembly::I64_DIV_U);
            }
            Instruction::I64RemS => {
                v.push(webassembly::I64_REM_S);
            }
            Instruction::I64RemU => {
                v.push(webassembly::I64_REM_U);
            }
            Instruction::I64And => {
                v.push(webassembly::I64_AND);
            }
            Instruction::I64Or => {
                v.push(webassembly::I64_OR);
            }
            Instruction::I64Xor => {
                v.push(webassembly::I64_XOR);
            }
            Instruction::I64Shl => {
                v.push(webassembly::I64_SHL);
            }
            Instruction::I64ShrS => {
                v.push(webassembly::I64_SHR_S);
            }
            Instruction::I64ShrU => {
                v.push(webassembly::I64_SHR_U);
            }
            Instruction::I64Rotl => {
                v.push(webassembly::I64_ROTL);
            }
            Instruction::I64Rotr => {
                v.push(webassembly::I64_ROTR);
            }
            Instruction::F32Abs => {
                v.push(webassembly::F32_ABS);
            }
            Instruction::F32Neg => {
                v.push(webassembly::F32_NEG);
            }
            Instruction::F32Ceil => {
                v.push(webassembly::F32_CEIL);
            }
            Instruction::F32Floor => {
                v.push(webassembly::F32_FLOOR);
            }
            Instruction::F32Trunc => {
                v.push(webassembly::F32_TRUNC);
            }
            Instruction::F32Nearest => {
                v.push(webassembly::F32_NEAREST);
            }
            Instruction::F32Sqrt => {
                v.push(webassembly::F32_SQRT);
            }
            Instruction::F32Add => {
                v.push(webassembly::F32_ADD);
            }
            Instruction::F32Sub => {
                v.push(webassembly::F32_SUB);
            }
            Instruction::F32Mul => {
                v.push(webassembly::F32_MUL);
            }
            Instruction::F32Div => {
                v.push(webassembly::F32_DIV);
            }
            Instruction::F32Min => {
                v.push(webassembly::F32_MIN);
            }
            Instruction::F32Max => {
                v.push(webassembly::F32_MAX);
            }
            Instruction::F32Copysign => {
                v.push(webassembly::F32_COPYSIGN);
            }
            Instruction::F64Abs => {
                v.push(webassembly::F64_ABS);
            }
            Instruction::F64Neg => {
                v.push(webassembly::F64_NEG);
            }
            Instruction::F64Ceil => {
                v.push(webassembly::F64_CEIL);
            }
            Instruction::F64Floor => {
                v.push(webassembly::F64_FLOOR);
            }
            Instruction::F64Trunc => {
                v.push(webassembly::F64_TRUNC);
            }
            Instruction::F64Nearest => {
                v.push(webassembly::F64_NEAREST);
            }
            Instruction::F64Sqrt => {
                v.push(webassembly::F64_SQRT);
            }
            Instruction::F64Add => {
                v.push(webassembly::F64_ADD);
            }
            Instruction::F64Sub => {
                v.push(webassembly::F64_SUB);
            }
            Instruction::F64Mul => {
                v.push(webassembly::F64_MUL);
            }
            Instruction::F64Div => {
                v.push(webassembly::F64_DIV);
            }
            Instruction::F64Min => {
                v.push(webassembly::F64_MIN);
            }
            Instruction::F64Max => {
                v.push(webassembly::F64_MAX);
            }
            Instruction::F64Copysign => {
                v.push(webassembly::F64_COPYSIGN);
            }
            Instruction::I32wrapF64 => {
                v.push(webassembly::I32_WRAP_F64);
            }
            Instruction::I32TruncSF32 => {
                v.push(webassembly::I32_TRUNC_S_F32);
            }
            Instruction::I32TruncUF32 => {
                v.push(webassembly::I32_TRUNC_U_F32);
            }
            Instruction::I32TruncSF64 => {
                v.push(webassembly::I32_TRUNC_S_F64);
            }
            Instruction::I32TruncUF64 => {
                v.push(webassembly::I32_TRUNC_U_F64);
            }
            Instruction::I64ExtendSI32 => {
                v.push(webassembly::I64_EXTEND_S_I32);
            }
            Instruction::I64ExtendUI32 => {
                v.push(webassembly::I64_EXTEND_U_I32);
            }
            Instruction::I64TruncSF32 => {
                v.push(webassembly::I64_TRUNC_S_F32);
            }
            Instruction::I64TruncUF32 => {
                v.push(webassembly::I64_TRUNC_U_F32);
            }
            Instruction::I64TruncSF64 => {
                v.push(webassembly::I64_TRUNC_S_F64);
            }
            Instruction::I64TruncUF64 => {
                v.push(webassembly::I64_TRUNC_U_F64);
            }
            Instruction::F32ConvertSI32 => {
                v.push(webassembly::F32_CONVERT_S_I32);
            }
            Instruction::F32ConvertUI32 => {
                v.push(webassembly::F32_CONVERT_U_I32);
            }
            Instruction::F32ConvertSI64 => {
                v.push(webassembly::F32_CONVERT_S_I32);
            }
            Instruction::F32ConvertUI64 => {
                v.push(webassembly::F32_CONVERT_U_I64);
            }
            Instruction::F32DemoteF64 => {
                v.push(webassembly::F32_DEMOTE_F64);
            }
            Instruction::F64ConvertSI32 => {
                v.push(webassembly::F64_CONVERT_S_I32);
            }
            Instruction::F64ConvertUI32 => {
                v.push(webassembly::F64_CONVERT_U_I32);
            }
            Instruction::F64ConvertSI64 => {
                v.push(webassembly::F64_CONVERT_S_I64);
            }
            Instruction::F64ConvertUI64 => {
                v.push(webassembly::F64_CONVERT_U_I64);
            }
            Instruction::F64PromoteF32 => {
                v.push(webassembly::F64_PROMOTE_F32);
            }
            Instruction::I32ReinterpretF32 => {
                v.push(webassembly::I32_REINTERPRET_F32);
            }
            Instruction::I64ReinterpretF64 => {
                v.push(webassembly::I64_REINTERPRET_F64);
            }
            Instruction::F32ReinterpretI32 => {
                v.push(webassembly::F32_REINTERPRET_I32);
            }
            Instruction::F64ReinterpretI64 => {
                v.push(webassembly::F64_REINTERPRET_I64);
            }
        }
    }
}

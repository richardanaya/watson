use crate::core::*;
use crate::util::*;
use alloc::vec::Vec;
use core::convert::TryInto;
use webassembly::*;

fn wasm_u32(input: &[u8]) -> Result<(&[u8], u32), &'static str> {
    let (i, byte_count) = match input.try_extract_u32(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i))
}

fn wasm_i32(input: &[u8]) -> Result<(&[u8], i32, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_i32(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_i64(input: &[u8]) -> Result<(&[u8], i64, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_i64(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_f32(input: &[u8]) -> Result<(&[u8], f32, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_f32(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_f64(input: &[u8]) -> Result<(&[u8], f64, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_f64(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_string(input: &[u8]) -> Result<(&[u8], &str), &'static str> {
    let (input, num_chars) = wasm_u32(input)?;
    let (input, chars) = take(num_chars as usize)(input)?;
    let s = match alloc::str::from_utf8(chars) {
        Ok(b) => b,
        Err(_) => return Err("could not parse utf8 string"),
    };
    Ok((input, s))
}

fn wasm_global_type(input: &[u8]) -> Result<(&[u8], ValueType, bool), &'static str> {
    let (input, global_value_type) = take(1)(input)?;
    let (input, global_type) = take(1)(input)?;
    Ok((
        input,
        global_value_type[0].try_into()?,
        global_type[0] == MUTABLE,
    ))
}

fn wasm_limit(input: &[u8]) -> Result<(&[u8], usize, Option<usize>), &'static str> {
    let (input, mem_type) = take(1)(input)?;
    match mem_type[0] {
        LIMIT_MIN_MAX => {
            let (input, min) = wasm_u32(input)?;
            let (input, max) = wasm_u32(input)?;
            Ok((input, min as usize, Some(max as usize)))
        }
        LIMIT_MIN => {
            let (input, min) = wasm_u32(input)?;
            Ok((input, min as usize, None))
        }
        _ => Err("unhandled memory type"),
    }
}

fn wasm_instruction(op: u8, input: &[u8]) -> Result<(&[u8], Instruction), &'static str> {
    let mut ip = input;
    let instruction;

    match op {
        UNREACHABLE => instruction = Instruction::Unreachable,
        NOP => instruction = Instruction::Nop,
        RETURN => instruction = Instruction::Return,

        BLOCK => {
            let (input, block_type) = take(1)(input)?;
            let (input, block_instructions) = wasm_expression(input)?;
            instruction = Instruction::Block(block_type[0], block_instructions);
            ip = input;
        }

        LOOP => {
            let (input, block_type) = take(1)(input)?;
            let (input, loop_instructions) = wasm_expression(input)?;
            instruction = Instruction::Loop(block_type[0], loop_instructions);
            ip = input;
        }

        IF => {
            let (input, block_type) = take(1)(input)?;
            let (input, if_instructions, else_instructions) = wasm_if_else(input)?;
            instruction = Instruction::If(block_type[0], if_instructions, else_instructions);
            ip = input;
        }

        BR => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::Br(idx);
            ip = input;
        }

        BR_IF => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::BrIf(idx);
            ip = input;
        }

        BR_TABLE => {
            let (input, num_labels) = wasm_u32(input)?;
            let parse_label = many_n(num_labels as usize, |input| wasm_u32(input));
            let (input, labels) = parse_label(input)?;
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::BrTable(labels, idx);
            ip = input;
        }

        CALL => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::Call(idx as usize);
            ip = input;
        }

        CALL_INDIRECT => {
            let (input, idx) = wasm_u32(input)?;
            let (input, _) = take(1)(input)?;
            instruction = Instruction::CallIndirect(idx);
            ip = input;
        }

        DROP => instruction = Instruction::Drop,
        SELECT => instruction = Instruction::Select,
        I32_CONST => {
            let (input, c, _) = wasm_i32(input)?;
            instruction = Instruction::I32Const(c);
            ip = input;
        }
        I64_CONST => {
            let (input, c, _) = wasm_i64(input)?;
            instruction = Instruction::I64Const(c);
            ip = input;
        }

        F32_CONST => {
            let (input, c, _) = wasm_f32(input)?;
            instruction = Instruction::F32Const(c);
            ip = input;
        }

        F64_CONST => {
            let (input, c, _) = wasm_f64(input)?;
            instruction = Instruction::F64Const(c);
            ip = input;
        }
        LOCAL_GET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::LocalGet(idx);
            ip = input;
        }
        LOCAL_SET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::LocalSet(idx);
            ip = input;
        }
        LOCAL_TEE => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::LocalTee(idx);
            ip = input;
        }
        GLOBAL_GET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::GlobalGet(idx);
            ip = input;
        }
        GLOBAL_SET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::GlobalSet(idx);
            ip = input;
        }
        I32_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load(align, offset);
            ip = input;
        }

        I64_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load(align, offset);
            ip = input;
        }

        F32_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F32Load(align, offset);
            ip = input;
        }

        F64_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F64Load(align, offset);
            ip = input;
        }

        I32_LOAD8_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load8S(align, offset);
            ip = input;
        }

        I32_LOAD8_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load8U(align, offset);
            ip = input;
        }

        I32_LOAD16_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load16S(align, offset);
            ip = input;
        }

        I32_LOAD16_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load16U(align, offset);
            ip = input;
        }

        I64_LOAD8_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load8S(align, offset);
            ip = input;
        }

        I64_LOAD8_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load8U(align, offset);
            ip = input;
        }

        I64_LOAD16_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load16S(align, offset);
            ip = input;
        }

        I64_LOAD16_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load16U(align, offset);
            ip = input;
        }

        I64_LOAD32_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load32S(align, offset);
            ip = input;
        }

        I64_LOAD32_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load32U(align, offset);
            ip = input;
        }

        I32_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Store(align, offset);
            ip = input;
        }

        I64_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store(align, offset);
            ip = input;
        }

        F32_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F32Store(align, offset);
            ip = input;
        }
        F64_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F64Store(align, offset);
            ip = input;
        }

        I32_STORE8 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Store8(align, offset);
            ip = input;
        }

        I32_STORE16 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Store16(align, offset);
            ip = input;
        }

        I64_STORE8 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store8(align, offset);
            ip = input;
        }

        I64_STORE16 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store16(align, offset);
            ip = input;
        }

        I64_STORE32 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store32(align, offset);
            ip = input;
        }

        MEMORY_GROW => {
            let (input, _) = wasm_u32(input)?;
            instruction = Instruction::MemoryGrow;
            ip = input;
        }

        MEMORY_SIZE => {
            let (input, _) = wasm_u32(input)?;
            instruction = Instruction::MemorySize;
            ip = input;
        }

        I32_EQZ => instruction = Instruction::I32Eqz,
        I32_EQ => instruction = Instruction::I32Eq,
        I32_NE => instruction = Instruction::I32Ne,
        I32_LT_S => instruction = Instruction::I32LtS,
        I32_LT_U => instruction = Instruction::I32LtU,
        I32_GT_S => instruction = Instruction::I32GtS,
        I32_GT_U => instruction = Instruction::I32GtU,
        I32_LE_S => instruction = Instruction::I32LeS,
        I32_LE_U => instruction = Instruction::I32LeU,
        I32_GE_S => instruction = Instruction::I32GeS,
        I32_GE_U => instruction = Instruction::I32GeU,
        I64_EQZ => instruction = Instruction::I64Eqz,
        I64_EQ => instruction = Instruction::I64Eq,
        I64_NE => instruction = Instruction::I64Ne,
        I64_LT_S => instruction = Instruction::I64LtS,
        I64_LT_U => instruction = Instruction::I64LtU,
        I64_GT_S => instruction = Instruction::I64GtS,
        I64_GT_U => instruction = Instruction::I64GtU,
        I64_LE_S => instruction = Instruction::I64LeS,
        I64_LE_U => instruction = Instruction::I64LeU,
        I64_GE_S => instruction = Instruction::I64GeS,
        I64_GE_U => instruction = Instruction::I64GeU,
        F32_EQ => instruction = Instruction::F32Eq,
        F32_NE => instruction = Instruction::F32Ne,
        F32_LT => instruction = Instruction::F32Lt,
        F32_GT => instruction = Instruction::F32Gt,
        F32_LE => instruction = Instruction::F32Le,
        F32_GE => instruction = Instruction::F32Ge,
        F64_EQ => instruction = Instruction::F64Eq,
        F64_NE => instruction = Instruction::F64Ne,
        F64_LT => instruction = Instruction::F64Lt,
        F64_GT => instruction = Instruction::F64Gt,
        F64_LE => instruction = Instruction::F64Le,
        F64_GE => instruction = Instruction::F64Ge,
        I32_CLZ => instruction = Instruction::I32Clz,
        I32_CTZ => instruction = Instruction::I32Ctz,
        I32_POPCNT => instruction = Instruction::I32Popcnt,
        I32_ADD => instruction = Instruction::I32Add,
        I32_SUB => instruction = Instruction::I32Sub,
        I32_MUL => instruction = Instruction::I32Mul,
        I32_DIV_S => instruction = Instruction::I32DivS,
        I32_DIV_U => instruction = Instruction::I32DivU,
        I32_REM_S => instruction = Instruction::I32RemS,
        I32_REM_U => instruction = Instruction::I32RemU,
        I32_AND => instruction = Instruction::I32And,
        I32_OR => instruction = Instruction::I32Or,
        I32_XOR => instruction = Instruction::I32Xor,
        I32_SHL => instruction = Instruction::I32Shl,
        I32_SHR_S => instruction = Instruction::I32ShrS,
        I32_SHR_U => instruction = Instruction::I32ShrU,
        I32_ROTL => instruction = Instruction::I32Rotl,
        I32_ROTR => instruction = Instruction::I32Rotr,
        I64_CLZ => instruction = Instruction::I64Clz,
        I64_CTZ => instruction = Instruction::I64Ctz,
        I64_POPCNT => instruction = Instruction::I64Popcnt,
        I64_ADD => instruction = Instruction::I64Add,
        I64_SUB => instruction = Instruction::I64Sub,
        I64_MUL => instruction = Instruction::I64Mul,
        I64_DIV_S => instruction = Instruction::I64DivS,
        I64_DIV_U => instruction = Instruction::I64DivU,
        I64_REM_S => instruction = Instruction::I64RemS,
        I64_REM_U => instruction = Instruction::I64RemU,
        I64_AND => instruction = Instruction::I64And,
        I64_OR => instruction = Instruction::I64Or,
        I64_XOR => instruction = Instruction::I64Xor,
        I64_SHL => instruction = Instruction::I64Shl,
        I64_SHR_S => instruction = Instruction::I64ShrS,
        I64_SHR_U => instruction = Instruction::I64ShrU,
        I64_ROTL => instruction = Instruction::I64Rotl,
        I64_ROTR => instruction = Instruction::I64Rotr,
        F32_ABS => instruction = Instruction::F32Abs,
        F32_NEG => instruction = Instruction::F32Neg,
        F32_CEIL => instruction = Instruction::F32Ceil,
        F32_FLOOR => instruction = Instruction::F32Floor,
        F32_TRUNC => instruction = Instruction::F32Trunc,
        F32_NEAREST => instruction = Instruction::F32Nearest,
        F32_SQRT => instruction = Instruction::F32Sqrt,
        F32_ADD => instruction = Instruction::F32Add,
        F32_SUB => instruction = Instruction::F32Sub,
        F32_MUL => instruction = Instruction::F32Mul,
        F32_DIV => instruction = Instruction::F32Div,
        F32_MIN => instruction = Instruction::F32Min,
        F32_MAX => instruction = Instruction::F32Max,
        F32_COPYSIGN => instruction = Instruction::F32Copysign,
        F64_ABS => instruction = Instruction::F64Abs,
        F64_NEG => instruction = Instruction::F64Neg,
        F64_CEIL => instruction = Instruction::F64Ceil,
        F64_FLOOR => instruction = Instruction::F64Floor,
        F64_TRUNC => instruction = Instruction::F64Trunc,
        F64_NEAREST => instruction = Instruction::F64Nearest,
        F64_SQRT => instruction = Instruction::F64Sqrt,
        F64_ADD => instruction = Instruction::F64Add,
        F64_SUB => instruction = Instruction::F64Sub,
        F64_MUL => instruction = Instruction::F64Mul,
        F64_DIV => instruction = Instruction::F64Div,
        F64_MIN => instruction = Instruction::F64Min,
        F64_MAX => instruction = Instruction::F64Max,
        F64_COPYSIGN => instruction = Instruction::F64Copysign,
        I32_WRAP_F64 => instruction = Instruction::I32wrapF64,
        I32_TRUNC_S_F32 => instruction = Instruction::I32TruncSF32,
        I32_TRUNC_U_F32 => instruction = Instruction::I32TruncUF32,
        I32_TRUNC_S_F64 => instruction = Instruction::I32TruncSF64,
        I32_TRUNC_U_F64 => instruction = Instruction::I32TruncUF64,
        I64_EXTEND_S_I32 => instruction = Instruction::I64ExtendSI32,
        I64_EXTEND_U_I32 => instruction = Instruction::I64ExtendUI32,
        I64_TRUNC_S_F32 => instruction = Instruction::I64TruncSF32,
        I64_TRUNC_U_F32 => instruction = Instruction::I64TruncUF32,
        I64_TRUNC_S_F64 => instruction = Instruction::I64TruncSF64,
        I64_TRUNC_U_F64 => instruction = Instruction::I64TruncUF64,
        F32_CONVERT_S_I32 => instruction = Instruction::F32ConvertSI32,
        F32_CONVERT_U_I32 => instruction = Instruction::F32ConvertUI32,
        F32_CONVERT_S_I64 => instruction = Instruction::F32ConvertSI64,
        F32_CONVERT_U_I64 => instruction = Instruction::F32ConvertUI64,
        F32_DEMOTE_F64 => instruction = Instruction::F32DemoteF64,
        F64_CONVERT_S_I32 => instruction = Instruction::F64ConvertSI32,
        F64_CONVERT_U_I32 => instruction = Instruction::F64ConvertUI32,
        F64_CONVERT_S_I64 => instruction = Instruction::F64ConvertSI64,
        F64_CONVERT_U_I64 => instruction = Instruction::F64ConvertUI64,
        F64_PROMOTE_F32 => instruction = Instruction::F64PromoteF32,
        I32_REINTERPRET_F32 => instruction = Instruction::I32ReinterpretF32,
        I64_REINTERPRET_F64 => instruction = Instruction::I64ReinterpretF64,
        F32_REINTERPRET_I32 => instruction = Instruction::F32ReinterpretI32,
        F64_REINTERPRET_I64 => instruction = Instruction::F64ReinterpretI64,
        _ => return Err("unknown expression"),
    };
    Ok((ip, instruction))
}

fn wasm_expression(input: &[u8]) -> Result<(&[u8], Vec<Instruction>), &'static str> {
    let mut instructions = vec![];
    let mut ip = input;
    loop {
        let (input, op) = take(1)(ip)?;
        ip = input;
        match op[0] {
            END => {
                ip = input;
                break;
            }

            _ => {
                let (input, instruction) = wasm_instruction(op[0], ip)?;
                instructions.push(instruction);
                ip = input;
            }
        }
    }
    Ok((ip, instructions))
}

type Instructions = Vec<Instruction>;

fn wasm_if_else(input: &[u8]) -> Result<(&[u8], Instructions, Option<Instructions>), &'static str> {
    let mut if_instructions = vec![];
    let mut else_instructions = vec![];
    let mut ip = input;
    let mut more = false;
    loop {
        let (input, op) = take(1)(ip)?;
        ip = input;
        match op[0] {
            END => {
                break;
            }
            ELSE => {
                more = true;
                break;
            }

            _ => {
                let (input, instruction) = wasm_instruction(op[0], ip)?;
                if_instructions.push(instruction);
                ip = input;
            }
        }
    }
    if more {
        loop {
            let (input, op) = take(1)(ip)?;
            ip = input;
            match op[0] {
                END => {
                    break;
                }

                _ => {
                    let (input, instruction) = wasm_instruction(op[0], ip)?;
                    else_instructions.push(instruction);
                    ip = input;
                }
            }
        }
        Ok((ip, if_instructions, Some(else_instructions)))
    } else {
        Ok((ip, if_instructions, None))
    }
}

fn section(input: &[u8]) -> Result<(&[u8], SectionView), &'static str> {
    let (input, id) = take(1)(input)?;
    let (input, section_length) = wasm_u32(input)?;
    match id[0] {
        SECTION_TYPE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, wasm_type) = take(1)(input)?;
                match wasm_type[0] {
                    FUNC => {
                        let (input, num_inputs) = wasm_u32(input)?;
                        let (input, inputs) = take(num_inputs as usize)(input)?;
                        let (input, num_outputs) = wasm_u32(input)?;
                        let (input, outputs) = take(num_outputs as usize)(input)?;
                        Ok((
                            input,
                            FunctionType {
                                inputs: inputs.to_vec().try_to_value_types()?,
                                outputs: outputs.to_vec().try_to_value_types()?,
                            },
                        ))
                    }
                    _ => Err("unknown type"),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Type(TypeSection { types: items })))
        }
        SECTION_FUNCTION => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let r = wasm_u32(input);
                match r {
                    Ok(n) => Ok((n.0, n.1 as usize)),
                    Err(e) => Err(e),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Function(FunctionSection {
                    function_types: items,
                }),
            ))
        }
        SECTION_START => {
            let (input, start_function) = wasm_u32(input)?;
            Ok((
                input,
                SectionView::Start(StartSection {
                    start_function: start_function as usize,
                }),
            ))
        }
        SECTION_EXPORT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, name) = wasm_string(input)?;
                let (input, export_type) = take(1)(input)?;
                let (input, export_index) = wasm_u32(input)?;
                match export_type[0] {
                    DESC_FUNCTION => Ok((
                        input,
                        WasmExportView::Function(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_MEMORY => Ok((
                        input,
                        WasmExportView::Memory(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_GLOBAL => Ok((
                        input,
                        WasmExportView::Global(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_TABLE => Ok((
                        input,
                        WasmExportView::Table(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    _ => Err("unknown export"),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Export(ExportSectionView { exports: items }),
            ))
        }
        SECTION_CODE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, _) = wasm_u32(input)?;
                let (input, num_local_vecs) = wasm_u32(input)?;
                let parse_local_vecs = many_n(num_local_vecs as usize, |input| {
                    let (input, num_locals) = wasm_u32(input)?;
                    let (input, local_type) = take(1 as usize)(input)?;
                    Ok((
                        input,
                        LocalCount {
                            count: num_locals,
                            value_type: local_type[0].try_into()?,
                        },
                    ))
                });
                let (input, local_vectors) = parse_local_vecs(input)?;

                let (input, instructions) = wasm_expression(input)?;
                Ok((
                    input,
                    CodeBlock {
                        locals: local_vectors,
                        instructions,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Code(CodeSection { code_blocks: items })))
        }
        SECTION_IMPORT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, module_name) = wasm_string(input)?;
                let (input, name) = wasm_string(input)?;
                let (input, import_type) = take(1)(input)?;
                match import_type[0] {
                    DESC_FUNCTION => {
                        let (input, type_index) = wasm_u32(input)?;
                        Ok((
                            input,
                            WasmImportView::Function(FunctionImportView {
                                module_name,
                                name,
                                type_index: type_index as usize,
                            }),
                        ))
                    }
                    DESC_MEMORY => {
                        let (input, min_pages, max_pages) = wasm_limit(input)?;
                        Ok((
                            input,
                            WasmImportView::Memory(MemoryImportView {
                                module_name,
                                name,
                                min_pages,
                                max_pages,
                            }),
                        ))
                    }
                    DESC_TABLE => {
                        let (input, element_type) = take(1)(input)?;
                        let (input, min, max) = wasm_limit(input)?;
                        Ok((
                            input,
                            WasmImportView::Table(TableImportView {
                                module_name,
                                name,
                                element_type: element_type[0],
                                min,
                                max,
                            }),
                        ))
                    }
                    DESC_GLOBAL => {
                        let (input, value_type, is_mutable) = wasm_global_type(input)?;
                        Ok((
                            input,
                            WasmImportView::Global(GlobalImportView {
                                module_name,
                                name,
                                value_type,
                                is_mutable,
                            }),
                        ))
                    }
                    _ => Err("unknown export"),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Import(ImportSectionView { imports: items }),
            ))
        }
        SECTION_GLOBAL => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, value_type, is_mutable) = wasm_global_type(input)?;
                let (input, expression) = wasm_expression(input)?;
                Ok((
                    input,
                    Global {
                        value_type,
                        is_mutable,
                        value_expression: expression,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Global(GlobalSection { globals: items })))
        }
        SECTION_CUSTOM => {
            let mut name_bytes_length = 0;
            let (num_chars, byte_count) = match input.try_extract_u32(0) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };
            let (input, _) = take(byte_count as usize)(input)?;
            let (input, chars) = take(num_chars as usize)(input)?;
            name_bytes_length += byte_count + num_chars as usize;
            let name = match core::str::from_utf8(chars) {
                Ok(b) => b,
                Err(_) => return Err("could not parse utf8 string"),
            };
            let (input, bytes) =
                take((section_length as usize - name_bytes_length) as usize)(input)?;
            Ok((
                input,
                SectionView::Custom(CustomSectionView { name, data: bytes }),
            ))
        }
        SECTION_TABLE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, element_type) = take(1)(input)?;
                if element_type[0] == ANYFUNC {
                    let (input, min, max) = wasm_limit(input)?;
                    Ok((
                        input,
                        Table {
                            element_type: element_type[0],
                            min,
                            max,
                        },
                    ))
                } else {
                    Err("unknown table type")
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Table(TableSection { tables: items })))
        }
        SECTION_DATA => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, mem_index) = wasm_u32(input)?;
                let (input, offset_expression) = wasm_expression(input)?;
                let (input, data_len) = wasm_u32(input)?;
                let (input, data) = take(data_len as usize)(input)?;
                Ok((
                    input,
                    DataBlockView {
                        memory: mem_index as usize,
                        offset_expression,
                        data,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Data(DataSectionView { data_blocks: items }),
            ))
        }
        SECTION_MEMORY => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, min, max) = wasm_limit(input)?;
                Ok((
                    input,
                    WasmMemory {
                        min_pages: min,
                        max_pages: max,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Memory(MemorySection { memories: items }),
            ))
        }
        SECTION_ELEMENT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, table) = wasm_u32(input)?;
                let (input, expression) = wasm_expression(input)?;
                let (input, num_functions) = wasm_u32(input)?;
                let parse_functions = many_n(num_functions as usize, |input| {
                    let (input, i) = wasm_u32(input)?;
                    Ok((input, i as usize))
                });
                let (input, functions) = parse_functions(input)?;
                Ok((
                    input,
                    WasmElement {
                        table: table as usize,
                        value_expression: expression,
                        functions,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Element(ElementSection { elements: items }),
            ))
        }
        _ => Err("unknow section"),
    }
}

pub fn wasm_module(input: &[u8]) -> Result<ProgramView, &'static str> {
    let (input, _) = tag(MAGIC_NUMBER)(input)?;
    let (input, _) = tag(VERSION_1)(input)?;
    let mut sections = vec![];
    let mut ip = input;
    let mut p = ProgramView { sections: vec![] };
    loop {
        match section(ip) {
            Ok((input, item)) => {
                sections.push(item);
                ip = input;
            }
            Err(e) => {
                if ip.is_empty() {
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }
    p.sections = sections;
    Ok(p)
}

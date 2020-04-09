use std::{env, error::Error, fs, process::exit};
use watson::*;
use webassembly::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        let buffer = fs::read(&args[1])?;
        let s = std::str::from_utf8(&buffer)?;
        let mut p = Program::new();
        let import_output_byte_idx = p.create_import("output_byte", &[ValueType::I32], &[])?;
        let import_input_byte_idx = p.create_import("input_byte", &[], &[ValueType::I32])?;
        p.create_memory("memory", 2, None)?;
        let (main_code, _) = p.create_export("main", &[], &[])?;
        main_code.locals.push(LocalCount {
            count: 1,
            value_type: ValueType::I32,
        });
        let mut bracket_check = 0;
        let ops = &mut main_code.instructions;
        for c in s.chars() {
            match c {
                '>' => {
                    //	++ptr;
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Const(1));
                    ops.push(Instruction::I32Add);
                    ops.push(Instruction::LocalSet(0));
                }
                '<' => {
                    //	--ptr;
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Const(1));
                    ops.push(Instruction::I32Sub);
                    ops.push(Instruction::LocalSet(0));
                }
                '+' => {
                    // ++*ptr;;
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Load(0, 0));
                    ops.push(Instruction::I32Const(1));
                    ops.push(Instruction::I32Add);
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Store(0, 0));
                }
                '-' => {
                    //	--*ptr;
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Load(0, 0));
                    ops.push(Instruction::I32Const(1));
                    ops.push(Instruction::I32Sub);
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Store(0, 0));
                }
                '.' => {
                    //	putchar(*ptr);
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Load(0, 0));
                    ops.push(Instruction::Call(import_output_byte_idx));
                }
                ',' => {
                    //	*ptr=getchar();
                    ops.push(Instruction::Call(import_input_byte_idx));
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Store(0, 0));
                }
                '[' => {
                    //while (*ptr) {
                    ops.push(Instruction::Raw(LOOP));
                    ops.push(Instruction::Raw(EMPTY));
                    ops.push(Instruction::LocalGet(0));
                    ops.push(Instruction::I32Load(0, 0));
                    ops.push(Instruction::BrIf(0));
                    bracket_check += 1;
                }
                ']' => {
                    // }
                    ops.push(Instruction::Raw(END));
                    bracket_check -= 1;
                }
                _ => (),
            }
        }
        if bracket_check != 0 {
            eprintln!("invalid program, brackets don't match count.");
            exit(1);
        }
        fs::write(&args[2], &p.compile())?;
    } else {
        println!("bf <input.bf> <output.wasm>")
    }
    Ok(())
}

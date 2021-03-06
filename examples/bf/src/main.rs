use std::{env, error::Error, fs, process::exit};
use watson::*;
use webassembly::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        let buffer = fs::read(&args[1])?;
        let s = std::str::from_utf8(&buffer)?;

        let mut bracket_check = 0; // used to make sure [ and ] are balanced

        let mut p = Program::new();
        let fn_output_byte = p.create_import("output_byte", &[ValueType::I32], &[])?;
        let fn_input_byte = p.create_import("input_byte", &[], &[ValueType::I32])?;
        p.create_memory("memory", 32, None)?;
        let (main_code, _) = p.create_export("main", &[], &[])?;

        main_code.locals.push(LocalCount {
            count: 1,
            value_type: ValueType::I32,
        });

        for c in s.chars() {
            match c {
                //	++ptr/--ptr
                x @ '>' | x @ '<' => main_code.instructions.extend_from_slice(&[
                    Instruction::LocalGet(0),
                    Instruction::I32Const(4),
                    if x == '>' {
                        Instruction::I32Add
                    } else {
                        Instruction::I32Sub
                    },
                    Instruction::LocalSet(0),
                ]),
                // ++*ptr/--*ptr
                x @ '+' | x @ '-' => main_code.instructions.extend_from_slice(&[
                    Instruction::LocalGet(0),
                    Instruction::LocalGet(0),
                    Instruction::I32Load(2, 0),
                    Instruction::I32Const(1),
                    if x == '+' {
                        Instruction::I32Add
                    } else {
                        Instruction::I32Sub
                    },
                    Instruction::I32Store(2, 0),
                ]),
                //	putchar(*ptr)
                '.' => main_code.instructions.extend_from_slice(&[
                    Instruction::LocalGet(0),
                    Instruction::I32Load(2, 0),
                    Instruction::Call(fn_output_byte),
                ]),
                //	*ptr=getchar()
                ',' => main_code.instructions.extend_from_slice(&[
                    Instruction::LocalGet(0),
                    Instruction::Call(fn_input_byte),
                    Instruction::I32Store(2, 0),
                ]),
                //while (*ptr) {
                '[' => {
                    main_code.instructions.extend_from_slice(&[
                        Instruction::Raw(BLOCK),
                        Instruction::Raw(EMPTY),
                        Instruction::Raw(LOOP),
                        Instruction::Raw(EMPTY),
                        Instruction::LocalGet(0),
                        Instruction::I32Load(2, 0),
                        Instruction::I32Eqz,
                        Instruction::BrIf(1),
                    ]);
                    bracket_check += 1;
                }
                // }
                ']' => {
                    main_code.instructions.extend_from_slice(&[
                        Instruction::Br(0),
                        Instruction::Raw(END),
                        Instruction::Raw(END),
                    ]);
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

use std::str::from_utf8;
use std::{env, error::Error, fs, process::exit};
use watson::*;

fn run(program: impl InterpretableProgram) -> Result<(), &'static str> {
    let mut interpreter = Interpreter::new(program);
    interpreter.call("main", &[]);
    loop {
        let execution_unit = interpreter.next();
        let response = match execution_unit {
            // if an import is called, figure out what to do
            ExecutionUnit::CallImport(x) => {
                if x.name == "print" {
                    let start = x.params[0].to_i32() as usize;
                    let mem = interpreter.memory();
                    let mut chars = vec![];
                    let mut i = 0;
                    loop {
                        if mem[i] == 0 {
                            break;
                        }
                        chars.push(mem[start + i]);
                        i += 1;
                    }
                    let text = from_utf8(&chars).unwrap();
                    println!("{}", text);
                }
                // handle a call to an import
                ExecutionResponse {}
            }
            // if there's nothing left to do, break out of loop
            ExecutionUnit::Complete => break,
            // handle default
            mut x @ _ => x.evaluate(),
        };
        interpreter.execute(response);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let buffer = fs::read(&args[1])?;
        let program = watson::parse(&buffer)?;
        run(program)?;
    } else {
        eprintln!("bfi <app.wasm>");
        exit(1);
    }
    Ok(())
}
use async_std::task;
use std::str::from_utf8;
use std::time::Duration;
use std::{env, error::Error, fs, process::exit};
use watson::*;

async fn run(program: impl InterpretableProgram) -> Result<Vec<WasmValue>, &'static str> {
    let mut interpreter = Interpreter::new(program);
    interpreter.call("main", &[])?;
    loop {
        let execution_unit = interpreter.next()?;
        let response = match execution_unit {
            // if an import is called, figure out what to do
            ExecutionUnit::CallImport(x) => {
                if x.name == "print" {
                    let start = x.params[0].to_i32() as usize;
                    let mem = match interpreter.memory() {
                        Some(m) => m,
                        None => return Err("there should be memory"),
                    };
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
                    ExecutionResponse::DoNothing
                } else if x.name == "sleep" {
                    let start = x.params[0].to_i32() as usize;
                    task::sleep(Duration::from_secs(1)).await;
                    ExecutionResponse::DoNothing
                } else {
                    panic!("unknown import call")
                }
            }
            // if there's nothing left to do, break out of loop
            ExecutionUnit::Complete(v) => break Ok(v),
            // handle default
            mut x @ _ => x.evaluate()?,
        };
        interpreter.execute(response)?;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let buffer = fs::read(&args[1])?;
        let program = watson::parse(&buffer)?;
        task::block_on(run(program))?;
    } else {
        eprintln!("sleepyprint <app.wasm>");
        exit(1);
    }
    Ok(())
}

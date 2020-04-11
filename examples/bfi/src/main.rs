use std::{env, error::Error, fs, process::exit};
use watson::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let buffer = fs::read(&args[1])?;
        let program = watson::parse(&buffer)?;
        let mut interpreter = Interpreter::new(program);
        interpreter.call("main", &[]);
        loop {
            let mut executionUnit = interpreter.next();
            let response = match executionUnit {
                // if an import is called, figure out what to do
                ExecutionUnit::CallImport(x) => {
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
    } else {
        eprintln!("bfi <app.wasm>");
        exit(1);
    }
    Ok(())
}

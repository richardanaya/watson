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
            match executionUnit {
                ExecutionUnit::Complete => break,
            }
            let response = executionUnit.evaluate();
            interpreter.execute(response);
        }
    } else {
        eprintln!("bfi <app.wasm>");
        exit(1);
    }
    Ok(())
}

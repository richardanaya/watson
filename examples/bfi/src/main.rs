use std::str::from_utf8;
use std::{env, error::Error, fs, process::exit};
use watson::*;

fn run(program: impl InterpretableProgram) -> Result<(), &'static str> {
    let mut interpreter = Interpreter::new(program);
    interpreter.call("main", &[]);
    loop {
        let execution_unit = interpreter.next();
        let response: ExecutionResponse = match execution_unit {
            // if an import is called, figure out what to do
            ExecutionUnit::CallImport(x) => {
                if x.name == "output_byte" {
                    let char_code = [x.params[0].to_i32() as u8];
                    let text = from_utf8(&char_code).unwrap();
                    print!("{}", text);
                    ExecutionResponse::DoNothing
                } else if x.name == "input_byte" {
                    ExecutionResponse::Values(vec![42.to_wasm_value()])
                } else {
                    panic!("not sure what to do")
                }
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

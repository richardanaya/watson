# watson

<a href="https://docs.rs/watson"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>

a hyper minimalistic `no_std` + `alloc` web assembly parser/compiler for Rust based off the [official specification](https://webassembly.github.io/spec/core/index.html)

- [X] supports all section types
- [X] helper functions for finding things
- [x] support for compilation to wasm
- [ ] interpreter
- [ ] lofi wasm mode (i.e. i32 only)

```rust
[dependencies]
watson = "0.9"
```

# Parse a web assembly module

```rust
use  watson::*;

let program = watons::parse(&bytes_of_wasm)?;
for s in program.sections.iter() {
   match s {
      CodeSection(code)=> ...,
      ...
   }
}
...
```

# Write an interpreter

**this is in progress**

```rust
async fn run(program: impl InterpretableProgram) -> Result<Vec<WasmValue>, &'static str> {
    let mut interpreter = Interpreter::new(program)?;
    let mut executor = interpreter.call("main", &[])?;
    loop {
        let execution_unit = executor.next()?;
        let response = match execution_unit {
            // if an import is called, figure out what to do
            ExecutionUnit::CallImport(x) => {
                if x.name == "print" {
                    let start = x.params[0].to_i32() as usize;
                    let mem = match executor.memory() {
                        Some(m) => m,
                        None => return Err("there should be memory"),
                    };
                    let mem = mem.borrow();
                    let mut chars = vec![];
                    let mut i = 0;
                    loop {
                        if mem[start + i] == 0 {
                            break;
                        }
                        chars.push(mem[start + i]);
                        i += 1;
                    }
                    let text = from_utf8(&chars).unwrap();
                    println!("{}", text);
                    ExecutionResponse::DoNothing
                } else if x.name == "sleep" {
                    let millis = x.params[0].to_i32();
                    task::sleep(Duration::from_millis(millis as u64)).await;
                    ExecutionResponse::DoNothing
                } else {
                    panic!("unknown import call")
                }
            }
            // if there's nothing left to do, break out of loop
            ExecutionUnit::Complete(v) => break Ok(v),
            // handle other execution units with default behavior
            mut x @ _ => x.evaluate()?,
        };
        executor.execute(response)?;
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
```

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `watson` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

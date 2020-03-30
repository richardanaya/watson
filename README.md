# watson

<a href="https://docs.rs/watson"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>

a hyper minimalistic `no_std` + `alloc` web assembly parser for Rust.

**coverage is currently in progress**


- [ ] custom section
- [x] type section
- [ ] import section
- [X] function section
- [ ] table section
- [X] memory section
- [ ] global section
- [X] export section
- [x] start section
- [ ] element section
- [X] code section
- [ ] data section

```rust
[dependencies]
watson = "0"
```

# Usage

```rust
use  watson::*;

let program = Program.parse(&bytes_of_wasm)?;
for s in program.sections {
   match s {
      CodeSection(code)=> ...,
      ...
   }
}
...
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

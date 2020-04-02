# watson

<a href="https://docs.rs/watson"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>

a hyper minimalistic `no_std` + `alloc` web assembly parser for Rust based off the [official specification](https://webassembly.github.io/spec/core/index.html)

- [X] single file ~1500 lines of code (mostly enums)
- [X] supports all section types
- [X] helper functions for finding things

```rust
[dependencies]
watson = "0.2"
```

# Usage

```rust
use  watson::*;

let program = Program.parse(&bytes_of_wasm)?;
for s in program.sections.iter() {
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

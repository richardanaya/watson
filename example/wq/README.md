# wq

A simple cli tool for turning web assembly into json. This is meant to be used with other tools like [`jq`](https://stedolan.github.io/jq/)

```
cargo install wq
```
# Usage

```bash
# basic print
wq test.wasm 
# write to file
wq test.wasm test.json 
# for pipe chaining
cat simplest.wasm | wq 
```
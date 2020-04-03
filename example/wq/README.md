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
```json
"sections":[{"type":"Type","content":{"types":[{"type":"Function","content":{"inputs":[],"outputs":["I32"]}}]}},{"type":"Function","content":{"function_types":[0]}},{"type":"Memory","content":{"memories":[{"min_pages":2,"max_pages":10}]}},{"type":"Export","content":{"exports":[{"type":"Function","content":{"name":"main","index":0}},{"type":"Memory","content":{"name":"memory","index":0}}]}},{"type":"Code","content":{"code_blocks":[{"locals":[],"code_expression":[{"op":"I32Const","params":42}]}]}}]}
```
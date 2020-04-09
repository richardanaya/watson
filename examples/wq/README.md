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
Getting pretty formated
```bash
cat simplest.wasm | wq | jq
```
```js
{
  "sections": [
    {
      "section_type": "type",
      "content": {
        "types": [
          {
            "value_type": "function",
            "content": {
              "inputs": [],
              "outputs": [
                "I32"
              ]
            }
          }
        ]
      }
    },
    {
      "section_type": "function",
      "content": {
        "function_types": [
          0
        ]
      }
    },
    {
      "section_type": "memory",
      "content": {
        "memories": [
          {
            "min_pages": 2,
            "max_pages": 10
          }
        ]
      }
    },
    {
      "section_type": "export",
      "content": {
        "exports": [
          {
            "export_type": "function",
            "content": {
              "name": "main",
              "index": 0
            }
          },
          {
            "export_type": "memory",
            "content": {
              "name": "memory",
              "index": 0
            }
          }
        ]
      }
    },
    {
      "section_type": "code",
      "content": {
        "code_blocks": [
          {
            "locals": [],
            "code_expression": [
              {
                "op": "I32Const",
                "params": 42
              }
            ]
          }
        ]
      }
    }
  ]
}
```

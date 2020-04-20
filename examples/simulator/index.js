import {html, render} from 'https://unpkg.com/lit-html?module';


const utf8dec = new TextDecoder("utf-8");
var interpreter;
var memory;
var code;
var interpreter_state; 

async function loadBytes(url) {
var response = await fetch(url);
var data = await response.arrayBuffer();
return data;
}

function get_string(mem,msgPtr){
    let memory = new Uint8Array(mem);
    const str = [];
    let i = msgPtr;
    while (memory[i] !== 0) {
        str.push(memory[i]);
        i++;
    }
    return utf8dec.decode(new Uint8Array(str));
}

async function loadWebAssembly(url) {
let bytes = await loadBytes(url);
let program = await WebAssembly.instantiate(bytes, {"env":{
    _log:function(msgPtr){
        let s = get_string(program.instance.exports.memory.buffer,msgPtr);
        document.querySelector("#log").innerHTML = s;
    }
}})
return program.instance.exports;
}

async function loadAndRun(path) {
    // lets get things loaded
    interpreter = await loadWebAssembly("simulator.wasm");
    let simpleProgramBytes = await loadBytes(path);
    // create views of our data as bytes
    memory = new Uint8Array(interpreter.memory.buffer);
    // allocate space in our interpreter for the program
    let bytesToCopy = new Uint8Array(simpleProgramBytes);
    let ptr = interpreter.malloc(bytesToCopy.length); 
    // copy the bytes of our program into interpreter memory
    memory = new Uint8Array(interpreter.memory.buffer);
    memory.set(bytesToCopy, ptr);
    // signal our interpreter to run given the location 
    // and length of our web assembly program we copied over
    let result = interpreter.load(ptr,bytesToCopy.length);  
    
    document.querySelector(".loader").classList.add("hidden");
    document.querySelector(".interpreter").classList.remove("hidden");
    document.querySelector(".section_code").classList.remove("hidden");


    document.querySelector("#next").addEventListener("click", function(){
        interpreter.next_instruction();
        showCurrentState();
        showCode();
    });

    showCurrentState();
    showCode();
}

function showCode() {
    if(!code){
        let msgPtr = interpreter.get_program();
        let str = get_string(interpreter.memory.buffer,msgPtr,msgPtr);
        let data = JSON.parse(str);
        let exports = data.sections.find(x=>x.section_type == "Export");
        code = data.sections.find(x=>x.section_type == "Code").content.code_blocks.map((x,i)=>{
            return {
                instructions:x.instructions,
                locals:x.locals,
                name: exports?exports.content.exports.filter(x=>x.Function).filter(x=>x.Function.index==i).map(x=>x.Function.name)[0]:undefined,
                }
        });
    }
    let section = document.querySelector(".section_code");
    render(
        code.map((f,i)=>html`
          <div class="function">
                <b>${f.name?f.name:"function "+i}:</b>
                <div class="instructions">
                   ${f.instructions.map((x,j)=>{
                       return html`<div class="instruction ${(interpreter_state.current_position[1] == i && interpreter_state.current_position[2] == j)?"selected":null}">${x.op} ${x.params.toString()}</div>`
                   })}
                </div>
            </div>
        `),
        section
      );
}

function showCurrentState() {
    let msgPtr = interpreter.get_interpreter();
    let str = get_string(interpreter.memory.buffer,msgPtr,msgPtr);
    interpreter_state = JSON.parse(str);
}

document.querySelector("#run").addEventListener("click", function(){
    loadAndRun(document.querySelector("#wasmpath").value).then();
})
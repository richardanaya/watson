use crate::core::*;
use alloc::vec::Vec;
use webassembly::*;

impl WasmCompiler for Program {
    fn compile(&self) -> Vec<u8> {
        let mut program_bytes = vec![];
        program_bytes.extend(MAGIC_NUMBER);
        program_bytes.extend(VERSION_1);
        for s in self.sections.iter() {
            match s {
                Section::Type(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.types.len().to_wasm_bytes());
                    for t in s.types.iter() {
                        sec_data.push(FUNC);
                        sec_data.extend(t.inputs.len().to_wasm_bytes());
                        for i in t.inputs.iter() {
                            sec_data.push(i.into_wasm_byte());
                        }
                        sec_data.extend(t.outputs.len().to_wasm_bytes());
                        for i in t.outputs.iter() {
                            sec_data.push(i.into_wasm_byte());
                        }
                    }
                    program_bytes.push(SECTION_TYPE);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Function(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.function_types.len().to_wasm_bytes());
                    for f in s.function_types.iter() {
                        sec_data.extend(f.to_wasm_bytes());
                    }
                    program_bytes.push(SECTION_FUNCTION);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Code(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.code_blocks.len().to_wasm_bytes());
                    for c in s.code_blocks.iter() {
                        let mut code = vec![];
                        code.extend(c.locals.len().to_wasm_bytes());
                        for l in c.locals.iter() {
                            code.extend(l.count.to_wasm_bytes());
                            code.push(l.value_type.into_wasm_byte());
                        }
                        for i in c.code_expression.iter() {
                            i.extend_wasm_bytes(&mut code);
                        }
                        code.push(END);
                        sec_data.extend(code.len().to_wasm_bytes());
                        sec_data.extend(&code);
                    }
                    program_bytes.push(SECTION_CODE);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Export(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.exports.len().to_wasm_bytes());
                    for i in s.exports.iter() {
                        match i {
                            WasmExport::Function(f) => {
                                sec_data.extend(f.name.len().to_wasm_bytes());
                                sec_data.extend(f.name.as_bytes());
                                sec_data.push(DESC_FUNCTION);
                                sec_data.extend(f.index.to_wasm_bytes());
                            }
                            WasmExport::Global(g) => {
                                sec_data.extend(g.name.len().to_wasm_bytes());
                                sec_data.extend(g.name.as_bytes());
                                sec_data.push(DESC_GLOBAL);
                                sec_data.extend(g.index.to_wasm_bytes());
                            }
                            WasmExport::Table(t) => {
                                sec_data.extend(t.name.len().to_wasm_bytes());
                                sec_data.extend(t.name.as_bytes());
                                sec_data.push(DESC_TABLE);
                                sec_data.extend(t.index.to_wasm_bytes());
                            }
                            WasmExport::Memory(m) => {
                                sec_data.extend(m.name.len().to_wasm_bytes());
                                sec_data.extend(m.name.as_bytes());
                                sec_data.push(DESC_MEMORY);
                                sec_data.extend(m.index.to_wasm_bytes());
                            }
                        }
                    }
                    program_bytes.push(SECTION_EXPORT);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Import(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.imports.len().to_wasm_bytes());
                    for i in s.imports.iter() {
                        match i {
                            WasmImport::Function(f) => {
                                sec_data.extend(f.module_name.len().to_wasm_bytes());
                                sec_data.extend(f.module_name.as_bytes());
                                sec_data.extend(f.name.len().to_wasm_bytes());
                                sec_data.extend(f.name.as_bytes());
                                sec_data.push(DESC_FUNCTION);
                                sec_data.extend(f.type_index.to_wasm_bytes());
                            }
                            WasmImport::Global(g) => {
                                sec_data.extend(g.module_name.len().to_wasm_bytes());
                                sec_data.extend(g.module_name.as_bytes());
                                sec_data.extend(g.name.len().to_wasm_bytes());
                                sec_data.extend(g.name.as_bytes());
                                sec_data.push(DESC_GLOBAL);
                                sec_data.push(g.value_type.into_wasm_byte());
                                if g.is_mutable {
                                    sec_data.push(MUTABLE);
                                } else {
                                    sec_data.push(IMMUTABLE);
                                }
                            }
                            WasmImport::Table(t) => {
                                sec_data.extend(t.module_name.len().to_wasm_bytes());
                                sec_data.extend(t.module_name.as_bytes());
                                sec_data.extend(t.name.len().to_wasm_bytes());
                                sec_data.extend(t.name.as_bytes());
                                sec_data.push(DESC_TABLE);
                                sec_data.push(t.element_type);
                                if t.max.is_some() {
                                    sec_data.push(LIMIT_MIN_MAX);
                                    sec_data.extend(t.min.to_wasm_bytes());
                                    sec_data.extend(t.max.unwrap().to_wasm_bytes());
                                } else {
                                    sec_data.push(LIMIT_MIN);
                                    sec_data.extend(t.min.to_wasm_bytes());
                                }
                            }
                            WasmImport::Memory(m) => {
                                sec_data.extend(m.module_name.len().to_wasm_bytes());
                                sec_data.extend(m.module_name.as_bytes());
                                sec_data.extend(m.name.len().to_wasm_bytes());
                                sec_data.extend(m.name.as_bytes());
                                sec_data.push(DESC_MEMORY);
                                if m.max_pages.is_some() {
                                    sec_data.push(LIMIT_MIN_MAX);
                                    sec_data.extend(m.min_pages.to_wasm_bytes());
                                    sec_data.extend(m.max_pages.unwrap().to_wasm_bytes());
                                } else {
                                    sec_data.push(LIMIT_MIN);
                                    sec_data.extend(m.min_pages.to_wasm_bytes());
                                }
                            }
                        }
                    }
                    program_bytes.push(SECTION_IMPORT);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Memory(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.memories.len().to_wasm_bytes());
                    for m in s.memories.iter() {
                        if m.max_pages.is_some() {
                            sec_data.push(LIMIT_MIN_MAX);
                            sec_data.extend(m.min_pages.to_wasm_bytes());
                            sec_data.extend(m.max_pages.unwrap().to_wasm_bytes());
                        } else {
                            sec_data.push(LIMIT_MIN);
                            sec_data.extend(m.min_pages.to_wasm_bytes());
                        }
                    }
                    program_bytes.push(SECTION_MEMORY);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Start(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.start_function.to_wasm_bytes());
                    program_bytes.push(SECTION_START);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Global(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.globals.len().to_wasm_bytes());
                    for g in s.globals.iter() {
                        sec_data.push(g.value_type.into_wasm_byte());
                        if g.is_mutable {
                            sec_data.push(MUTABLE);
                        } else {
                            sec_data.push(IMMUTABLE);
                        }
                        for i in g.value_expression.iter() {
                            i.extend_wasm_bytes(&mut sec_data);
                        }
                        sec_data.push(END);
                    }
                    program_bytes.push(SECTION_GLOBAL);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Table(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.tables.len().to_wasm_bytes());
                    for t in s.tables.iter() {
                        sec_data.push(ANYFUNC);
                        if t.max.is_some() {
                            sec_data.push(LIMIT_MIN_MAX);
                            sec_data.extend(t.min.to_wasm_bytes());
                            sec_data.extend(t.max.unwrap().to_wasm_bytes());
                        } else {
                            sec_data.push(LIMIT_MIN);
                            sec_data.extend(t.min.to_wasm_bytes());
                        }
                    }
                    program_bytes.push(SECTION_TABLE);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Data(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.data_blocks.len().to_wasm_bytes());
                    for d in s.data_blocks.iter() {
                        sec_data.extend(d.memory.to_wasm_bytes());
                        for i in d.offset_expression.iter() {
                            i.extend_wasm_bytes(&mut sec_data);
                        }
                        sec_data.push(END);
                        sec_data.extend(d.data.len().to_wasm_bytes());
                        sec_data.extend(&d.data);
                    }
                    program_bytes.push(SECTION_DATA);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Custom(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.name.len().to_wasm_bytes());
                    sec_data.extend(s.name.as_bytes());
                    sec_data.extend(&s.data);
                    program_bytes.push(SECTION_CUSTOM);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
                Section::Element(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.elements.len().to_wasm_bytes());
                    for e in s.elements.iter() {
                        sec_data.extend(e.table.to_wasm_bytes());
                        for i in e.value_expression.iter() {
                            i.extend_wasm_bytes(&mut sec_data);
                        }
                        sec_data.push(END);
                        sec_data.extend(e.functions.len().to_wasm_bytes());
                        for f in e.functions.iter() {
                            sec_data.extend(f.to_wasm_bytes());
                        }
                    }
                    program_bytes.push(SECTION_ELEMENT);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                }
            }
        }
        program_bytes
    }
}

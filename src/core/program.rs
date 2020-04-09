use super::common::*;
use super::view::*;
use crate::alloc::string::ToString;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ProgramView<'a> {
    #[serde(borrow)]
    pub sections: Vec<SectionView<'a>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Program {
    pub sections: Vec<Section>,
}

impl<'p> ProgramView<'p> {
    pub fn find_exported_function<'a>(
        &'a self,
        name: &str,
    ) -> Result<&'a ExportView, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let SectionView::Export(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(SectionView::Export(export_section)) = result {
            let result = self.sections.iter().find(|x| {
                if let SectionView::Code(_) = x {
                    true
                } else {
                    false
                }
            });
            if let Some(SectionView::Code(_)) = result {
                let result = export_section.exports.iter().find(|x| {
                    if let WasmExportView::Function(f) = x {
                        f.name == name
                    } else {
                        false
                    }
                });
                let main_export = match result {
                    Some(WasmExportView::Function(f)) => f,
                    _ => {
                        let e = "could not find export";
                        return Err(e);
                    }
                };
                Ok(main_export)
            } else {
                Err("could not find code section")
            }
        } else {
            Err("could not find export section")
        }
    }

    pub fn find_code_block<'a>(&'a self, index: usize) -> Result<&'a CodeBlock, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let SectionView::Code(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(SectionView::Code(code_section)) = result {
            if index >= code_section.code_blocks.len() {
                Err("invalid code block index")
            } else {
                Ok(&code_section.code_blocks[index])
            }
        } else {
            Err("could find code section")
        }
    }

    pub fn to_owned(&self) -> Program {
        Program {
            sections: self
                .sections
                .iter()
                .map(|x| x.to_owned())
                .collect::<Vec<Section>>(),
        }
    }
}

impl Default for Program {
    fn default() -> Program {
        Program {
            sections: Vec::new(),
        }
    }
}

impl Program {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_exported_function<'a>(&'a self, name: &str) -> Result<&'a Export, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let Section::Export(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(Section::Export(export_section)) = result {
            let result = self.sections.iter().find(|x| {
                if let Section::Code(_) = x {
                    true
                } else {
                    false
                }
            });
            if let Some(Section::Code(_)) = result {
                let result = export_section.exports.iter().find(|x| {
                    if let WasmExport::Function(f) = x {
                        f.name == name
                    } else {
                        false
                    }
                });
                let main_export = match result {
                    Some(WasmExport::Function(f)) => f,
                    _ => {
                        let e = "could not find export";
                        return Err(e);
                    }
                };
                Ok(main_export)
            } else {
                Err("could not find code section")
            }
        } else {
            Err("could not find export section")
        }
    }

    pub fn find_code_block<'a>(&'a self, index: usize) -> Result<&'a CodeBlock, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let Section::Code(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(Section::Code(code_section)) = result {
            if index >= code_section.code_blocks.len() {
                Err("invalid code block index")
            } else {
                Ok(&code_section.code_blocks[index])
            }
        } else {
            Err("could find code section")
        }
    }

    pub fn create_import<'a>(
        &'a mut self,
        name: &str,
        inputs: &[ValueType],
        outputs: &[ValueType],
    ) -> Result<usize, &'static str> {
        let type_section = match self
            .sections
            .iter_mut()
            .find(|x| matches!(x, Section::Type(_)))
        {
            Some(x) => x,
            None => {
                self.sections
                    .push(Section::Type(TypeSection { types: Vec::new() }));
                let len = self.sections.len() - 1;
                &mut self.sections[len]
            }
        };

        let type_index = if let Section::Type(s) = type_section {
            match s
                .types
                .iter()
                .enumerate()
                .find(|x| x.1.inputs == inputs && x.1.outputs == outputs)
            {
                Some(x) => x.0,
                None => {
                    s.types.push(FunctionType {
                        inputs: inputs.to_vec(),
                        outputs: outputs.to_vec(),
                    });
                    s.types.len() - 1
                }
            }
        } else {
            unreachable!()
        };

        let imports_section = match self
            .sections
            .iter_mut()
            .find(|x| matches!(x, Section::Import(_)))
        {
            Some(x) => x,
            None => {
                self.sections.push(Section::Import(ImportSection {
                    imports: Vec::new(),
                }));
                let len = self.sections.len() - 1;
                &mut self.sections[len]
            }
        };

        if let Section::Import(s) = imports_section {
            s.imports.push(WasmImport::Function(FunctionImport {
                module_name: "env".to_string(),
                name: name.to_string(),
                type_index,
            }));
            Ok(s.imports.len() - 1)
        } else {
            unreachable!()
        }
    }

    pub fn create_export<'a>(
        &'a mut self,
        name: &str,
        inputs: &[ValueType],
        outputs: &[ValueType],
    ) -> Result<(&'a mut CodeBlock, usize), &'static str> {
        let type_section = match self
            .sections
            .iter_mut()
            .find(|x| matches!(x, Section::Type(_)))
        {
            Some(x) => x,
            None => {
                self.sections
                    .push(Section::Type(TypeSection { types: Vec::new() }));
                let len = self.sections.len() - 1;
                &mut self.sections[len]
            }
        };

        let type_index = if let Section::Type(s) = type_section {
            match s
                .types
                .iter()
                .enumerate()
                .find(|x| x.1.inputs == inputs && x.1.outputs == outputs)
            {
                Some(x) => x.0,
                None => {
                    s.types.push(FunctionType {
                        inputs: inputs.to_vec(),
                        outputs: outputs.to_vec(),
                    });
                    s.types.len() - 1
                }
            }
        } else {
            unreachable!()
        };

        let function_section = match self
            .sections
            .iter_mut()
            .find(|x| matches!(x, Section::Function(_)))
        {
            Some(x) => x,
            None => {
                self.sections.push(Section::Function(FunctionSection {
                    function_types: Vec::new(),
                }));
                let len = self.sections.len() - 1;
                &mut self.sections[len]
            }
        };

        let func_index = if let Section::Function(s) = function_section {
            s.function_types.push(type_index);
            s.function_types.len() - 1
        } else {
            unreachable!()
        };

        let exports_section = match self
            .sections
            .iter_mut()
            .find(|x| matches!(x, Section::Export(_)))
        {
            Some(x) => x,
            None => {
                self.sections.push(Section::Export(ExportSection {
                    exports: Vec::new(),
                }));
                let len = self.sections.len() - 1;
                &mut self.sections[len]
            }
        };

        if let Section::Export(s) = exports_section {
            s.exports.push(WasmExport::Function(Export {
                name: name.to_string(),
                index: func_index,
            }));
        } else {
            unreachable!()
        }

        let code_section_index = match self
            .sections
            .iter()
            .enumerate()
            .find(|x| matches!(x, (_,Section::Code(_))))
        {
            Some(x) => x.0,
            None => {
                self.sections.push(Section::Code(CodeSection {
                    code_blocks: Vec::new(),
                }));
                self.sections.len() - 1
            }
        };

        if let Section::Code(s) = &mut self.sections[code_section_index] {
            s.code_blocks.push(CodeBlock {
                locals: Vec::new(),
                instructions: Vec::new(),
            });
            let idx = s.code_blocks.len() - 1;
            Ok((&mut s.code_blocks[idx], idx))
        } else {
            unreachable!()
        }
    }

    fn ensure_memories<'a>(&'a mut self) -> (&'a mut MemorySection,usize) {
        let idx = match self
            .sections
            .iter()
            .enumerate()
            .find(|x| matches!(x, (_,Section::Memory(_))))
        {
            Some(x) => x.0,
            None => {
                self.sections.push(Section::Memory(MemorySection {
                    memories: vec![],
                }));
                self.sections.len()-1
            }
        };
        if let Section::Memory(s) = &mut self.sections[idx] {
            return (s,idx);
        } else {
            unreachable!();
        }
    }

    fn ensure_exports<'a>(&'a mut self) -> (&'a mut ExportSection,usize) {
        let idx = match self
            .sections
            .iter()
            .enumerate()
            .find(|x| matches!(x, (_,Section::Export(_))))
        {
            Some(x) => x.0,
            None => {
                self.sections.push(Section::Export(ExportSection {
                    exports: vec![],
                }));
                self.sections.len()-1
            }
        };
        if let Section::Export(s) = &mut self.sections[idx] {
            return (s,idx);
        } else {
            unreachable!();
        }
    }


    pub fn create_memory<'a>(
        &'a mut self,
        name: &str,
        min: usize,
        max: Option<usize>,
    ) -> Result<(&'a mut WasmMemory, usize), &'static str> {
        let mem_idx;
        let mem_sec_idx;
        {
            let (memory_section,idx) = self.ensure_memories();
            mem_sec_idx = idx;
            memory_section.memories.push(WasmMemory {
                min_pages:min,
                max_pages:max,
            });
            mem_idx = memory_section.memories.len()-1;
        }
        {
            let (export_section,_) = self.ensure_exports();

            export_section.exports.push(WasmExport::Memory(Export{
                name:name.to_string(),
                index:mem_idx,
            }));
        }

        if let Section::Memory(s) = &mut self.sections[mem_sec_idx]{
            Ok((&mut s.memories[mem_idx],mem_idx))
        } else {
            unreachable!();
        }
    }
}

use super::common::*;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ProgramView<'a> {
    #[serde(borrow)]
    pub sections: Vec<SectionView<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl Program {
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
}

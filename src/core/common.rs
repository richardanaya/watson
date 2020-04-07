use super::instructions::*;
use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use webassembly::*;

pub trait WasmValueTypes {
    fn try_to_value_types(self) -> Result<Vec<ValueType>, &'static str>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    pub fn into_wasm_byte(self) -> u8 {
        match self {
            ValueType::I32 => I32,
            ValueType::I64 => I64,
            ValueType::F32 => F32,
            ValueType::F64 => F64,
        }
    }
}

impl TryFrom<u8> for ValueType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            I32 => Ok(ValueType::I32),
            I64 => Ok(ValueType::I64),
            F32 => Ok(ValueType::F32),
            F64 => Ok(ValueType::F64),
            _ => Err("could not convert data type"),
        }
    }
}

impl WasmValueTypes for Vec<u8> {
    fn try_to_value_types(self) -> Result<Vec<ValueType>, &'static str> {
        let mut r = vec![];
        for v in self {
            r.push(v.try_into()?);
        }
        Ok(r)
    }
}

impl TryFrom<&u8> for ValueType {
    type Error = &'static str;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            I32 => Ok(ValueType::I32),
            I64 => Ok(ValueType::I64),
            F32 => Ok(ValueType::F32),
            F64 => Ok(ValueType::F64),
            _ => Err("could not convert data type"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionType {
    pub inputs: Vec<ValueType>,
    pub outputs: Vec<ValueType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TypeSection {
    pub types: Vec<FunctionType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionSection {
    pub function_types: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct LocalCount {
    pub count: u32,
    pub value_type: ValueType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CodeBlock {
    pub locals: Vec<LocalCount>,
    pub code_expression: Vec<Instruction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CodeSection {
    pub code_blocks: Vec<CodeBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExportView<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
//#[serde(tag = "export_type", content = "content")]
#[repr(C)]
pub enum WasmExportView<'a> {
    //#[serde(rename = "function")]
    #[serde(borrow)]
    Function(ExportView<'a>),
    //#[serde(rename = "table")]
    #[serde(borrow)]
    Table(ExportView<'a>),
    //#[serde(rename = "memory")]
    #[serde(borrow)]
    Memory(ExportView<'a>),
    //#[serde(rename = "global")]
    #[serde(borrow)]
    Global(ExportView<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExportSectionView<'a> {
    #[serde(borrow)]
    pub exports: Vec<WasmExportView<'a>>,
}

impl<'a> ExportSectionView<'a> {
    fn to_owned(&self) -> ExportSection {
        ExportSection {
            exports: self
                .exports
                .iter()
                .map(|x| match x {
                    WasmExportView::Function(x) => WasmExport::Function(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                    WasmExportView::Global(x) => WasmExport::Global(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                    WasmExportView::Memory(x) => WasmExport::Memory(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                    WasmExportView::Table(x) => WasmExport::Table(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                })
                .collect::<Vec<WasmExport>>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Export {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
//#[serde(tag = "export_type", content = "content")]
#[repr(C)]
pub enum WasmExport {
    //#[serde(rename = "function")]
    Function(Export),
    //#[serde(rename = "table")]
    Table(Export),
    //#[serde(rename = "memory")]
    Memory(Export),
    //#[serde(rename = "global")]
    Global(Export),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExportSection {
    pub exports: Vec<WasmExport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub type_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GlobalImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub value_type: ValueType,
    pub is_mutable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct MemoryImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TableImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "import_type", content = "content")]
#[repr(C)]
pub enum WasmImportView<'a> {
    //#[serde(rename = "function")]
    #[serde(borrow)]
    Function(FunctionImportView<'a>),
    //#[serde(rename = "global")]
    #[serde(borrow)]
    Global(GlobalImportView<'a>),
    //#[serde(rename = "memory")]
    #[serde(borrow)]
    Memory(MemoryImportView<'a>),
    //#[serde(rename = "table")]
    #[serde(borrow)]
    Table(TableImportView<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionImport {
    pub module_name: String,
    pub name: String,
    pub type_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GlobalImport {
    pub module_name: String,
    pub name: String,
    pub value_type: ValueType,
    pub is_mutable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct MemoryImport {
    pub module_name: String,
    pub name: String,
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TableImport {
    pub module_name: String,
    pub name: String,
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "import_type", content = "content")]
#[repr(C)]
pub enum WasmImport {
    //#[serde(rename = "function")]
    Function(FunctionImport),
    //#[serde(rename = "global")]
    Global(GlobalImport),
    //#[serde(rename = "memory")]
    Memory(MemoryImport),
    //#[serde(rename = "table")]
    Table(TableImport),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ImportSection {
    pub imports: Vec<WasmImport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ImportSectionView<'a> {
    #[serde(borrow)]
    pub imports: Vec<WasmImportView<'a>>,
}

impl<'a> ImportSectionView<'a> {
    fn to_owned(&self) -> ImportSection {
        ImportSection {
            imports: self
                .imports
                .iter()
                .map(|x| match x {
                    WasmImportView::Function(x) => WasmImport::Function(FunctionImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        type_index: x.type_index,
                    }),
                    WasmImportView::Global(x) => WasmImport::Global(GlobalImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        value_type: x.value_type,
                        is_mutable: x.is_mutable,
                    }),
                    WasmImportView::Memory(x) => WasmImport::Memory(MemoryImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        min_pages: x.min_pages,
                        max_pages: x.max_pages,
                    }),
                    WasmImportView::Table(x) => WasmImport::Table(TableImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        element_type: x.element_type,
                        min: x.min,
                        max: x.max,
                    }),
                })
                .collect::<Vec<WasmImport>>(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct WasmMemory {
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct MemorySection {
    pub memories: Vec<WasmMemory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct StartSection {
    pub start_function: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Global {
    pub value_type: ValueType,
    pub is_mutable: bool,
    pub value_expression: Vec<Instruction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GlobalSection {
    pub globals: Vec<Global>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Table {
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TableSection {
    pub tables: Vec<Table>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataBlockView<'a> {
    pub memory: usize,
    pub offset_expression: Vec<Instruction>,
    #[serde(borrow)]
    pub data: &'a [u8],
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataSectionView<'a> {
    #[serde(borrow)]
    pub data_blocks: Vec<DataBlockView<'a>>,
}

impl<'a> DataSectionView<'a> {
    fn to_owned(&self) -> DataSection {
        DataSection {
            data_blocks: self
                .data_blocks
                .iter()
                .map(|x| DataBlock {
                    memory: x.memory,
                    offset_expression: x.offset_expression.clone(),
                    data: x.data.to_vec(),
                })
                .collect::<Vec<DataBlock>>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataBlock {
    pub memory: usize,
    pub offset_expression: Vec<Instruction>,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataSection {
    pub data_blocks: Vec<DataBlock>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CustomSectionView<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    #[serde(borrow)]
    pub data: &'a [u8],
}

impl<'a> CustomSectionView<'a> {
    fn to_owned(&self) -> CustomSection {
        CustomSection {
            name: self.name.to_string(),
            data: self.data.to_vec(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CustomSection {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct WasmElement {
    pub table: usize,
    pub value_expression: Vec<Instruction>,
    pub functions: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ElementSection {
    pub elements: Vec<WasmElement>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "section_type", content = "content")]
#[repr(C)]
pub enum SectionView<'a> {
    //#[serde(rename = "type")]
    Type(TypeSection),
    //#[serde(rename = "function")]
    Function(FunctionSection),
    //#[serde(rename = "code")]
    Code(CodeSection),
    //#[serde(rename = "export")]
    #[serde(borrow)]
    Export(ExportSectionView<'a>),
    //#[serde(rename = "import")]
    #[serde(borrow)]
    Import(ImportSectionView<'a>),
    //#[serde(rename = "memory")]
    Memory(MemorySection),
    //#[serde(rename = "start")]
    Start(StartSection),
    //#[serde(rename = "global")]
    Global(GlobalSection),
    //#[serde(rename = "table")]
    Table(TableSection),
    //#[serde(rename = "data")]
    #[serde(borrow)]
    Data(DataSectionView<'a>),
    //#[serde(rename = "custom")]
    #[serde(borrow)]
    Custom(CustomSectionView<'a>),
    //#[serde(rename = "element")]
    Element(ElementSection),
}

impl<'a> SectionView<'a> {
    pub fn to_owned(&self) -> Section {
        match self {
            SectionView::Type(s) => Section::Type(s.clone()),
            SectionView::Function(s) => Section::Function(s.clone()),
            SectionView::Code(s) => Section::Code(s.clone()),
            SectionView::Export(s) => Section::Export(s.to_owned()),
            SectionView::Import(s) => Section::Import(s.to_owned()),
            SectionView::Memory(s) => Section::Memory(s.clone()),
            SectionView::Start(s) => Section::Start(s.clone()),
            SectionView::Global(s) => Section::Global(s.clone()),
            SectionView::Table(s) => Section::Table(s.clone()),
            SectionView::Data(s) => Section::Data(s.to_owned()),
            SectionView::Custom(s) => Section::Custom(s.to_owned()),
            SectionView::Element(s) => Section::Element(s.clone()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "section_type", content = "content")]
#[repr(C)]
pub enum Section {
    //#[serde(rename = "type")]
    Type(TypeSection),
    //#[serde(rename = "function")]
    Function(FunctionSection),
    //#[serde(rename = "code")]
    Code(CodeSection),
    //#[serde(rename = "export")]
    Export(ExportSection),
    //#[serde(rename = "import")]
    Import(ImportSection),
    //#[serde(rename = "memory")]
    Memory(MemorySection),
    //#[serde(rename = "start")]
    Start(StartSection),
    //#[serde(rename = "global")]
    Global(GlobalSection),
    //#[serde(rename = "table")]
    Table(TableSection),
    //#[serde(rename = "data")]
    Data(DataSection),
    //#[serde(rename = "custom")]
    Custom(CustomSection),
    //#[serde(rename = "element")]
    Element(ElementSection),
}

pub trait WasmCompiler {
    fn compile(&self) -> Vec<u8>;
}

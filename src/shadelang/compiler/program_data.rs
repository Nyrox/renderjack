use crate::shadelang::ast::TypeKind;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SymbolMeta {
    pub type_kind: TypeKind,
    pub stack_offset: Option<usize>,
    pub is_static: bool,
}

#[derive(Clone, Debug)]
pub struct FuncMeta {
    pub symbols: HashMap<String, SymbolMeta>,
    pub address: Option<usize>,
}

impl FuncMeta {
    pub fn new() -> Self {
        FuncMeta {
            symbols: HashMap::new(),
            address: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProgramData {
    pub functions: HashMap<String, FuncMeta>,
    pub global_symbols: HashMap<String, SymbolMeta>,
    pub min_stack_size: usize,
    pub static_section_size: usize,
}

impl ProgramData {
    pub fn new() -> Self {
        ProgramData {
            functions: HashMap::new(),
            global_symbols: HashMap::new(),
            min_stack_size: 0,
            static_section_size: 0,
        }
    }
}
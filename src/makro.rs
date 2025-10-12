use std::collections::HashMap;
use khi::parse::pdm::{ParsedValue};

/// Todo: Currently only support for Math type macros, but should be expanded to several types.
pub type Macros = HashMap<String, MathMacro>;

pub struct MathMacro {
    pub(crate) arity: usize,
    pub(crate) expansion: ParsedValue,
}

pub struct LocalMacroRegistry<'a> {
    project_macros: &'a Macros,
    document_macros: &'a Macros,
}

impl<'a> LocalMacroRegistry<'a> {
    pub fn new(project_macros: &'a HashMap<String, MathMacro>, document_macros: &'a HashMap<String, MathMacro>) -> Self {
        Self { project_macros, document_macros }
    }
}

pub trait MacroMap {
    fn get(&self, key: &str) -> Option<&MathMacro>;
}

impl<'a> MacroMap for LocalMacroRegistry<'a> {
    fn get(&self, key: &str) -> Option<&MathMacro> {
        if let Some(v) = self.document_macros.get(key) {
            Some(v)
        } else if let Some(v) = self.project_macros.get(key) {
            Some(v)
        } else {
            None
        }
    }
}

impl MacroMap for HashMap<String, MathMacro> {
    fn get(&self, key: &str) -> Option<&MathMacro> {
        self.get(key)
    }
}

use std::collections::HashMap;

pub type Styles = HashMap<String, Style>;

/// A class style.
pub struct Style {
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
    pub abbreviation: Option<String>,
    pub symbol_path: Option<String>, // TODO: Enable symbols
}

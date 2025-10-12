use std::collections::HashMap;
use std::path::Path;
use khi::{Dictionary, Text, Value};
use khi::parse::pdm::{ParsedDictionary};
use crate::file::{read_file_content_to_dictionary, read_file_to_string};
use crate::style::{Style, Styles};

pub fn read_style_file(path: &Path) -> Result<Styles, String> {
    let style = read_file_to_string(path, "style")?;
    let style = read_file_content_to_dictionary(path, "style", &style)?;
    read_style_dictionary(&style)
}

pub fn read_style_dictionary(style_dictionary: &ParsedDictionary) -> Result<Styles, String> {
    let mut styles = HashMap::new();
    for (class_key, class_style) in style_dictionary.iter() {
        if !class_style.is_dictionary() {
            eprintln!(r#"The type "{class_key}" at {}:{} must be a dictionary."#, class_style.from().line, class_style.from().column);
        }
        let at = class_style.from();
        let class_style = class_style.as_dictionary().unwrap();
        // Read name.
        let name = if let Some(name) = class_style.get("Name") {
            if !name.is_text() {
                return Err(format!(r#"Name in type {class_key} at {}:{} must be text."#, name.from().line, name.from().column));
            }
            name.as_text().unwrap().as_str().to_string()
        } else {
            return Err(format!(r#"The type {class_key} at {}:{} must have a Name entry."#, at.line, at.column));
        };
        // Read description.
        let description = if let Some(description) = class_style.get("Description") {
            if !description.is_text() {
                return Err(format!(r#"Description in type {class_key} at {}:{} must be text."#, description.from().line, description.from().column));
            }
            Some(description.as_text().unwrap().as_str().to_string())
        } else {
            None
        };
        let abbreviation = if let Some(abbreviation) = class_style.get("Abbreviation") {
            if !abbreviation.is_text() {
                return Err(format!("Abbreviation in type {class_key} at {}:{} must be text.", abbreviation.from().line, abbreviation.from().column));
            }
            Some(abbreviation.as_text().unwrap().as_str().to_string())
        } else {
            None
        };
        // Read colour.
        let colour = if let Some(colour) = class_style.get("Colour") {
            if !colour.is_text() {
                return Err(format!("Colour in type {class_key} at {}:{} must be text.", colour.from().line, colour.from().column));
            }
            Some(colour.as_text().unwrap().as_str().to_string())
            // TODO: Verify colour value
        } else {
            None
        };
        // let symbol = if let Some(s) = type_value.get("Symbol") { // TODO: Fix symbol path
        //     if !s.is_text() {
        //         eprintln!("Error: Symbol must be text.");
        //         continue;
        //     };
        //     let s = s.as_text().unwrap().as_str();
        //     Some(String::from(s))
        // } else {
        //     None
        // };
        let style = Style { name: name.clone(), description, colour, abbreviation, symbol_path: None };
        styles.insert(name, style);
    }
    Ok(styles)
}

use std::path::Path;
use khi::parse::pdm::{ParsedList, ParsedTupleElement};
use khi::{List, TaggedTuple, Value};
use crate::makro::{MathMacro, Macros};
use crate::file::{read_excludable_file_to_string, read_file_content_to_list};
use crate::tuple_split;

// TODO: Only math macros currently

pub fn read_macro_definition_file(macros: &mut Macros, path: &Path) -> Result<(), String> {
    let content = match read_excludable_file_to_string(path, "macro definition")? {
        None => return Ok(()),
        Some(c) => c,
    };
    let list = read_file_content_to_list(path, "macro definition", &content)?;
    read_macro_definitions_list(macros, &list)?;
    Ok(())
}

pub fn read_macro_definitions_list(macros: &mut Macros, definitions: &ParsedList) -> Result<(), String> {
    for definition in definitions.iter() {
        if !definition.is_tagged_tuple() {
            return Err(format!("Definition must be a tag."));
        }
        let definition = definition.as_tagged_tuple().unwrap();
        let typ = definition.name().unwrap();
        if typ != "Math" { // TODO: Only math currently.
            return Err(format!("Definition type must be Math."));
        }
        let (positional, named) = tuple_split(definition);
        if positional.len() != 2 {
            return Err(format!("Definition must have a signature and an expansion."));
        }
        let signature = match positional.get(0) {
            Some(&signature) => signature,
            None => return Err(format!("Definition does not take optional arguments.")),
        };
        if !signature.is_tagged_tuple() {
            return Err(format!("Signature of macro definition must be a tag."));
        }
        let signature = signature.as_tagged_tuple().unwrap();
        let name = signature.name().unwrap();
        let arity = signature.len();
        let expansion = match positional.get(1) {
            Some(&expansion) => expansion,
            None => return Err(format!("Definition does not take optional arguments.")),
        };
        if macros.contains_key(name) {
            return Err(format!("Macro with name {} is already defined.", name));
        }
        let name = name.into();
        let mcr = MathMacro {
            arity,
            expansion: expansion.clone(),
        };
        macros.insert(name, mcr);
    }
    Ok(())
}

//! Read the project file.

use khi::pdm::{ParsedDictionary, Position};
use khi::{Dictionary, List, Text, Value};
use khi::tex::{BreakMode, write_tex_with};
use crate::{tex_error_to_text};

pub fn read_project(project: &ParsedDictionary) -> Result<ReadProject, String> {
    let model = read_model(project)?;
    let resolution_paths = read_resolution(project)?;
    let preamble = read_preamble(project)?;
    Ok((model, resolution_paths, preamble))
}

// pub type ReadProject = (ReadModel, ResolutionPaths, DefinedCommands); // TODO
pub type ReadProject = (ReadModel, ResolutionPaths, Option<String>);

//// Model

/// Read model.
fn read_model(parsed_project: &ParsedDictionary) -> Result<ReadModel, String> {
    if let Some(parsed_article_types) = parsed_project.get("Types") {
        if !parsed_article_types.is_dictionary() {
            return Err(format!(r#"The "Types" section must be a dictionary."#));
        }
        let parsed_article_types = parsed_article_types.as_dictionary().unwrap();
        read_types(parsed_article_types)
    } else {
        Ok(ReadModel::new())
    }
}

fn read_types(parsed_types: &ParsedDictionary) -> Result<Vec<ReadArticleType>, String> {
    let mut read_types = Vec::new();
    for (type_key, type_value) in parsed_types.iter() {
        if !type_value.is_dictionary() {
            eprintln!(r#"The type "{type_key}" at {}:{} must be a dictionary."#, type_value.from().line, type_value.from().column);
        }
        let readed_type = type_value.as_dictionary().unwrap();
        let article_type = read_type(type_key, readed_type, type_value.from())?;
        read_types.push(article_type);
    }
    Ok(read_types)
}

fn read_type(type_key: &str, type_value: &ParsedDictionary, at: Position) -> Result<ReadArticleType, String> {
    // Read key.
    let key = if let Some(key) = type_value.get("Key") {
        if !key.is_text() {
            return Err(format!(r#"Key in type {type_key} at {}:{} must be text."#, key.from().line, key.from().column));
        }
        key.as_text().unwrap().as_str().to_string()
    } else {
        return Err(format!(r#"The type {type_key} at {}:{} must have a Key entry."#, at.line, at.column));
    };
    // Read name.
    let name = if let Some(name) = type_value.get("Name") {
        if !name.is_text() {
            return Err(format!(r#"Name in type {type_key} at {}:{} must be text."#, name.from().line, name.from().column));
        }
        name.as_text().unwrap().as_str().to_string()
    } else {
        return Err(format!(r#"The type {type_key} at {}:{} must have a Name entry."#, at.line, at.column));
    };
    // Read description.
    let description = if let Some(description) = type_value.get("Description") {
        if !description.is_text() {
            return Err(format!(r#"Description in type {type_key} at {}:{} must be text."#, description.from().line, description.from().column));
        }
        Some(description.as_text().unwrap().as_str().to_string())
    } else {
        None
    };
    let abbreviation = if let Some(abbreviation) = type_value.get("Abbreviation") {
        if !abbreviation.is_text() {
            return Err(format!("Abbreviation in type {type_key} at {}:{} must be text.", abbreviation.from().line, abbreviation.from().column));
        }
        Some(abbreviation.as_text().unwrap().as_str().to_string())
    } else {
        None
    };
    // Read colour.
    let colour = if let Some(colour) = type_value.get("Colour") {
        if !colour.is_text() {
            return Err(format!("Colour in type {type_key} at {}:{} must be text.", colour.from().line, colour.from().column));
        }
        colour.as_text().unwrap().as_str().to_string()
        // TODO: Verify colour value
    } else {
        "#636363".to_string()
    };
    // let symbol = if let Some(s) = type_value.get("Symbol") { // TODO: Fix symbol
    //     if !s.is_text() {
    //         eprintln!("Error: Symbol must be text.");
    //         continue;
    //     };
    //     let s = s.as_text().unwrap().as_str();
    //     Some(String::from(s))
    // } else {
    //     None
    // };
    let links = if let Some(links) = type_value.get("Links") {
        if !links.is_dictionary() {
            return Err(format!(r#"Links in type {type_key} at {}:{} must be a dictionary."#, links.from().line, links.from().column));
        }
        let links = links.as_dictionary().unwrap();
        read_links(links)?
    } else {
        vec![]
    };
    Ok(ReadArticleType { key, name, description, abbreviation, colour, links })
}

fn read_links(links: &ParsedDictionary) -> Result<Vec<ReadLinkType>, String> {
    let mut link_types = vec![];
    for (link_key, link_value) in links.iter() {
        if !link_value.is_dictionary() {
            return Err(format!("Error: Links value must be a dictionary."));
        }
        let link_value = link_value.as_dictionary().unwrap();
        let key = link_key.to_string();
        let origin_name = crate::read_text_value_or_err(link_value, "OriginName")?;
        let origin_description = crate::read_text_value_or_err(link_value, "OriginDescription")?;
        let target_name = crate::read_text_value_or_err(link_value, "TargetName")?;
        let target_description = crate::read_text_value_or_err(link_value, "TargetDescription")?;
        let target_show = crate::read_text_value_or_err(link_value, "TargetShow")?;
        let target_show = match target_show.as_str() {
            "True" => true,
            "False" => false,
            _ => return Err(format!("Invalid TargetShow value: {}.", &target_show)),
        };
        let link_type = ReadLinkType { key, origin_name, origin_description, target_name, target_description, target_show };
        link_types.push(link_type);
    }
    Ok(link_types)
}

pub type ReadModel = Vec<ReadArticleType>;

pub struct ReadArticleType {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) abbreviation: Option<String>,
    pub(crate) colour: String,
    pub(crate) links: Vec<ReadLinkType>,
}

pub struct ReadLinkType {
    pub(crate) key: String,
    pub(crate) origin_name: String,
    pub(crate) origin_description: String,
    pub(crate) target_name: String,
    pub(crate) target_description: String,
    pub(crate) target_show: bool,
}

//// Resolution paths

fn read_resolution(project: &ParsedDictionary) -> Result<ResolutionPaths, String> {
    if let Some(resolve) = project.get("Resolve") {
        if !resolve.is_list() {
            return Err(format!(r#"The Resolve section must be a list."#));
        }
        let resolve = resolve.as_list().unwrap();
        let mut paths = vec![];
        for parsed_path in resolve.iter() {
            if !parsed_path.is_text() {
                return Err(format!("The elements of Resolve must be text document paths."));
            }
            let path = parsed_path.as_text().unwrap().as_str().to_string();
            paths.push(path);
        }
        Ok(paths)
    } else {
        Ok(vec![])
    }
}

pub type ResolutionPaths = Vec<String>;

//// Preamble

fn read_preamble(project: &ParsedDictionary) -> Result<Option<String>, String> {
    if let Some(preamble) = project.get("Preamble") {
        let tex = match write_tex_with(preamble, BreakMode::Never) {
            Ok(t) => t,
            Err(err) => return Err(tex_error_to_text(err)?),
        };
        Ok(Some(tex))
    } else {
        Ok(None)
    }
}

// TODO: Fix DefinedCommands instead of copy paste TeX macros
// pub type DefinedCommands = HashMap<String, DefinedCommand>;
//
// pub struct DefinedCommand {
//     replace: ParsedValue,
// }

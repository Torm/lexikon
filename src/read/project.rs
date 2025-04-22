//! Read the project file.

use std::path::PathBuf;
use khi::pdm::{ParsedDictionary};
use khi::{Dictionary, List, Text, Value};
use khi::tex::{BreakMode, write_tex_with};
use crate::{tex_error_to_text};

pub fn read_project(project: &ParsedDictionary) -> Result<ReadProject, String> {
    let model = read_header(project)?;
    let resolution_paths = read_resolution(project)?;
    let preamble = read_preamble(project)?;
    let dependencies = read_dependencies(project)?;
    Ok((model, resolution_paths, preamble, dependencies))
}

// pub type ReadProject = (ReadModel, ResolutionPaths, DefinedCommands); // TODO
pub type ReadProject = (String, ResolutionPaths, Option<String>, Vec<ReadDependencyEntry>);

pub struct ReadProjectX {
    model_path: String,
    resolution_paths: ResolutionPaths,
    // defined_commands: ... TODO
    preamble: Option<String>,
    dependencies: Vec<ReadDependencyEntry>,
}

//// Header

fn read_header(project: &ParsedDictionary) -> Result<String, String> {
    if let Some(model_file_path) = project.get("ModelFile") {
        if !model_file_path.is_text() {
            return Err(format!(r#"The ModelFile entry must be text."#));
        }
        Ok(String::from(model_file_path.as_text().unwrap().as_str()))
    } else {
        Ok(String::from("model.khi"))
    }
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

//// Dependencies

fn read_dependencies(project: &ParsedDictionary) -> Result<Vec<ReadDependencyEntry>, String> {
    let mut dependencies = vec![];
    if let Some(readt_dependencies) = project.get("Dependencies") {
        if !readt_dependencies.is_dictionary() {
            return Err(format!(r#"Dependencies section must be a dictionary."#));
        }
        let readt_dependencies = readt_dependencies.as_dictionary().unwrap();
        for (name, dependency) in readt_dependencies.iter() {
            if !dependency.is_dictionary() {
                return Err(format!(r#"Dependency must be a dictionary."#));
            }
            let dependency = dependency.as_dictionary().unwrap();
            let path = if let Some(path) = dependency.get("Path") {
                if !path.is_text() {
                    return Err(format!(r#"Dependency Path entry must be text."#));
                }
                PathBuf::from(path.as_text().unwrap().as_str())
            } else {
                return Err(format!(r#"Dependency must have a Path entry."#));
            };
            let include = if let Some(inclusion) = dependency.get("Include") {
                if !inclusion.is_text() {
                    return Err(format!(r#"Dependency Include entry must be text."#));
                }
                String::from(inclusion.as_text().unwrap().as_str())
            } else {
                String::from("All")
            };
            dependencies.push(ReadDependencyEntry { path, include });
        }
    }
    Ok(dependencies)
}

pub struct ReadDependencyEntry {
    pub path: PathBuf,
    pub include: String,
}

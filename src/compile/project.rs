//! Read the project file.

use std::path::{Path, PathBuf};
use khi::{Dictionary, List, Text, Value};
use khi::parse::pdm::{ParsedDictionary};
use crate::file::{read_file_content_to_dictionary, read_file_to_string};

pub struct ProjectSettings {
    pub(crate) resolution_paths: ResolutionPaths,
    pub(crate) style_path: Option<PathBuf>,
    pub(crate) config_paths: Vec<PathBuf>,
    pub(crate) dependencies: Vec<DependencySettings>,
}

pub struct DependencySettings {
    pub path: String,
    pub include: DependencyInclude,
    /// Output path of the dependency's documents within the /documents output directory.
    pub out: String,
}

#[derive(Copy, Clone)]
pub enum DependencyInclude {
    /// Include all articles and documents.
    All,
    /// Include all articles. Exclude documents.
    Articles,
    /// Include all articles. Obfuscate article keys. Exclude documents.
    Obfuscated,
}

impl DependencyInclude {
    pub fn include_documents(&self) -> bool {
        match self {
            DependencyInclude::All => true,
            DependencyInclude::Articles => false,
            DependencyInclude::Obfuscated => false,
        }
    }
    pub fn random_keys(&self) -> bool {
        match self {
            DependencyInclude::All => false,
            DependencyInclude::Articles => false,
            DependencyInclude::Obfuscated => true,
        }
    }
}

pub type ResolutionPaths = Vec<String>;

/// Read and parse project file.
pub fn read_project_file(path: &Path) -> Result<ProjectSettings, String> {
    let content = read_file_to_string(path, "project")?;
    let parse = read_file_content_to_dictionary(path, "project", &content)?;
    read_project_file_contents(&parse)
}

pub fn read_project_file_contents(project: &ParsedDictionary) -> Result<ProjectSettings, String> {
    let resolution_paths = read_resolution_paths(project)?;
    let style_path = read_style_path(project)?;
    let config_paths = read_configuration_paths(project)?;
    let dependencies = read_dependency_settings(project)?;
    Ok(ProjectSettings { resolution_paths, style_path, config_paths, dependencies })
}

fn read_resolution_paths(project: &ParsedDictionary) -> Result<ResolutionPaths, String> {
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

fn read_style_path(project: &ParsedDictionary) -> Result<Option<PathBuf>, String> {
    if let Some(style_path) = project.get("StyleFile") {
        if !style_path.is_text() {
            return Err(format!(r#"The StyleFile section must be text."#));
        }
        let style_path = PathBuf::from(style_path.as_text().unwrap().as_str().to_string());
        Ok(Some(style_path))
    } else {
        Ok(None)
    }
}

fn read_configuration_paths(project: &ParsedDictionary) -> Result<Vec<PathBuf>, String> {
    let mut paths = vec![];
    if let Some(preamble) = project.get("ConfigFiles") {
        if !preamble.is_list() {
            return Err(format!(r#"The ConfigFiles entry must be a list."#));
        }
        let read_paths = preamble.as_list().unwrap();
        for read_path in read_paths.iter() {
            if !read_path.is_text() {
                return Err(format!("A ConfigFiles entry must be a file system path."));
            }
            let path = PathBuf::from(read_path.as_text().unwrap().as_str());
            paths.push(path);
        }
    }
    Ok(paths)
}

fn read_dependency_settings(project: &ParsedDictionary) -> Result<Vec<DependencySettings>, String> {
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
                String::from(path.as_text().unwrap().as_str())
            } else {
                return Err(format!(r#"Dependency must have a Path entry."#));
            };
            let include = if let Some(inclusion) = dependency.get("Include") {
                if !inclusion.is_text() {
                    return Err(format!(r#"Dependency Include entry must be text."#));
                }
                let include = inclusion.as_text().unwrap().as_str();
                match include {
                    "All" => DependencyInclude::All,
                    "Articles" => DependencyInclude::Articles,
                    "Obfuscated" => DependencyInclude::Obfuscated,
                    _ => return Err(format!("Include must be All, Articles or Obfuscated.")),
                }
            } else {
                DependencyInclude::All
            };
            let out = if let Some(out) = dependency.get("OutPath") {
                if !out.is_text() {
                    return Err(format!(r#"OutPath must be a file system path."#));
                }
                out.as_text().unwrap().as_str().to_string()
            } else {
                String::from(".")
            };
            dependencies.push(DependencySettings { path, include, out });
        }
    }
    Ok(dependencies)
}

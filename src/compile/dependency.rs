//! Dependency inclusion

use std::path::Path;
use khi::{Dictionary};
use crate::compile::compile::{read_document_dir, read_model_file, read_project_file, Include, ReadDependency, ReadFsDocument};
use crate::compile::model::Model;
use crate::read::model::read_model;
use crate::read::project::read_project;

/// Read a dependency project.
///
/// 1) Read the dependency project file.
/// 2) Verify compatibility of the dependency with the project.
/// 3) Read the dependency documents.
pub fn include_dependency(model: &Model, dependency_path: &Path) -> Result<Vec<ReadFsDocument>, String> {
    let parsed_dependency_project = dependency_path.join("project.khi");
    let parsed_dependency_project = read_project_file(&parsed_dependency_project)?;
    let (dep_model_path, paths, preamble, dependencies) = read_project(&parsed_dependency_project)?;
    // Verify model is compatible.
    let parsed_model = read_model_file(Path::new(&dep_model_path))?;
    let read_model = read_model(&parsed_model)?;
    for read_type in read_model.iter() {
        let article_type = model.get_type(&read_type.key);
        if article_type.is_none() {
            return Err(format!("Dependency article type {} does not exist.", &read_type.key));
        }
        let article_type = article_type.unwrap();
        for read_link in &read_type.links {
            let link_type = article_type.get_link(&read_link.key);
            if link_type.is_none() {
                return Err(format!("Dependency link type {} does not exist.", &read_link.key));
            }
        }
    }
    // Verify dependencies of the dependency are dependencies of the project.
    // TODO
    //
    let dependency_documents_dir_path = dependency_path.join("documents");
    let read_documents = read_document_dir(&dependency_documents_dir_path)?;
    Ok(read_documents)
}

// //! Dependency inclusion
// 
// use std::path::Path;
// use crate::article::Articles;
// use crate::compile::document::{read_document_dir_from, UnlinkedDocument};
// use crate::makro::Macros;
// use crate::compile::project::{read_project_file_contents, read_project_file, DependencySettings, DependencyInclude};
// 
// pub struct ReadDependency<'a> {
//     pub include: DependencyInclude,
//     pub documents: Vec<UnlinkedDocument<'a>>,
//     pub preamble: Macros,
// }
// 
// pub fn read_dependencies(registry: &mut Articles, documents: &mut Vec<UnlinkedDocument>, configurations: &[DependencySettings]) -> Result<(), String> {
//     for configuration in configurations {
//         let path = configuration.path.as_str();
//         let include = configuration.include;
//         let dependency_documents = read_dependency(path)?;
//         read_dependency(registry, documents)?;
//     }
//     Ok(())
// }
// 
// /// Read a project dependency.
// ///
// /// 1) Read the dependency project file.
// /// 2) Verify compatibility of the dependency with the project.
// /// 3) Read the dependency documents.
// pub fn read_dependency<'a>(dependency_path: &'a Path) -> Result<Vec<UnlinkedDocument<'a>>, String> {
//     let parsed_dependency_project = dependency_path.join("project.khi");
//     let parsed_dependency_project = read_project_file(&parsed_dependency_project)?;
//     let read_dependency = read_project_file_contents(&parsed_dependency_project)?;
//     // Verify model is compatible.
//     let parsed_model = read_model_file(Path::new(&read_dependency.model_path))?;
//     let read_model = read_model(&parsed_model)?;
//     
//     compare_models(&model, &read_model)?;
//     // Verify dependencies of the dependency are dependencies of the project.
//     // TODO
//     //
//     let dependency_documents_dir_path = dependency_path.join("");
//     
//     let read_documents = read_document_dir_from(&dependency_documents_dir_path)?;
//     Ok(read_documents)
// }

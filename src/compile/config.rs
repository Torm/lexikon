use std::fs::{read_dir};
use std::path::{Path, PathBuf};
use crate::compile::makro::read_macro_definition_file;
use crate::compile::template::{read_template_file, Templates};
use crate::makro::{Macros};

pub fn read_configuration_files(templates: &mut Templates, macros: &mut Macros, paths: &[PathBuf]) -> Result<(), String> {
    for path in paths {
        if path.is_dir() {
            read_config_dir(macros, templates, path)?;
        } else {
            let file_name = path.file_name().unwrap();
            if file_name.as_encoded_bytes().ends_with(b".macros.khi") {
                let file_path = path.join(&file_name);
                eprintln!("Reading macro definition file {}", file_path.display());
                read_macro_definition_file(macros, &file_path)?;
            } else if file_name.as_encoded_bytes().ends_with(b".templates.khi") {
                let file_path = path.join(&file_name);
                eprintln!("Reading template file {}", file_path.display());
                read_template_file(templates, &file_path)?;
            } else {
                return Err(format!("Configuration file {} must be either a .macros.khi or a .templates.khi file.", file_name.to_str().unwrap()));
            }
        }
    }
    Ok(())
}

/// Read a configuration directory.
pub fn read_config_dir(macros: &mut Macros, templates: &mut Templates, path: &Path) -> Result<(), String> {
    for dir_entry in read_dir(&path).unwrap() {
        let dir_entry = dir_entry.unwrap();
        let file_name = dir_entry.file_name();
        let entry_type = dir_entry.file_type().unwrap();
        if entry_type.is_file() {
            if file_name.as_encoded_bytes().ends_with(b".macros.khi") {
                let file_path = path.join(&file_name);
                read_macro_definition_file(macros, &file_path)?;
                eprintln!("Read macro definition file {}", file_path.display());
            } else if file_name.as_encoded_bytes().ends_with(b".templates.khi") {
                let file_path = path.join(&file_name);
                read_template_file(templates, &file_path)?;
                eprintln!("Read template file {}", file_path.display());
            }
        } else if entry_type.is_dir() {
            let dir_path = path.join(&file_name);
            read_config_dir(macros, templates, &dir_path)?;
        }
    }
    Ok(())
}









use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use khi::parse::parse::{parse_dictionary_str, parse_list_str};
use khi::parse::parse::parser::{error_to_string};
use khi::parse::pdm::{ParsedDictionary, ParsedList};

/// Read a file to a string.
pub fn read_file_to_string(path: &Path, file_type: &str) -> Result<String, String> {
    let file = File::open(path);
    if file.is_err() {
        return Err(format!("Error opening {file_type} file '{}'; does it exist?", path.to_str().unwrap()));
    }
    let mut file = file.unwrap();
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err(format!("Error reading {file_type} file '{}'.", path.to_str().unwrap()));
    }
    Ok(contents)
}

/// Read a file to a string.
///
/// Returns None if the file starts with "# EXCLUDE".
pub fn read_excludable_file_to_string(file_path: &Path, file_type: &str) -> Result<Option<String>, String> {
    let contents = read_file_to_string(file_path, file_type)?;
    if contents.starts_with("# EXCLUDE") {
        return Ok(None);
    }
    Ok(Some(contents))
}

/// Parse a dictionary from the content of a file.
pub fn read_file_content_to_dictionary(path: &Path, file_type: &str, content: &str) -> Result<ParsedDictionary, String> {
    match parse_dictionary_str(content) {
        Ok(d) => Ok(d),
        Err(errors) => {
            let mut errorstr = String::new();
            errorstr.push_str(&format!("Error parsing {file_type} file '{}' as dictionary:\n\n", path.to_str().unwrap()));
            for err in errors { //todo
                errorstr.push_str(&format!("{}\n", error_to_string(&err)));
            }
            Err(errorstr)
        }
    }
}

/// Parse a dictionary from the content of a file.
pub fn read_file_content_to_list(path: &Path, file_type: &str, content: &str) -> Result<ParsedList, String> {
    match parse_list_str(content) {
        Ok(l) => Ok(l),
        Err(errors) => {
            let mut errorstr = String::new();
            errorstr.push_str(&format!("Error parsing {file_type} file '{}' as list:\n\n", path.to_str().unwrap()));
            for err in errors { //todo
                errorstr.push_str(&format!("{}\n", error_to_string(&err)));
            }
            Err(errorstr)
        }
    }
}

/// Update the modification times of the files.
///
/// If a file is identical to a previous version, set the modification time to
/// the old time.
pub fn carry_modification_dates(old_path: &Path, new_path: &Path) -> Result<(), String> {
    Ok(()) // TODO
}

/// Check if some content is identical to some file's content.
/// Returns false if the file does not exist or if it is not identical.
fn is_file_identical(path: &Path, content: &str) -> Result<bool, String> {
    if fs::exists(&path).unwrap() {
        let mut current_class_file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Unable to read current class file {}.", path.to_str().unwrap())),
        };
        let mut current_class_content = String::new();
        current_class_file.read_to_string(&mut current_class_content); // TODO
        Ok(current_class_content == content)
    } else {
        Ok(false)
    }
}

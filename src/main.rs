mod tex;
mod article;
pub mod document;
pub mod key;
pub mod makro;
pub mod name;
mod file;
pub mod web;
pub mod compile;
mod markup;
mod strings;
mod style;
pub mod relation;

use std::{env, fs};
use std::path::Path;
use std::slice::Iter;
use khi::{Dictionary, ParsedTupleElement, Text, Tuple, Value};
use khi::parse::pdm::{ParsedDictionary, ParsedTuple, ParsedValue};
use zeroarg::{parse_arguments, Argument};
use crate::article::Articles;
use crate::compile::dependency;
use crate::compile::config::read_configuration_files;
use crate::compile::document::read_source_dir;
use crate::compile::project::{read_project_file, ProjectSettings};
use crate::compile::style::{read_style_file};
use crate::compile::template::Templates;
use crate::document::Documents;
use crate::file::carry_modification_dates;
use crate::makro::Macros;
use crate::style::Styles;
use crate::web::asset::{include_assets, include_static_assets};
use crate::web::class::write_class_directory;
use crate::web::class_style::{write_class_style_css_file, write_class_style_json_file};
use crate::web::document::write_documents;
use crate::web::index::write_index;

type Html = String;

fn main() {
    let mut args = env::args();
    args.next();
    let arguments = match parse_arguments(args) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("Error: Command line syntax error.");
            return;
        }
    };
    let mut help = false;
    for argument in arguments {
        match argument {
            Argument::Operand(operand) => {
                eprintln!("Error: Operand {} not supported", &operand);
                return;
            }
            Argument::Attribute(k, _) => {
                eprintln!("Error: Attribute {} not supported", &k);
                return;
            }
            Argument::Flag(flag) => {
                if flag == "h" || flag == "help" || flag == "?" {
                    help = true;
                } else {
                    eprintln!("Error: Flag {} not supported", &flag);
                    return;
                }
            }
        }
    }
    if help {
        eprintln!("lexikon [-h]\n\nProcess project in current directory with command \"lexikon\".");
        return;
    }
    let root_dir_path = env::current_dir().unwrap();
    eprintln!("Processing project at {}.", root_dir_path.to_str().unwrap());
    match compile() {
        Ok(_) => {
            eprintln!("Project compiled successfully.")
        }
        Err(e) => {
            eprintln!("Error:\n{}", &e);
        }
    }
}

/// Compile the project.
pub fn compile() -> Result<(), String> {
    // Read project file.
    let ProjectSettings { resolution_paths, style_path, config_paths, dependencies } = read_project_file("project.khi".as_ref())?;
    // Read configuration files and class style file.
    let mut macros = Macros::new();
    let mut templates = Templates::new();
    read_configuration_files(&mut templates, &mut macros, &config_paths)?;
    let styles = if let Some(style_path) = style_path {
        read_style_file(style_path.as_ref())?
    } else {
        Styles::new()
    };
    // Read document source directory.
    let mut articles = Articles::new();
    let mut documents = Documents::new();
    read_source_dir(&templates, &resolution_paths, &macros, &mut articles, &mut documents, &Path::new("src"))?;
    eprintln!("Compelte. Art: {} Class: {} Docs: {}", articles.article_map.len(), articles.class_map.len(), documents.len()); ////////////////////////////////////////////
    // Read dependencies.
//    dependency::read_dependencies(&mut articles, &mut documents, read_project.dependencies.as_slice())?;//todo
    // Write website.
    let temp_path = Path::new(".website.tmp");
    let target_path = Path::new("website");
    // Clear and create temp target directory if it was for some reason not cleaned.
    if fs::exists(temp_path).unwrap() {
        fs::remove_dir_all(temp_path);
    }
    fs::create_dir(temp_path);
    // Write website files.
    write_class_style_json_file(temp_path, &styles)?;
    write_class_style_css_file(temp_path, &styles)?;
    write_class_directory(temp_path, &articles)?;
    write_documents(&styles, &resolution_paths, &articles, temp_path, &documents)?;
    //write_index(Path::new(""), temp_path);
    include_static_assets(temp_path)?;
    include_assets(temp_path)?;
//    include_index_and_icon(temp_path)?;
//    carry_modification_dates(target_path, temp_path)?;
    // Replace the old target directory with the newly generated files.
    fs::remove_dir_all(target_path);
    fs::rename(temp_path, target_path);
//    // Clear and create temp target directory.
//    if fs::exists(temp_path).unwrap() {
//        fs::remove_dir_all(temp_path);
//    }
    Ok(())
}

fn tex_error_to_text<T>(error: crate::tex::PreprocessorError) -> Result<T, String> {
    let err = match error {
        tex::PreprocessorError::IllegalTable(p) => format!("TeX: Illegal list at {}:{}.", p.line, p.column),
        tex::PreprocessorError::IllegalDictionary(p) => format!("TeX: Illegal dictionary at {}:{}.", p.line, p.column),
        tex::PreprocessorError::IllegalTuple(p) => format!("TeX: Illegal tuple at {}:{}.", p.line, p.column),
        tex::PreprocessorError::ZeroTable(p) => format!("TeX: Empty table at {}:{}.", p.line, p.column),
        tex::PreprocessorError::MacroError(p, e) => format!("TeX: Macro error at {}:{}:\n{}", p.line, p.column, &e),
        tex::PreprocessorError::MissingOptionalArgument(p) => format!("TeX: Missing optional argument at {}:{}.", p.line, p.column),
    };
    Err(err)
}

/// Read value of a text entry from a dictionary. Err if inextant.
pub fn read_text_value_or_err(dictionary: &ParsedDictionary, key: &str) -> Result<String, String> {
    let v = dictionary.get(key);
    if v.is_none() {
        return Err(format!("Entry {key} not found."));
    }
    let v = v.unwrap();
    if !v.is_text() {
        return Err(format!("Entry {key} is not text."));
    }
    let v = v.as_text().unwrap();
    Ok(String::from(v.as_str()))
}

pub fn tuple_split(tuple: &ParsedTuple) -> (Vec<&ParsedValue>, Vec<(&str, &ParsedValue)>) {
    tuple_splite(tuple.iter())
}

pub fn tuple_splite<'a>(tuple: impl Iterator<Item=ParsedTupleElement<'a, &'a ParsedValue>>) -> (Vec<&'a ParsedValue>, Vec<(&'a str, &'a ParsedValue)>) {
    let mut positional = vec![];
    let mut named = vec![];
    for element in tuple.into_iter() {
        match element {
            ParsedTupleElement::Element(e) => {
                positional.push(e);
            }
            ParsedTupleElement::NamedElement(n, e) => {
                named.push((n, e));
            }
        }
    }
    (positional, named)
}
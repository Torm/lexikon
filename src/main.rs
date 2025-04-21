//!

mod read;
mod compile;
mod web;

use std::env;
use khi::{Dictionary, Text, Value};
use khi::pdm::{ParsedDictionary};
use khi::tex::PreprocessorError;
use zeroarg::{parse_arguments, Argument};
use crate::compile::compile::compile;

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
        eprintln!("notarium [-h] (file)\n\nProcess current dictionary with \"notarium .\".");
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

fn tex_error_to_text<T>(error: PreprocessorError) -> Result<T, String> {
    let err = match error {
        PreprocessorError::IllegalTable(p) => format!("TeX: Illegal list at {}:{}.", p.line, p.column),
        PreprocessorError::IllegalDictionary(p) => format!("TeX: Illegal dictionary at {}:{}.", p.line, p.column),
        PreprocessorError::IllegalTuple(p) => format!("TeX: Illegal tuple at {}:{}.", p.line, p.column),
        PreprocessorError::ZeroTable(p) => format!("TeX: Empty table at {}:{}.", p.line, p.column),
        PreprocessorError::MacroError(p, e) => format!("TeX: Macro error at {}:{}:\n{}", p.line, p.column, &e),
        PreprocessorError::MissingOptionalArgument(p) => format!("TeX: Missing optional argument at {}:{}.", p.line, p.column),
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

use std::alloc::{alloc, alloc_zeroed, Layout};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::{read_dir, File};
use std::io::Read;
use std::mem::{uninitialized, zeroed, MaybeUninit};
use std::path::{Path};
use std::rc::{Rc, Weak};
use khi::{Dictionary, List, TaggedTuple, Text, Value};
use khi::parse::parse::parse_dictionary_str;
use khi::parse::pdm::{ParsedDictionary, ParsedList, ParsedTaggedTuple, Position};
use rand::TryRngCore;
use crate::article::{Articles};
use crate::compile::article::{read_article};
use crate::compile::makro::{read_macro_definitions_list};
use crate::compile::project::{DependencyInclude, ResolutionPaths};
use crate::compile::template::{Templates};
use crate::dir::Dir;
use crate::document::{Document, DocumentElement, PanelElement};
use crate::file::{read_excludable_file_to_string, read_file_content_to_dictionary};
use crate::makro::{LocalMacroRegistry, MacroMap, Macros};
use crate::key::KeyReader;
use crate::markup::{Markup};
use crate::preprocess_markup::process_unexpanded_markup;
use crate::tuple_split;

/// Directory configuration. Stored in dir.khi files. Only contains Directory name at the moment.
pub type DirConfig = String;

/// Read the dir file and return the name stored in it.
/// If no dir file, return directory name.
pub(crate) fn read_dir_file(dir_file_path: &Path) -> Result<DirConfig, String> {
    if dir_file_path.exists() {
        let mut file = File::open(dir_file_path).unwrap();
        let mut filebuf = String::new();
        file.read_to_string(&mut filebuf).unwrap();
        match parse_dictionary_str(filebuf.as_str()) {
            Ok(d) => {
                if let Some(name) = d.get("Name") {
                    if !name.is_text() {
                        return Err(format!("Name in dir file '{}' must be text.", dir_file_path.to_str().unwrap()));
                    }
                    let name = name.as_text().unwrap().as_str();
                    Ok(name.to_string())
                } else {
                    Err(format!("Name not found in dir file '{}'.", dir_file_path.to_str().unwrap()))
                }
            }
            Err(e) => {
                Err(format!("Error parsing dir file '{}'.", dir_file_path.to_str().unwrap()))
                // TODO
            }
        }
    } else {
        Err(format!("Dir file '{}' does not exist.", dir_file_path.to_str().unwrap()))
    }
}

/// Read a source dir. Recursively reads all nested directories and document files.
pub fn read_source_dir(templates: &Templates, resolution_paths: &ResolutionPaths, macros: &Macros, data: &mut Articles, documents: &mut Vec<Rc<Document>>, path: &Path, file_name: OsString) -> Result<Rc<Dir>, String> {
    read_document_dir(templates, resolution_paths, macros, data, documents, path, file_name, None)
}

/// Read a document dir.
fn read_document_dir(
    templates: &Templates, resolution_paths: &ResolutionPaths, macros: &Macros,
    data: &mut Articles, documents: &mut Vec<Rc<Document>>, path: &Path,
    file_name: OsString, parent: Option<Weak<Dir>>,
) -> Result<Rc<Dir>, String> {

    let name = {
        let dir_file_path = path.join("dir.khi");
        if dir_file_path.exists() { // TODO Is file?
            let dir_config = read_dir_file(&dir_file_path)?;
            dir_config
        } else {
            if parent.is_some() { // The tree root changes name from src to documents.
                file_name.clone().into_string().unwrap()
            } else {
                String::from("Documents")
            }
        }
    };

    // Todo: Use the UniqueRc when it is stable
    let dir = {
        let dir: Dir = Dir { name: name.clone(), file_name: OsString::from("ERROR"), subdirs: vec![], subdocs: vec![], parent: parent.clone() };
        Rc::new(dir)
    };
    let w = Rc::downgrade(&dir);


    let mut subdirs = vec![];
    let mut subdocs = vec![];

    for dir_entry in read_dir(&path).unwrap() {
        let dir_entry = dir_entry.unwrap();
        let file_name = dir_entry.file_name();
        let entry_type = dir_entry.file_type().unwrap();
        if entry_type.is_file() {
            if file_name.as_encoded_bytes().ends_with(b".document.khi") || file_name.as_encoded_bytes().ends_with(b".doc.khi") {
                let document_path = path.join(&file_name);
                eprintln!("Reading document file {}", document_path.to_str().unwrap());
                let subdoc = read_document_file(templates, documents, data, macros, DependencyInclude::All, file_name, &document_path, w.clone())?;
                if let Some(subdoc) = subdoc { // If the file is not excluded.
                    subdocs.push(subdoc);
                }
            }
        } else if entry_type.is_dir() {
            let dir_path = path.join(&file_name);
            let subdir = read_document_dir(templates, resolution_paths, macros, data, documents, &dir_path, file_name, Some(w.clone()))?;
            subdirs.push(subdir);
        }
    }
    // Todo: Use the UniqueRc when it is stable
    let dir = unsafe {
        let r = Rc::into_raw(dir);
        let r = r.cast_mut();
        let file_name = if parent.is_some() { // Tree root changes path from /src to /documents
            file_name
        } else {
            OsString::from("documents")
        };
        *r = Dir { name, file_name, subdirs, subdocs, parent };
        Rc::from_raw(r)
    };
    Ok(dir)
}

pub struct DocumentKey(String);

pub fn read_document_file(templates: &Templates, documents: &mut Vec<Rc<Document>>, registry: &mut Articles, macros: &Macros, include: DependencyInclude, file_name: OsString, path: &Path, parent_dir: Weak<Dir>) -> Result<Option<Rc<Document>>, String> {
    let content = match read_excludable_file_to_string(path, "document")? {
        None => return Ok(None),
        Some(c) => c,
    };
    let dict = read_file_content_to_dictionary(path, "document", &content)?;
    let document = read_document_khidict(templates, documents, registry, macros, include, file_name, &dict, parent_dir)?;
    Ok(Some(document))
}

pub fn read_document_khidict(templates: &Templates, documents: &mut Vec<Rc<Document>>, registry: &mut Articles, macros: &Macros, include: DependencyInclude, file_name: OsString, document: &ParsedDictionary, parent_dir: Weak<Dir>) -> Result<Rc<Document>, String> {
    let key = if let Some(key) = document.get("Key") {
        if !key.is_text() {
            return Err(format!("Key in document must be text."));
        }
        key.as_text().unwrap().as_str().to_string()
    } else {
        return Err(format!("Document must have a Key entry."));
    };
    let title = if let Some(title) = document.get("Title") {
        if !title.is_text() {
            return Err(format!("Title in document must be text."));
        }
        title.as_text().unwrap().as_str().to_string()
    } else {
        return Err(format!("Document must have a Title entry."));
    };
    let description = if let Some(description) = document.get("Description") {
        if !description.is_text() {
            return Err(format!("Description in document must be text."));
        }
        Some(description.as_text().unwrap().as_str().to_string())
    } else {
        None
    };
    // TODO: Include Author?
    // Read local macro definitions.
    let mut document_macros = Macros::new();
    if let Some(macros) = document.get("Macros") {
        if !macros.is_list() {
            return Err(format!("Macros entry in document must be a list."));
        }
        let list = macros.as_list().unwrap();
        read_macro_definitions_list(&mut document_macros, list)?
    };
    let local_macros = LocalMacroRegistry::new(macros, &document_macros);
    // Read resolution paths.
    let resolution_paths = if let Some(resolution_paths) = document.get("Resolve") {
        if !resolution_paths.is_list() {
            return Err(format!("The Resolve section must be a list."));
        }
        let parsed_paths = resolution_paths.as_list().unwrap();
        read_resolution_paths(parsed_paths)?
    } else {
        vec![]
    };
    // Read aliases.
    let aliases = if let Some(aliases) = document.get("Alias") {
        if !aliases.is_dictionary() {
            return Err(format!("The Alias section must be a dictionary."));
        }
        let aliases = aliases.as_dictionary().unwrap();
        let mut amap = HashMap::new();
        for (read_alias, target) in aliases.iter() {
            if !target.is_text() {
                return Err(format!("Alias value must be text."));
            }
            let target = target.as_text().unwrap().as_str();
            amap.insert(String::from(read_alias), String::from(target));
        }
        amap
    } else {
        HashMap::new()
    };
    // Read document content section.
    let structure = if let Some(parsed_content) = document.get("Content") {
        if !parsed_content.is_list() {
            return Err(format!("The Content section must be a list."));
        }
        let parsed_content = parsed_content.as_list().unwrap();
        read_content_section(templates, registry, &aliases, &local_macros, parsed_content, key.as_str())?
    } else {
        vec![]
    };
    // Warn document // TODO PRAGMA
    for delem in structure.iter() {
        if let DocumentElement::Panel(elements) = delem {
            for el in elements.iter() {
                if let PanelElement::ArticleLink { key, .. } = el {
                    let article_ref = registry.get_article(key).unwrap();
                    let article = article_ref.borrow_mut();
                    if article.content.is_empty() {
                        eprintln!("[Warning] Article {} has no content.", key); // TODO PRAGMA
                    }
                }
            }
        }
    }
    // Register document.
    let document = Document { key, title, description, resolution_paths, file_name, parent_dir, structure };
    let document = Rc::new(document);
    documents.push(document.clone());
    Ok(document)
}

/// Read the content of a document.
fn read_content_section(
    templates: &Templates,
    articles: &mut Articles,
    aliases: &HashMap<String, String>,
    macro_map: &LocalMacroRegistry,
    content_list: &ParsedList,
    document_key: &str
) -> Result<Vec<DocumentElement>, String> {
    let mut elements = vec![];
    let mut heading_level = 1; // Keep track of heading level to prevent bad sectioning structure.
    for entry in content_list.iter() {
        if !entry.is_tagged_tuple() {
            return Err(format!("Element at {}:{} must be a tagged value", entry.from().line, entry.from().column));
        }
        let tag = entry.as_tagged_tuple().unwrap();
        let at = entry.from();
        let name = tag.name().unwrap();
        let (tuple, opts) = tuple_split(tag);
        if name == "H1" || name == "H2" || name == "H3" || name == "H4" || name == "H5" || name == "H6" {
            let HeadingElement { level, heading, index, inline } = read_heading_element(macro_map, tag)?;
            if level > heading_level + 1 {
                return Err(format!("Heading level jumped by multiple levels at {}:{}", entry.from().line, entry.from().column));
            }
            heading_level = level;
            if inline {
                append_paneled_element(&mut elements, PanelElement::Heading { level, heading, index })?;
            } else {
                elements.push(DocumentElement::Heading { level, heading, index });
            }
        } else if name == "P" {
            if tuple.len() != 1 {
                return Err(format!("<P> takes 1 argument."));
            }
            let argument = tuple.get(0).unwrap();
            elements.push(DocumentElement::Paragraph(process_unexpanded_markup(macro_map, argument)?));
        } else if name == "@" {
            let include = read_include_element(aliases, tag, at, document_key)?;
            append_paneled_element(&mut elements, include)?;
//        } else if name == "I" { // TODO: Panel subcollections
//            if tuple.len() != 1 {
//                return Err(format!("<I> takes 1 list argument."));
//            }
//            let argument = tuple.get(0).unwrap();
//            read_inline_grouping(&mut read_elements, &mut read_articles, argument, at, document_key)?;
        } else {
            let article_link = read_article_element(templates, articles, aliases, macro_map, tag, at, document_key)?;
            append_paneled_element(&mut elements, article_link)?;
        }
    }
    Ok(elements)
}

// fn read_inline_grouping(read_elements: &mut ReadElements, read_articles: &mut Vec<ReadArticle>, value: &ParsedValue, at: Position, document_key: &str) -> Result<(), String> {
//     if value.len_as_tuple() != 2 {
//         return Err(format!("Inline section must have 2 arguments."));
//     }
//     let index = value.as_tuple().unwrap().get(0).unwrap();
//     if !index.is_text() {
//         return Err(format!("Index must be text."));
//     }
//     let index = index.as_text().unwrap().as_str().to_string();
//     let tag = value.as_tuple().unwrap().get(1).unwrap();
//     if let Some(tag) = tag.as_tagged() {
//         read_inline_section_tag(read_elements, read_articles, tag, at, document_key, index)?;
//     } else if let Some(list) = tag.as_list() {
//         for element in list.iter() {
//             if !element.is_tagged() {
//                 return Err(format!("Value of inline section list must be tag."));
//             }
//             let element = element.as_tagged().unwrap();
//             read_inline_section_tag(read_elements, read_articles, element, at, document_key, index.clone())?;
//         }
//     } else {
//         return Err(format!("Inline section must have a tag or list of tags value."));
//     }
//     Ok(())
// }
// 
// fn read_inline_section_tag(read_elements: &mut ReadElements, read_articles: &mut Vec<ReadArticle>, tag: &ParsedTaggedValue, at: Position, document_key: &str, index: String) -> Result<(), String> {
//     if tag.name.as_ref() == "@" {
//         let mut include = read_content_include(tag, at, document_key)?;
//         match &mut include { // TODO: Group elements instead of displaying an index above each.
//             ReadInlineElement::Heading(_, _, i) => {
//                 *i = Some(index);
//             }
//             ReadInlineElement::Link(_, i) => {
//                 *i = Some(index);
//             }
//         }
//         append_inline_element(read_elements, include)?;
//     } else {
//         let (mut read_element, read_article) = read_article_definition(tag, at, document_key)?;
//         match &mut read_element {
//             ReadInlineElement::Heading(_, _, i) => {
//                 *i = Some(index);
//             }
//             ReadInlineElement::Link(_, i) => {
//                 *i = Some(index);
//             }
//         }
//         read_articles.push(read_article);
//         append_inline_element(read_elements, read_element)?;
//     }
//     Ok(())
// }

/// Append a link to the document structure. // TODO: Move to compilation
///
/// If the tail element is an article panel, the link is appended to it. Otherwise,
/// a new article panel is created which the link is appended to.
pub fn append_paneled_element(structure: &mut Vec<DocumentElement>, element: PanelElement) -> Result<(), String> {
    if let Some(DocumentElement::Panel(labels)) = structure.last_mut() {
        labels.push(element);
    } else {
        let panel = DocumentElement::Panel(vec![element]);
        structure.push(panel);
    }
    Ok(())
}

struct HeadingElement {
    level: u8,
    heading: Markup,
    index: Option<String>,
    inline: bool,
}

/// Read a heading command in document contents.
///
/// (level, heading, index, inline)
fn read_heading_element(macros: &impl MacroMap, tag: &ParsedTaggedTuple) -> Result<HeadingElement, String> {
    let command = tag.name().unwrap();
    let (positional, named) = tuple_split(tag);
    let (index, heading) = if tag.len() == 2 {
        let index = positional.get(0).unwrap();
        if !index.is_text() { return Err(format!("Heading at {}:{} must be tuple with 1 or 2 elements.", "?", "?")) } // TODO .at()
        let index = index.as_text().unwrap().as_str().to_string();
        let heading = Markup::from_markup(macros, positional.get(1).unwrap())?;
        (Some(index), heading)
    } else if tag.len() == 1 {
        let heading = Markup::from_markup(macros, positional.get(0).unwrap())?;
        (None, heading)
    } else {
        return Err(format!("Heading at {}:{} must be tuple with 1 or 2 elements.", "?", "?")); // TODO .at()
    };
    // Check inline param.
    let inline = tag.get_attribute_by("i").is_some();
    // Level
    let level = match command {
        "H1" => 1, "H2" => 2, "H3" => 3, "H4" => 4, "H5" => 5, "H6" => 6, _ => unreachable!(),
    };
    if level == 1 {
        return Err(format!("Found illegal heading at {}:{}. H1 headings are not allowed.", "?", "?")); // TODO .at()
    }
    let heading = HeadingElement { level, heading, index, inline };
    Ok(heading)
}

fn read_article_element(templates: &Templates, registry: &mut Articles, aliases: &HashMap<String, String>, macros: &impl MacroMap, tag: &ParsedTaggedTuple, at: Position, document_key: &str) -> Result<PanelElement, String> {
    let read_article = read_article(templates, macros, registry, tag, at, document_key)?;
    let key = read_article.borrow().key.clone();
    let element = PanelElement::ArticleLink { key, index: None };
    Ok(element)
}

/// Read an article inclusion in document content.
///
/// Either a class key or an article key must be specified.
fn read_include_element(aliases: &HashMap<String, String>, tag: &ParsedTaggedTuple, at: Position, document_key: &str) -> Result<PanelElement, String> {
    let (include_key, named) = tuple_split(tag);
    if include_key.len() != 1 {
        return Err(format!("Content include takes 1 key argument."));
    }
    let include_key = include_key.get(0).unwrap();
    if !include_key.is_text() {
        return Err(format!("Value of include command <@> at {}:{} must be text.", at.line, at.column));
    }
    let include_key = include_key.as_text().unwrap().as_str();
    match read_include_key(document_key, include_key)? {
        LinkKey::Class(key) => Ok(PanelElement::ClassLink { key: Rc::from(key), index: None }),
        LinkKey::Article(key) => Ok(PanelElement::ArticleLink { key: Rc::from(key), index: None }),
    }
}

enum LinkKey {
    Class(String), Article(String)
}

/// Read the key in an include element. This key can be a class key or an
/// article key.
fn read_include_key(document_key: &str, key: &str) -> Result<LinkKey, String> {
    let mut reader = KeyReader::new(key);
    if reader.is_plain_key() {
        let (key, article) = reader.parse_plain()?;
        if !reader.is_at_end() {
            return Err(format!("Expected end in key."));
        }
        if article {
            Ok(LinkKey::Article(key))
        } else {
            Ok(LinkKey::Class(key))
        }
    } else if reader.is_parenthesized() {
        let key = reader.parse_parenthesized()?;
        if !reader.is_at_end() {
            return Err(format!("Expected end in key."));
        }
        Ok(LinkKey::Article(format!("{}@{}", &key, document_key)))
    } else {
        return Err(format!("Invalid declaration key."));
    }
}

fn read_resolution_paths(parsed_paths: &ParsedList) -> Result<Vec<String>, String> {
    let mut paths = vec![];
    for parsed_path in parsed_paths.iter() {
        if !parsed_path.is_text() {
            return Err(format!("An element of Resolve must be text."));
        }
        let path = parsed_path.as_text().unwrap().as_str().to_string();
        paths.push(path)
    }
    Ok(paths)
}

fn resolve_alias<'a>(aliases: &'a HashMap<String, String>, alias: &'a str) -> &'a str {
    if let Some(target) = aliases.get(alias) {
        target.as_str()
    } else {
        alias
    }
}

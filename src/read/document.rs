//! Reading of documents.

use khi::{Dictionary, List, Tagged, Text, Tuple, Value};
use khi::pdm::{ParsedDictionary, ParsedList, ParsedTaggedValue, ParsedValue, Position};
use crate::read::article::{read_article_declaration, ReadArticle};
use crate::read::content::read_content_text;
use crate::compile::key::KeyReader;
//// Document

pub fn read_document(document: &ParsedDictionary) -> Result<ReadDocument, String> {
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
    let preamble = None;
    // Read preamble. // TODO
    // let preamble = if let Some(preamble) = parse.get("TexPreamble") {
    //     let tex = match write_tex_with(preamble, BreakMode::Never) {
    //         Ok(t) => t,
    //         Err(err) => return Err(tex_error_to_text(err)?),
    //     };
    //     Some(tex)
    // } else {
    //     None
    // };
    let resolution_paths = if let Some(resolution_paths) = document.get("Resolve") {
        if !resolution_paths.is_list() {
            return Err(format!("The Resolve section must be a list."));
        }
        let parsed_paths = resolution_paths.as_list().unwrap();
        read_resolution_paths(parsed_paths)?
    } else {
        vec![]
    };
    // Read document content section.
    let (read_elements, read_articles) = if let Some(parsed_content) = document.get("Content") {
        if !parsed_content.is_list() {
            return Err(format!("The Content section must be a list."));
        }
        let parsed_content = parsed_content.as_list().unwrap();
        read_document_content(parsed_content, key.as_str())?
    } else {
        (vec![], vec![])
    };
    Ok(ReadDocument { key, title, description, preamble, resolution_paths, read_elements, read_articles })
}

pub struct ReadDocument {
    pub(crate) key: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) preamble: Option<String>,
    pub(crate) resolution_paths: Vec<String>,
    pub(crate) read_elements: ReadElements,
    pub(crate) read_articles: Vec<ReadArticle>,
}

//// Resolution paths

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

//// Content

/// Read the content of a document.
fn read_document_content(parsed_content: &ParsedList, document_key: &str) -> Result<(ReadElements, Vec<ReadArticle>), String> {
    let mut read_elements = vec![];
    let mut read_articles = vec![];
    let mut heading_level = 1; // Keep track of heading level to prevent bad sectioning structure.
    for parsed_element in parsed_content.iter() {
        if !parsed_element.is_tagged() {
            return Err(format!("Element at {}:{} must be a tagged value", parsed_element.from().line, parsed_element.from().column));
        }
        let tag = parsed_element.as_tagged().unwrap();
        let at = parsed_element.from();
        let name = tag.name.as_ref();
        let value = tag.get();
        if name == "H1" || name == "H2" || name == "H3" || name == "H4" || name == "H5" || name == "H6" {
            let (level, heading, index, inline) = read_content_heading(tag)?;
            if level > heading_level + 1 {
                return Err(format!("Heading level jumped by multiple levels at {}:{}", value.from().line, value.from().column));
            }
            heading_level = level;
            if inline {
                append_inline_element(&mut read_elements, ReadInlineElement::Heading(level, heading, index))?;
            } else {
                read_elements.push(ReadElement::Heading(level, index, heading));
            }
        } else if name == "P" {
            let paragraph = read_content_text(value)?;
            read_elements.push(ReadElement::Paragraph(paragraph));
        } else if name == "@" {
            let include = read_content_include(tag, at, document_key)?;
            append_inline_element(&mut read_elements, include)?;
        } else if name == "I" {
            read_inline_section(&mut read_elements, &mut read_articles, value, at, document_key)?;
        } else {
            let (read_link, read_article) = read_content_declaration(tag, at, document_key)?;
            append_inline_element(&mut read_elements, read_link)?;
            read_articles.push(read_article);
        }
    }
    Ok((read_elements, read_articles))
}

fn read_inline_section(read_elements: &mut ReadElements, read_articles: &mut Vec<ReadArticle>, value: &ParsedValue, at: Position, document_key: &str) -> Result<(), String> {
    if value.len_as_tuple() != 2 {
        return Err(format!("Inline section must have 2 arguments."));
    }
    let index = value.as_tuple().unwrap().get(0).unwrap();
    if !index.is_text() {
        return Err(format!("Index must be text."));
    }
    let index = index.as_text().unwrap().as_str().to_string();
    let tag = value.as_tuple().unwrap().get(1).unwrap();
    if let Some(tag) = tag.as_tagged() {
        read_inline_section_tag(read_elements, read_articles, tag, at, document_key, index)?;
    } else if let Some(list) = tag.as_list() {
        for element in list.iter() {
            if !element.is_tagged() {
                return Err(format!("Value of inline section list must be tag."));
            }
            let element = element.as_tagged().unwrap();
            read_inline_section_tag(read_elements, read_articles, element, at, document_key, index.clone())?;
        }
    } else {
        return Err(format!("Inline section must have a tag or list of tags value."));
    }
    Ok(())
}

fn read_inline_section_tag(read_elements: &mut ReadElements, read_articles: &mut Vec<ReadArticle>, tag: &ParsedTaggedValue, at: Position, document_key: &str, index: String) -> Result<(), String> {
    if tag.name.as_ref() == "@" {
        let mut include = read_content_include(tag, at, document_key)?;
        match &mut include { // TODO: Group elements instead of displaying an index above each.
            ReadInlineElement::Heading(_, _, i) => {
                *i = Some(index);
            }
            ReadInlineElement::Link(_, i) => {
                *i = Some(index);
            }
        }
        append_inline_element(read_elements, include)?;
    } else {
        let (mut read_element, read_article) = read_content_declaration(tag, at, document_key)?;
        match &mut read_element {
            ReadInlineElement::Heading(_, _, i) => {
                *i = Some(index);
            }
            ReadInlineElement::Link(_, i) => {
                *i = Some(index);
            }
        }
        read_articles.push(read_article);
        append_inline_element(read_elements, read_element)?;
    }
    Ok(())
}

/// Append a link to the document structure. // TODO: Move to compilation
///
/// If the tail element is an article panel, the link is appended to it. Otherwise,
/// a new article panel is created which the link is appended to.
pub fn append_inline_element(structure: &mut ReadElements, element: ReadInlineElement) -> Result<(), String> {
    if let Some(ReadElement::Panel(labels)) = structure.last_mut() {
        labels.push(element);
    } else {
        let panel = ReadElement::Panel(vec![element]);
        structure.push(panel);
    }
    Ok(())
}

pub type ReadElements = Vec<ReadElement>;

/// An element of the structure of a document.
pub enum ReadElement {
    Heading(u8, Option<String>, String),
    Paragraph(String),
    Panel(Vec<ReadInlineElement>),
}

/// An element of a links panel.
pub enum ReadInlineElement { // TODO: Remove this, add inline field to heading, handle in compilation
    /// An inline heading within the links.
    Heading(u8, String, Option<String>),
    /// A label containing an article and optional label index.
    Link(String, Option<String>),
}

fn read_content_heading(tag: &ParsedTaggedValue) -> Result<(u8, String, Option<String>, bool), String> {
    let command = tag.name.as_ref();
    let value = tag.get();
    let (index, heading) = if value.is_tuple() {
        let value = value.as_tuple().unwrap();
        if value.len() != 2 {
            return Err(format!("Heading at {}:{} must be tuple with 1 or 2 elements.", tag.value.from().line, tag.value.from().column));
        }
        (Some(read_content_text(value.get(0).unwrap())?), read_content_text(value.get(1).unwrap())?)
    } else {
        (None, read_content_text(value)?)
    };
    // Check inline param.
    let inline = tag.attributes.iter().find(|x| x.0.as_ref() == "i").is_some();
    // Level
    let level = match command {
        "H1" => 1, "H2" => 2, "H3" => 3, "H4" => 4, "H5" => 5, "H6" => 6, _ => unreachable!(),
    };
    if level == 1 {
        return Err(format!("Found illegal heading at {}:{}. H1 headings are not allowed.", tag.value.from().line, tag.value.from().column));
    }
    Ok((level, heading, index, inline))
}

fn read_content_include(tag: &ParsedTaggedValue, at: Position, document_key: &str) -> Result<ReadInlineElement, String> {
    let include_key = tag.get();
    if !include_key.is_text() {
        return Err(format!("Value of @ include command at {}:{} must be text.", at.line, at.column));
    }
    let include_key = include_key.as_text().unwrap().as_str();
    let key = match read_include_key(include_key)? {
        IncludeKey::Class(key) => key,
        IncludeKey::Article(key) => key,
        IncludeKey::Local(key) => format!("{}@{}", &key, document_key),
    };
    // TODO: Allow article aliasing.
    Ok(ReadInlineElement::Link(key, None))
}

fn read_content_declaration(tag: &ParsedTaggedValue, at: Position, document_key: &str) -> Result<(ReadInlineElement, ReadArticle), String> {
    let read_article = read_article_declaration(tag, at, document_key)?;
    let key = read_article.article_key.clone();
    let element = ReadInlineElement::Link(key, None);
    Ok((element, read_article))
}

fn read_include_key(key: &str) -> Result<IncludeKey, String> {
    let mut reader = KeyReader::new(key);
    if reader.is_plain_key() {
        let (key, article) = reader.parse_plain()?;
        if !reader.is_at_end() {
            return Err(format!("Expected end in key."));
        }
        if article {
            Ok(IncludeKey::Article(key))
        } else {
            Ok(IncludeKey::Class(key))
        }
    } else if reader.is_parenthesized() {
        let key = reader.parse_parenthesized()?;
        if !reader.is_at_end() {
            return Err(format!("Expected end in key."));
        }
        Ok(IncludeKey::Local(key))
    } else {
        return Err(format!("Invalid declaration key."));
    }
}

enum IncludeKey {
    Class(String), Article(String), Local(String),
}

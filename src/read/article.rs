//! Reading articles.

use khi::pdm::{ParsedDictionary, ParsedTaggedValue, ParsedValue, Position};
use khi::{Dictionary, List, Tagged, Text, Value};
use khi::tex::{write_tex_with, BreakMode};
use crate::{tex_error_to_text};
use crate::read::content::read_content_text;
use crate::compile::key::KeyReader;

pub fn read_article_declaration(tag: &ParsedTaggedValue, at: Position, document_key: &str) -> Result<ReadArticle, String> {
    let type_key = tag.name.to_string();
    let mut arguments: Vec<&ParsedValue> = tag.get().iter_as_tuple().into_iter().collect();
    // Extract links.
    let links = if !arguments.is_empty() && arguments[0].is_dictionary() {
        let parsed_links = arguments.remove(0).as_dictionary().unwrap();
        read_links(parsed_links, at)?
    } else {
        vec![]
    };
    // Extract key.
    let (class_key, article_key) = if let Some(key) = remove_first(&mut arguments) { // TODO: Read class key and article key in compilation
        if !key.is_text() {
            return Err(format!("Key in article at {}:{} must be text.", at.line, at.column));
        }
        let key = key.as_text().unwrap().as_str();
        match read_article_key_declaration(key)? {
            DeclarationKey::Class(key) => (key.clone(), format!("{}@{}", &key, document_key)),
            DeclarationKey::ClassAndLocal(class_key, local_key) => (class_key, format!("{}@{}", &local_key, document_key)),
            DeclarationKey::Local(key) => (format!("{}@{}", &key, document_key), format!("{}@{}", &key, document_key)),
        }
    } else {
        return Err(format!("Expected first argument of key in article at {}:{}.", at.line, at.column));
    };
    // Extract names.
    let names = if let Some(names) = remove_first(&mut arguments) {
        read_declaration_names(names)?
    } else {
        return Err(format!("Expected second argument of names in article at {}:{}.", at.line, at.column));
    };
    // Extract content.
    let content = if let Some(content) = remove_first(&mut arguments) {
        read_article_content(content)?
    } else {
        vec![]
    };
    // Check all arguments taken.
    if arguments.len() != 0 {
        return Err(format!("More arguments than expected in article at {}:{}.", at.line, at.column));
    }
    Ok(ReadArticle { type_key, class_key, article_key, names, content, links })
}

fn remove_first<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.len() > 0 {
        Some(vec.remove(0))
    } else {
        None
    }
}

pub struct ReadArticle {
    pub(crate) type_key: String,
    pub(crate) class_key: String,
    pub(crate) article_key: String, // TODO: Parse class and article keys in compilation
    pub(crate) names: Vec<String>,
    pub(crate) content: Vec<ArticleElement>,
    pub(crate) links: Vec<(String, Vec<String>)>,
}

fn read_links(parsed_links: &ParsedDictionary, at: Position) -> Result<Vec<(String, Vec<String>)>, String> {
    let mut links = vec![];
    for (link_type, linked_classes) in parsed_links.iter() {
        let link_type = link_type.to_string();
        if !linked_classes.is_list() {
            return Err(format!("Linked classes must be a list in article at {}:{}.", at.line, at.column));
        }
        let linked_classes = linked_classes.as_list().unwrap();
        let mut linked_class_keys = vec![];
        for linked_class in linked_classes.iter() {
            if !linked_class.is_text() {
                return Err(format!("Linked class must be a text in article at {}:{}.", at.line, at.column));
            }
            let linked_class = linked_class.as_text().unwrap();
            linked_class_keys.push(linked_class.as_str().to_string());
        }
        links.push((link_type, linked_class_keys));
    }
    Ok(links)
}

/// Read the declared names.
fn read_declaration_names(argument: &ParsedValue) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    if argument.is_list() {
        for name in argument.as_list().unwrap().iter() {
            let name = read_content_text(name)?;
            names.push(name);
        }
    } else {
        let name = read_content_text(argument)?;
        names.push(name)
    }
    Ok(names)
}

/// Read the body of an article.
fn read_article_content(input: &ParsedValue) -> Result<Vec<ArticleElement>, String> {
    let mut article_elements = vec![];
    if input.is_list() {
        let content = input.as_list().unwrap();
        for c in content.iter() {
            if !c.is_tagged() {
                // return Err(format!("Element of article content list at {}:{} must be a tagged value", c.from().line, c.from().column));
                // TODO: Below is very temporary workaround.
                let txt = read_content_text(c)?;
                article_elements.push(ArticleElement::Html(format!(r#"<p>{}</p>"#, &txt))); // TODO Use Paragraph but wrap in div not p?
                continue;
            }
            let tag = c.as_tagged().unwrap();
            let name = tag.name.as_ref();
            if name == "H1" { // TODO: Not allowed, + check levels, check in compilation?
                return Err(format!("H1 heading not allowed in article at {}:{}.", input.from().line, input.from().column));
            } else if name == "H2" {
                let tex = read_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleElement::Heading(2, tex));
            } else if name == "H3" {
                let tex = read_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleElement::Heading(3, tex));
            } else if name == "H4" {
                let tex = read_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleElement::Heading(4, tex));
            } else if name == "H5" {
                let tex = read_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleElement::Heading(5, tex));
            } else if name == "H6" {
                let tex = read_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleElement::Heading(6, tex));
            } else if name == "P" {
                let tex = read_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleElement::Paragraph(tex));
            } else if name == "$$" {
                let tex = write_tex_with(tag.value.as_ref(), BreakMode::Never).or_else(tex_error_to_text)?;
                let tex = format!("\\[{tex}\\]");
                article_elements.push(ArticleElement::Paragraph(tex));
            // } else if name == "Ol" {
            //     if !tag.value.is_list() {
            //         return Err(format!("Expected list at {}:{}.", tag.value.from().line, tag.value.from().column));
            //     }
            //     let list = tag.value.as_list().unwrap();
            //     let mut html = String::new();
            //     html.push_str("<ol>");
            //     for v in list.iter() {
            //         html.push_str(&format!("<li>{}</li>", read_content_text(v)?));
            //     }
            //     html.push_str("</ol>");
            //     article_elements.push(ArticleElement::Html(html));
            // } else if name == "Ul" {
            //     if !tag.value.is_list() {
            //         return Err(format!("Expected list at {}:{}.", tag.value.from().line, tag.value.from().column));
            //     }
            //     let list = tag.value.as_list().unwrap();
            //     let mut html = String::new();
            //     html.push_str("<ul>");
            //     for v in list.iter() {
            //         html.push_str(&format!("<li>{}</li>", read_content_text(v)?));
            //     }
            //     html.push_str("</ul>");
            //     article_elements.push(ArticleElement::Html(html));
            } else {
                return Err(format!("Element of article content list at {}:{} must be a heading or paragraph.", c.from().line, c.from().column));
            }
        }
    } else {
        let tex = read_content_text(input)?;
        article_elements.push(ArticleElement::Paragraph(tex));
    }
    Ok(article_elements)
}


/// Element of article content.
#[derive(Clone)]
pub enum ArticleElement {
    Heading(u8, String),
    Paragraph(String),
    Html(String),
}

/// Read a declaration key.
///
/// Supports `prjkey`, `(dockey)` and `prjkey(dockey)`.
fn read_article_key_declaration(declaration: &str) -> Result<DeclarationKey, String> {
    let mut reader = KeyReader::new(declaration);
    if reader.is_plain_key() {
        let (key, article) = reader.parse_plain()?;
        if article {
            return Err(format!("Declaration key cannot be an article key."));
        }
        reader.skip_whitespace()?;
        if reader.is_parenthesized() {
            let local = reader.parse_parenthesized()?;
            if !reader.is_at_end() {
                return Err(format!("Expected end in key."));
            }
            Ok(DeclarationKey::ClassAndLocal(key, local))
        } else {
            if !reader.is_at_end() {
                return Err(format!("Expected end in key."));
            }
            Ok(DeclarationKey::Class(key))
        }
    } else if reader.is_parenthesized() {
        let key = reader.parse_parenthesized()?;
        if !reader.is_at_end() {
            return Err(format!("Expected end in key."));
        }
        Ok(DeclarationKey::Local(key))
    } else {
        return Err(format!("Invalid declaration key."));
    }
}

pub enum DeclarationKey {
    Class(String), ClassAndLocal(String, String), Local(String),
}

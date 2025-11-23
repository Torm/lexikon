use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use khi::{Dictionary, List, Tagged, Text, Value};
use khi::parse::pdm::{ParsedDictionary, ParsedTaggedValue, ParsedValue, Position};
use crate::article::{Article, ArticleElement, Class, Articles, verify_parameter_match};
use crate::relation::{RelationClass};
use crate::compile::template::{read_relation_list, read_relation_term_value, Templates};
use crate::makro::{MacroMap};
use crate::key::KeyReader;
use crate::tex::{write_tex_with, BreakMode};
use crate::{tex_error_to_text, tuple_split};
use crate::compile::name::read_names;
use crate::markup::{process_markup, Markup};

/// Read an article definition.
pub fn read_article<'a>(
    templates: &Templates,
    macros: &impl MacroMap,
    registry: &mut Articles,
    tag: &'a ParsedTaggedValue,
    at: Position,
    document_key: &str,
) -> Result<Rc<RefCell<Article>>, String> {
    let template_key = tag.name.to_string();
    let template = if let Some(template) = templates.get(&template_key) {
        template
    } else {
        return Err(format!("Template {} is not registered.", &template_key));
    };
    let (mut positionals, named) = tuple_split(tag.get());
    // Extract key.
    let (class_key, article_key) = if let Some(key) = remove_first(&mut positionals) {
        if !key.is_text() {
            return Err(format!("Key in article at {}:{} must be text.", at.line, at.column));
        }
        let key = key.as_text().unwrap().as_str();
        match read_article_key_declaration(key)? {
            DeclaredKey::Class(key) => (key.clone(), format!("{}@{}", &key, document_key)),
            DeclaredKey::ClassAndLocal(class_key, local_key) => (class_key, format!("{}@{}", &local_key, document_key)),
            DeclaredKey::Local(key) => (format!("{}@{}", &key, document_key), format!("{}@{}", &key, document_key)),
        }
    } else {
        return Err(format!("Expected first argument of key in article at {}:{}.", at.line, at.column));
    };
    let class_key: Rc<str> = class_key.as_str().into();
    let article_key: Rc<str> = article_key.as_str().into();
    // Extract names.
    let (names, parameters) = if let Some(names) = remove_first(&mut positionals) {
        read_names(macros, names)?
    } else {
        return Err(format!("Expected second argument of names in article at {}:{}.", at.line, at.column));
    };
    // Template class-style
    let style = if let Some(style) = &template.style {
        Some(style.clone())
    } else {
        None
    };
    // Register class if it does not exist. If it exists, verify that the parameters match.
    if let Some(c) = registry.class_map.get(&class_key) {
        let class = c.borrow();
        if let Err(e) = verify_parameter_match(&class.parameters, &parameters) {
            return Err(format!("Article at {}:{} did not match parameters with class {}.", at.line, at.column, &class_key));
        }
    } else {
        let class = Class {
            key: class_key.clone(),
            parameters,
            articles: vec![],
            relations: HashSet::new(),
            style,
        };
        registry.class_map.insert(class_key.clone(), Rc::new(RefCell::new(class)));
    };
    let class = registry.get_class(&class_key).unwrap().clone();
    // Extract content if it is defined.
    let content = if let Some(content) = remove_first(&mut positionals) {
        process_article_content(macros, content)?
    } else {
        vec![]
    };
    // Extract relations if they are defined.
    let this_rel_class = RelationClass::Name(class_key.clone());
    let mut relations = vec![];
    if let Some(relation_templates) = remove_first(&mut positionals) {
        if !relation_templates.is_list() {
            return Err(format!("Relations (arg 3) in article at {}:{} must be a list.", at.line, at.column));
        }
        let relation_templates = relation_templates.as_list().unwrap();
        let relation_templates = read_relation_list(relation_templates)?;
        for relation_template in relation_templates {
            let relation = relation_template.realize(&this_rel_class, None)?;
            relations.push(relation);
        }
    }
    // Instantiate template default relations.
    for default_rel in template.default_relations.iter() {
        let relation = default_rel.realize(&this_rel_class, None)?;
        relations.push(relation);
    }
    // Handle arguments to template.
    for (k, v) in named {
        if let Some(argument_relations) = template.argument_relations.get(k) {
            if !v.is_list() {
                return Err(format!("Template argument at {}:{} must be a list.", at.line, at.column));
            }
            let args = v.as_list().unwrap();
            for arg in args.iter() {
                let arg_class = read_relation_term_value(arg)?;
                let arg_rel_class = arg_class.realize(&this_rel_class, None)?;
                for argument_relation in argument_relations {
                    let relation = argument_relation.realize(&this_rel_class, Some(&arg_rel_class))?;
                    relations.push(relation);
                }
            }
        } else {
            return Err(format!("Template does not have argument {} at {}:{}.", k, at.line, at.column));
        }
    }
    // Check all arguments taken.
    if positionals.len() != 0 {
        return Err(format!("More arguments than expected in article at {}:{}.", at.line, at.column));
    }
    // Register article. If it exists, create a separator. // TODO: Warn behind document flag/pragma
    if let Some(article) = registry.article_map.get(&article_key) {
        let mut iarticle = article.borrow_mut();
        iarticle.names.extend(names);
        iarticle.content.push(ArticleElement::LocalSeparator);
        iarticle.content.extend(content);
        eprintln!("[Warning] Article {} has multiple instances.", iarticle.key.as_ref()); // TODO PRAGMA
        Ok(article.clone())
    } else {
        let article = Article { key: article_key.clone(), class: Rc::downgrade(&class), names, content };
        let article = Rc::new(RefCell::new(article));
        registry.article_map.insert(article_key, article.clone());
        // Register article in class.
        {
            let mut class = class.borrow_mut();
            class.articles.push(Rc::downgrade(&article));
        }
        //
        Ok(article)
    }

}

fn remove_first<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.len() > 0 {
        Some(vec.remove(0))
    } else {
        None
    }
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

pub enum DeclaredKey {
    Class(String), ClassAndLocal(String, String), Local(String),
}

/// Read a declaration key.
///
/// Supports `pkey`, `(dkey)` and `pkey(dkey)`.
pub fn read_article_key_declaration(declaration: &str) -> Result<DeclaredKey, String> {
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
            Ok(DeclaredKey::ClassAndLocal(key, local))
        } else {
            if !reader.is_at_end() {
                return Err(format!("Expected end in key."));
            }
            Ok(DeclaredKey::Class(key))
        }
    } else if reader.is_parenthesized() {
        let key = reader.parse_parenthesized()?;
        if !reader.is_at_end() {
            return Err(format!("Expected end in key."));
        }
        Ok(DeclaredKey::Local(key))
    } else {
        return Err(format!("Invalid declaration key."));
    }
}

/// Read the body of an article.
pub fn process_article_content(macros: &impl MacroMap, input: &ParsedValue) -> Result<Vec<ArticleElement>, String> {
    let mut article_elements = vec![];
    if input.is_list() {
        let content = input.as_list().unwrap();
        for c in content.iter() {
            if !c.is_tagged() {
                // return Err(format!("Element of article content list at {}:{} must be a tagged value", c.from().line, c.from().column));
                // TODO: Below is very temporary workaround.
                let txt = process_markup(macros, c)?;
                article_elements.push(ArticleElement::Paragraph(Markup::raw(&format!(r#"<p>{}</p>"#, &txt.0)))); // TODO Use Paragraph but wrap in div not p?
                continue;
            }
            let tag = c.as_tagged().unwrap();
            let (tuple, opts) = tuple_split(tag.get());
            let name = tag.name.as_ref();
            if name == "H1" || name == "H2" || name == "H3" || name == "H4" || name == "H5" || name == "H6" { // TODO: Not allowed, + check levels, check in compilation?
                let level = match name {
                    "H1" => return Err(format!("H1 heading not allowed in article at {}:{}.", input.from().line, input.from().column)),
                    "H2" => 2, "H3" => 3, "H4" => 4, "H5" => 5, "H6" => 6,
                    _ => unreachable!(),
                };
                let tex = process_markup(macros, tuple.get(0).unwrap())?;
                article_elements.push(ArticleElement::Heading { level, markup: tex });
            } else if name == "P" {
                let tex = process_markup(macros, tuple.get(0).unwrap())?;
                article_elements.push(ArticleElement::Paragraph(tex));
            } else if name == "$$" {
                let tex = write_tex_with(tuple.get(0).unwrap(), macros, BreakMode::Never).or_else(tex_error_to_text)?;
                let tex = format!("\\[{tex}\\]");
                let tex = Markup::raw(&tex);
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
        let tex = process_markup(macros, input)?;
        article_elements.push(ArticleElement::Paragraph(tex));
    }
    Ok(article_elements)
}

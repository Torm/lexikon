use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::article::{Article, ArticleElement, Articles, Class};
use serde_json::{Value as JsonValue, Map as JsonMap};
use crate::name::NameElement;

/// Write class files to the class directory.
pub(crate) fn write_class_directory(root_path: &Path, classes: &Articles) -> Result<(), String> {
    let class_dir_path = root_path.join("classes");
    if let Err(_) = fs::create_dir(&class_dir_path) {
        return Err(format!("Error creating class directory {}.", class_dir_path.to_str().unwrap())); // Create the temporary class directory.
    }
    // Write articles.
    for (_, class) in classes.get_classes().iter() {
        let class = class.borrow();
        write_class_data_file(&class_dir_path, &class)?;
        write_class_page_file(&class_dir_path, &class)?;
    }
    Ok(())
}

/// Write or update a class file.
fn write_class_data_file(class_dir_path: &Path, class: &Class) -> Result<(), String> {
    let class_key = class.key.as_ref();
    let class_file_name = format!("{}.json", class_key);
    let class_path = class_dir_path.join(&class_file_name);
    let class_data = generate_class_json(class)?;
    let mut file = File::create(&class_path).unwrap();
    if let Err(_) = file.write_all(class_data.as_bytes()) {
        return Err(format!("Error writing to class file {}.", class_path.to_str().unwrap()));
    }
    Ok(())
}

fn write_class_page_file(class_dir_path: &Path, class: &Class) -> Result<(), String> {
    Ok(()) // TODO
}

/// Generate class json, which contains entries "parameters", "style", "articles" and "relations".
pub fn generate_class_json(class: &Class) -> Result<String, String> {
    let mut class_json = JsonMap::new();
    // Write parameters.
    if !class.parameters.is_empty() {
        let mut params = vec![];
        for parameter in &class.parameters {
            params.push(JsonValue::String(parameter.to_string()));
        }
        class_json.insert("parameters".to_string(), JsonValue::Array(params));
    }
    // Write style.
    if let Some(style) = &class.style {
        class_json.insert("style".into(), JsonValue::String(style.to_string()));
    }
    // Write articles.
    let mut articles_json = JsonMap::new();
    for article in class.articles.iter() {
        let article = article.upgrade().unwrap();
        let article = article.borrow();
        let article_key = article.key.clone();
        let article_json = generate_article_json(&article);
        articles_json.insert(article_key.to_string(), JsonValue::Object(article_json));
    }
    class_json.insert("articles".into(), JsonValue::Object(articles_json));
    // Write relations.
//    generate_class_relations(class);
    //
    if let Ok(json) = serde_json::to_string(&class_json) {
        Ok(json)
    } else {
        Err(format!("Error converting class to JSON."))
    }
}

/// Generate article json, which contains entries "names" and "content".
fn generate_article_json(article: &Article) -> JsonMap<String, JsonValue> {
    let mut article_json = JsonMap::new();
    // Names
    let mut names_json = vec![];
    for name in &article.names {
        let mut name_json = vec![];
        for ne in name {
            let element = match ne {
                NameElement::Name(name) => {
                    JsonValue::Array(vec![JsonValue::String(name.0.clone()), JsonValue::String(String::from(""))])
                }
                NameElement::Preposition(markup) => {
                    JsonValue::String(markup.0.clone())
                }
                NameElement::Parameter { markup, class } => {
                    JsonValue::Array(vec![JsonValue::String(markup.0.clone()), JsonValue::String(class.to_string())])
                }
            };
            name_json.push(element);
        }
        names_json.push(JsonValue::Array(name_json));
    }
    article_json.insert("names".into(), JsonValue::Array(names_json));
    // Content
    let article_elements = article.content.as_slice();
    let mut content = vec![];
    generate_article_content(&mut content, article_elements);
    let content = String::from_utf8(content).unwrap();
    article_json.insert("content".into(), JsonValue::String(content)); // TODO: Allow content entry to be Array?
    article_json
}

//fn generate_class_relations(class: &Class) -> JsonMap<String, JsonValue> {
//
//}

pub fn generate_class_page(class: &Class) -> Result<String, String> {
    Ok(String::new()) // TODO
}

pub(crate) fn generate_article_content(html: &mut Vec<u8>, content: &[ArticleElement]) {
    for element in content {
        match element {
            ArticleElement::Heading { level, markup } => {
                html.extend_from_slice(&format!("<h{level}>{}</h{level}>", markup.0.as_str()).as_bytes());
            }
            ArticleElement::Paragraph(text) => {
                html.extend_from_slice(&format!(r#"{}"#, text.0.as_str()).as_bytes());
            }
            ArticleElement::LocalSeparator => {
                html.extend_from_slice("<hr>".as_bytes());
            }
        }
    }
}

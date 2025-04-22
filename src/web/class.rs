
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use crate::compile::class::{Article, Class};
use crate::web::document::generate_article_content;
use crate::web::json_map_set_string;

pub fn generate_class_data(class: &Class) -> Result<String, String> {
    let mut class_json = JsonMap::new();
    // Write type.
    let type_key = &class.article_type.key;
    json_map_set_string(&mut class_json, "type", type_key);
    // Write articles.
    let mut articles_json = JsonMap::new();
    for article in class.articles.borrow().iter() {
        let article_key = article.key.borrow().clone();
        let article_json = generate_article_json(article);
        articles_json.insert(article_key, JsonValue::Object(article_json));
    }
    class_json.insert("articles".into(), JsonValue::Object(articles_json));
    // Write links.
    let mut links_json = JsonMap::new();
    for (link_type, linked_classes) in class.links_out.borrow().iter() {
        let link_type_key = link_type.key.clone();
        let mut linked_classes_keys_json = vec![];
        for linked_class in linked_classes.iter() {
            let linked_class_key = JsonValue::String(linked_class.key.to_string());
            linked_classes_keys_json.push(linked_class_key);
        }
        links_json.insert(link_type_key, JsonValue::Array(linked_classes_keys_json));
    }
    for (link_type, linked_classes) in class.links_in.borrow().iter() {
        if !link_type.target_show { // TODO: Include in JSON, but don't generate links in JS.
            continue;
        }
        let link_type_key = format!("{}:{}", &link_type.article_type.key, &link_type.key);
        let mut linked_classes_keys_json = vec![];
        for linked_class in linked_classes.iter() {
            let linked_class_key = JsonValue::String(linked_class.key.to_string());
            linked_classes_keys_json.push(linked_class_key);
        }
        links_json.insert(link_type_key, JsonValue::Array(linked_classes_keys_json));
    }
    class_json.insert("links".into(), JsonValue::Object(links_json));
    //
    if let Ok(json) = serde_json::to_string(&class_json) {
        Ok(json)
    } else {
        Err(format!("Error converting class to JSON."))
    }
}

fn generate_article_json(article: &Article) -> JsonMap<String, JsonValue> {
    let mut article_json = JsonMap::new();
    let mut names_json = vec![];
    for name in &article.names {
        let name_json = JsonValue::String(name.clone());
        names_json.push(name_json);
    }
    article_json.insert("names".into(), JsonValue::Array(names_json));
    let article_elements = article.content.as_slice();
    let mut content = vec![];
    generate_article_content(&mut content, article_elements);
    let content = String::from_utf8(content).unwrap();
    article_json.insert("content".into(), JsonValue::String(content)); // TODO: Allow content entry to be Array?
    article_json
}

pub fn generate_class_page(class: &Class) -> Result<String, String> {
    Ok(String::new()) // TODO
}

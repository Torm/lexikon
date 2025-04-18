
use serde_json::Value as JsonValue;
use serde_json::Map as JsonMap;
use crate::compile::model::Model;
use super::json_map_set_string;

pub fn generate_model_data(model: &Model) -> Result<String, String> {
    let mut types_json = JsonMap::new();
    for (type_key, article_type) in model.type_map.borrow().iter() {
        let mut type_json = JsonMap::new();
        json_map_set_string(&mut type_json, "name", article_type.name.as_str());
        json_map_set_string(&mut type_json, "description", article_type.description.as_str());
        if let Some(abbreviation) = &article_type.abbreviation {
            json_map_set_string(&mut type_json, "abbreviation", abbreviation.as_str());
        }
        if let Some(colour) = &article_type.colour {
            json_map_set_string(&mut type_json, "colour", colour.as_str());
        }
        if article_type.links.borrow().len() > 0 {
            let mut links_json = JsonMap::new();
            for link in article_type.links.borrow().iter() {
                let mut link_json = JsonMap::new();
                json_map_set_string(&mut link_json, "origin.name", link.origin_name.as_str());
                json_map_set_string(&mut link_json, "origin.description", link.origin_description.as_str());
                json_map_set_string(&mut link_json, "target.name", link.target_name.as_str());
                json_map_set_string(&mut link_json, "target.description", link.target_description.as_str());
                json_map_set_string(&mut link_json, "target.show", link.target_show.to_string());
                links_json.insert(link.key.clone(), JsonValue::Object(link_json));
            }
            type_json.insert("links".into(), JsonValue::Object(links_json));
        }
        types_json.insert(type_key.clone(), JsonValue::Object(type_json));
    }
    match serde_json::to_string(&types_json) {
        Ok(j) => Ok(j),
        Err(_e) => Err(format!("Error converting model to JSON.")),
    }
}

pub fn generate_model_css(model: &Model) -> Result<String, String> {
    let mut css = String::new();
    for (type_key, article_type) in model.type_map.borrow().iter() {
        css.push_str(&format!(".{}-type{{", type_key));
        let default_colour = String::from("#353535");
        let colour = article_type.colour.as_ref().unwrap_or(&default_colour);
        css.push_str(&format!("--type-colour:{};", colour));
        css.push('}');
    }
    Ok(css)
}

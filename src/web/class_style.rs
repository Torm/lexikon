use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::style::Styles;
use crate::web::json_map_set_string;
use serde_json::{Value as JsonValue, Map as JsonMap};

const DEFAULT_COLOUR: &str = "#353535";

pub fn generate_class_style_json(styles: &Styles) -> Result<String, String> {
    let mut styles_json = JsonMap::new();
    for (style_key, style) in styles {
        let mut style_json = JsonMap::new();
        let name = style.name.as_str();
        json_map_set_string(&mut style_json, "name", style.name.as_str());
        if let Some(description) = &style.description {
            json_map_set_string(&mut style_json, "description", description.as_str());
        }
        if let Some(abbreviation) = &style.abbreviation {
            json_map_set_string(&mut style_json, "abbreviation", abbreviation.as_str());
        }
        if let Some(colour) = &style.colour {
            json_map_set_string(&mut style_json, "colour", colour.as_str());
        }
        styles_json.insert(name.to_string(), JsonValue::Object(style_json));
    }
    match serde_json::to_string(&styles_json) {
        Ok(j) => Ok(j),
        Err(_e) => Err(format!("Error converting style to JSON.")),
    }
}

pub fn generate_class_style_css(styles: &Styles) -> Result<String, String> {
    let mut css = String::new();
    for (style_key, style) in styles {
        let name = style.name.as_str();
        css.push_str(&format!(".{}-style{{", name));
        if let Some(colour) = &style.colour {
            css.push_str(&format!("--type-colour:{};", colour.as_str()));
        } else {
            css.push_str(&format!("--type-colour:{};", DEFAULT_COLOUR));
        }
        css.push('}');
    }
    Ok(css)
}

/// Write or update the model file.
pub fn write_class_style_json_file(root_path: &Path, styles: &Styles) -> Result<(), String> {
    let file_path = root_path.join("style.json");
    let mut file = File::create_new(&file_path).or(
        Err(format!("Error creating style.json file {}.", file_path.to_str().unwrap()))
    )?;
    let style_json = generate_class_style_json(styles)?;
    file.write_all(style_json.as_bytes()).or(
        Err(format!("Error writing to style.json file {}.", file_path.to_str().unwrap()))
    )?;
    Ok(())
}

pub fn write_class_style_css_file(root_path: &Path, styles: &Styles) -> Result<(), String> {
    let styles_css_path = root_path.join("style.css");
    let mut file = File::create_new(&styles_css_path).or(
        Err(format!("Error creating style.css file {}.", styles_css_path.to_str().unwrap()))
    )?;
    let style_css = generate_class_style_css(styles)?;
    file.write_all(style_css.as_bytes()).or(
        Err(format!("Error writing to style.css file {}.", styles_css_path.to_str().unwrap()))
    )?;
    Ok(())
}

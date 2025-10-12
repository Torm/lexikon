use std::path::Path;
use serde_json::{Value as JsonValue, Map as JsonMap};

pub mod class;
pub mod asset;
pub mod class_style;
pub mod document;
//pub mod dirpage;
pub mod index;
//mod name;

pub fn include_index_and_icon(root_path: &Path) -> Result<(), String> {
    Ok(()) // TODO
}

pub(crate) fn json_map_set_string(map: &mut JsonMap<String, JsonValue>, key: impl Into<String>, value: impl Into<String>) {
    let key = key.into();
    let value = JsonValue::String(value.into());
    map.insert(key, value);
}

// fn replace_tokens(out: &mut Vec<u8>, mut inn: &[u8], tokens: &[(&[u8], fn(&mut Vec<u8>, &[u8]) -> &[u8])]) {
//     'charloop: while inn.len() > 0 {
//         for (token, action) in tokens {
//             if inn.starts_with(token) {
//                 inn = &inn[token.len()..];
//                 inn = action(out, inn);
//                 continue 'charloop;
//             }
//         }
//         out.push(inn[0]);
//         inn = &inn[1..];
//     }
// }

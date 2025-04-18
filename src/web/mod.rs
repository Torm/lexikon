//! Generation of web content.

use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;

pub mod class;
pub mod document;
pub mod model;

fn json_map_set_string(map: &mut JsonMap<String, JsonValue>, key: impl Into<String>, value: impl Into<String>) {
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

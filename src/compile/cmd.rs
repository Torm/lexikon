// use std::collections::HashMap;
// use khi::pdm::ParsedValue;
// use khi::{Text, Value};
//
// pub struct DefinedCommand {
//     parameters: u8,
//
// }
//
// pub struct MacroPreprocessor {
//     commands: HashMap<String, DefinedCommand>,
// }
//
// impl MacroPreprocessor {
//
//     fn preprocess(&mut self, value: &ParsedValue) -> Result<String, String> {
//         self.preprocess_with_arguments()
//     }
//
//
//     fn preprocess_with_arguments(&mut self, value: &ParsedValue, arguments: &[String]) {
//         let mut result = String::new();
//         match value {
//             ParsedValue::Tagged(tagged, _, _) => {
//                 let name = tagged.name.as_ref();
//                 if let Ok(num) = name.parse::<usize>() {
//                     if let Some(argument) = arguments.get(num) {
//
//                     } else {
//
//                     }
//                 } else if let Some(command) = self.commands.get(name) {
//                     let parameters = command.parameters;
//                     if tagged.value.len_as_tuple() != parameters as usize {
//                         return Err(format!("", ));
//                     }
//                 } else {
//
//                 }
//             }
//             value => {
//                 match khi::tex::write_tex() {
//                     Ok(v) => {}
//                     Err(e) => return Err(format!("", )),
//                 }
//             }
//         }
//         result
//     }
//
// }

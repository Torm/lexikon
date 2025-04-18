//! Text content & commands

use khi::pdm::ParsedValue;
use khi::tex::{write_tex_with, BreakMode};
use khi::{Compound, Element, Tagged, Text, Tuple, Value};
use crate::tex_error_to_text;

pub fn read_content_text(input: &ParsedValue) -> Result<String, String> {
    let mut output = String::new();
    convert_content_text(&mut output, input)?;
    Ok(output)
}

fn convert_content_text(output: &mut String, input: &ParsedValue) -> Result<(), String> {
    match input {
        ParsedValue::Text(s, _, _) => Ok(output.push_str(s.as_str())),
        ParsedValue::Tagged(t, _, _) => {
            let name = t.name.as_ref();
            if name == "$" {
                let tex = write_tex_with(t.value.as_ref(), BreakMode::Never).or_else(tex_error_to_text)?;
                output.push('\\');
                output.push('(');
                output.push_str(&tex);
                output.push('\\');
                output.push(')');
                Ok(())
            } else if name == "$$" {
                let tex = write_tex_with(t.value.as_ref(), BreakMode::Never).or_else(tex_error_to_text)?;
                output.push('\\');
                output.push('[');
                output.push_str(&tex);
                output.push('\\');
                output.push(']');
                Ok(())
            } else if name == "n" {
                match t.value.as_ref() {
                    ParsedValue::Tuple(t, ..) => {
                        if t.is_empty() {
                            output.push_str("<br>");
                        } else if t.len() == 1 {
                            return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", input.from().line, input.from().column));
                        } else {
                            let mut iter = t.iter();
                            let fv = iter.next().unwrap();
                            convert_content_text(output, fv)?;
                            for v in iter {
                                output.push_str("<br>");
                                convert_content_text(output, v)?;
                            }
                        }
                    }
                    _ => return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", t.value.from().line, t.value.from().column)),
                }
                Ok(())
            } else if name == "@" {
                if t.value.len_as_tuple() != 2 {
                    return Err(format!("Link at {}:{} must have a href argument and a label argument.", t.value.from().line, t.value.from().column))
                }
                let values = t.value.as_tuple().unwrap();
                let href = values.get(0).unwrap().as_text().unwrap().as_str();
                let label = read_content_text(values.get(1).unwrap())?;
                output.push_str(&format!("<a href=\"{href}\">{label}</a>"));
                Ok(())
            } else if name == "code" {
                let code = t.get();
                if !code.is_text() {
                    return Err(format!("Code at {}:{} must be text.", t.value.from().line, t.value.from().column))
                }
                let code = code.as_text().unwrap().as_str();
                output.push_str(&format!("<pre><code>{}</code></pre>", code));
                Ok(())
            } else if name == "icode" {
                let code = t.get();
                if !code.is_text() {
                    return Err(format!("Code at {}:{} must be text.", t.value.from().line, t.value.from().column))
                }
                let code = code.as_text().unwrap().as_str();
                output.push_str(&format!("<code>{}</code>", code));
                Ok(())
            } else {
                Err(format!("Unexpected command in content text at {}:{}.", input.from().line, input.from().column))
            }
        }
        ParsedValue::Tuple(_, _, _) => return Err(format!("Unexpected tuple in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::Dictionary(_, _, _) => return Err(format!("Unexpected dictionary in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::List(_, _, _) => return Err(format!("Unexpected list in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::Compound(c, _, _) => {
            for c in c.iter() {
                match c {
                    Element::Element(e) => convert_content_text(output, e)?,
                    Element::Whitespace => output.push(' '),
                }
            }
            Ok(())
        }
        ParsedValue::Nil(..) => Ok(()),
    }
}

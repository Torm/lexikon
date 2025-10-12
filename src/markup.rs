use khi::parse::pdm::ParsedValue;
use khi::{Catenation, Element, Tagged, Text, Tuple, Value};
use crate::makro::{MacroMap};
use crate::tex::{write_tex_with, BreakMode};
use crate::{tex_error_to_text, tuple_split};
// TODO: Processing markup produces LaTeX/HTML, which should maybe be done in the web package. But this works for now since this is the only option.

#[derive(Clone)]
pub struct Markup(pub String);

impl Markup {

    pub fn from_markup(macros: &impl MacroMap, markup: &ParsedValue) -> Result<Self, String> {
        process_markup(macros, markup)
    }

    pub fn raw(str: &str) -> Self {
        Self(str.to_string())
    }

}

pub fn process_markup(macros: &impl MacroMap, input: &ParsedValue) -> Result<Markup, String> {
    let mut output = String::new();
    process_markup_text(&mut output, macros, input)?;
    Ok(Markup::raw(&output))
}

fn process_markup_text(output: &mut String, macros: &impl MacroMap, input: &ParsedValue) -> Result<(), String> {
    match input {
        ParsedValue::Text(s, _, _) => Ok(output.push_str(s.as_str())),
        ParsedValue::Tagged(t, _, _) => {
            let name = t.name.as_ref();
            let (poss, opts) = tuple_split(t.get());
            if name == "$" {
                let tex = write_tex_with(poss.get(0).unwrap(), macros, BreakMode::Never).or_else(tex_error_to_text)?;
                output.push('\\');
                output.push('(');
                output.push_str(&tex);
                output.push('\\');
                output.push(')');
                Ok(())
            } else if name == "$$" {
                let tex = write_tex_with(poss.get(0).unwrap(), macros, BreakMode::Never).or_else(tex_error_to_text)?;
                output.push('\\');
                output.push('[');
                output.push_str(&tex);
                output.push('\\');
                output.push(']');
                Ok(())
            } else if name == "n" {
                match poss.get(0).unwrap() {
                    ParsedValue::Tuple(t, ..) => {
                        if t.is_empty() {
                            output.push_str("<br>");
                        } else if t.len() == 1 {
                            return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", input.from().line, input.from().column));
                        } else {
                            let mut iter = poss.iter();
                            let fv = iter.next().unwrap();
                            process_markup_text(output, macros, fv)?;
                            for v in iter {
                                output.push_str("<br>");
                                process_markup_text(output, macros, v)?;
                            }
                        }
                    }
                    _ => return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column)),
                }
                Ok(())
            } else if name == "@" {
                if t.get().len() != 2 {
                    return Err(format!("Link at {}:{} must have a href argument and a label argument.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column))
                }
                let href = poss.get(0).unwrap().as_text().unwrap().as_str();
                let label = process_markup(macros, poss.get(1).unwrap())?;
                output.push_str(&format!("<a href=\"{href}\">{}</a>", &label.0));
                Ok(())
            } else if name == "code" {
                if poss.len() != 1 {
                    return Err(format!("<code> takes 1 text argument."));
                }
                let code = poss.get(0).unwrap();
                if !code.is_text() {
                    return Err(format!("Code at {}:{} must be text.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column))
                }
                let code = code.as_text().unwrap().as_str();
                output.push_str(&format!("<pre><code>{}</code></pre>", code));
                Ok(())
            } else if name == "icode" {
                if poss.len() != 1 {
                    return Err(format!("<code> takes 1 text argument."));
                }
                let code = poss.get(0).unwrap();
                if !code.is_text() {
                    return Err(format!("Code at {}:{} must be text.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column))
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
        ParsedValue::Catenation(c, _, _) => {
            for c in c.iter() {
                match c {
                    Element::Element(e) => process_markup_text(output, macros, e)?,
                    Element::Whitespace => output.push(' '),
                }
            }
            Ok(())
        }
        ParsedValue::Nil(..) => Ok(()),
    }
}

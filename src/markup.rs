use std::rc::Rc;
use khi::parse::pdm::{ParsedCatenation, ParsedTag, ParsedTaggedTuple, ParsedText, ParsedTupleElement, ParsedValue, Position};
use khi::{Catenation, Element, TaggedTuple, Text, Value};
use crate::makro::{MacroMap};
use crate::tex::{write_tex_with, BreakMode};
use crate::{tex_error_to_text, tuple_split};
use crate::preprocess_markup::{process_markup_level, process_unexpanded_markup};
// TODO: Processing markup produces LaTeX/HTML, which should maybe be done in the web package. But this works for now since this is the only option.

#[derive(Clone)]
pub struct Markup(pub String);

impl Markup {

    pub fn from_markup(macros: &impl MacroMap, markup: &ParsedValue) -> Result<Self, String> {
        process_unexpanded_markup(macros, markup)
    }

    pub fn raw(str: &str) -> Self {
        Self(str.to_string())
    }

}

pub(crate) fn process_article_markup_text(output: &mut String, macros: &impl MacroMap, text: &ParsedText) -> Result<(), String> {
    output.push_str(text.as_str());
    Ok(())
}

pub(crate) fn process_article_markup_tag(output: &mut String, macros: &impl MacroMap, tag: &ParsedTaggedTuple, from: Position) -> Result<(), String> {
    let name = if let Some(name) = tag.name() {
        name
    } else {
        return Err(format!("Empty tag name at {}:{}", from.line, from.column))
    };
    let (poss, opts) = tuple_split(tag);
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
        if poss.len() == 0 {
            output.push_str("<br>");
        } else {
            match poss.get(0).unwrap() {
                ParsedValue::TaggedTuple(t, ..) => {
                    if t.is_empty() {
                        output.push_str("<br>");
                    } else if t.len() == 1 {
                        return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", from.line, from.column));
                    } else {
                        let mut iter = poss.iter();
                        let fv = iter.next().unwrap();
                        process_markup_level(output, macros, fv)?;
                        for v in iter {
                            output.push_str("<br>");
                            process_markup_level(output, macros, v)?;
                        }
                    }
                }
                _ => return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column)),
            }
        }
        Ok(())
    } else if name == "link" {
        if tag.len() != 2 {
            return Err(format!("Link at {}:{} must have a href argument and a label argument.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column))
        }
        let href = poss.get(0).unwrap().as_text().unwrap().as_str();
        let label = process_unexpanded_markup(macros, poss.get(1).unwrap())?;
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
        let mut code = String::from(code.as_text().unwrap().as_str());
        code = code.replace("&", "&amp;");
        code = code.replace("<", "&lt;");
        code = code.replace(">", "&gt;");
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
        let mut code = String::from(code.as_text().unwrap().as_str());
        code = code.replace("&", "&amp;");
        code = code.replace("<", "&lt;");
        code = code.replace(">", "&gt;");
        output.push_str(&format!("<code>{}</code>", code));
        Ok(())
    } else if name == "raw!" {
        if poss.len() != 1 {
            return Err(format!("<raw!> takes 1 text argument."));
        }
        let raw = poss.get(0).unwrap();
        if !raw.is_text() {
            return Err(format!("Raw text at {}:{} must be text.", poss.get(0).unwrap().from().line, poss.get(0).unwrap().from().column))
        }
        output.push_str(raw.as_text().unwrap().as_str());
        Ok(())
    } else {
        Err(format!("Unexpected command in content text at {}:{}.", from.line, from.column))
    }
}

// fn escape_html() { // TODO
//     code = code.replace("&", "&amp;");
//     code = code.replace("<", "&lt;");
//     code = code.replace(">", "&gt;");
// }

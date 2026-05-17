use std::rc::Rc;
use khi::parse::pdm::{ParsedCatenation, ParsedTag, ParsedTaggedTuple, ParsedText, ParsedTupleElement, ParsedValue, Position};
use khi::{Catenation, Element, TaggedTuple, Text, Value};
use crate::makro::{MacroMap};
use crate::tex::{write_tex_with, BreakMode};
use crate::{tex_error_to_text, tuple_split};
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

pub fn process_unexpanded_markup(macros: &impl MacroMap, input: &ParsedValue) -> Result<Markup, String> {
    let mut output = String::new();
    process_markup_level(&mut output, macros, input)?;
    Ok(Markup::raw(&output))
}

pub fn process_markup_level(output: &mut String, macros: &impl MacroMap, input: &ParsedValue) -> Result<(), String> {
    let from = input.from();
    let to = input.to();
    let input = decompose_level(input);
    let input = recompose_level(&input, from, to)?;
    process_article_markup(output, macros, &input)
}

enum DecomposedValue {
    Value(ParsedValue),
    InlineMathDelimiter,
    Space,
}

fn decompose_level(input: &ParsedValue) -> Vec<DecomposedValue> { // TODO Implement value.iter_as_catenation()
    let mut components = vec![];
    // Decompose the components
    match input {
        ParsedValue::Text(text, from, to) => { // TODO Use blocks of escape characters instead of 1 escape
            decompose_text(&mut components, text, *from, *to)
        }
        ParsedValue::Catenation(catenation, from, to) => {
            for val in catenation.iter() {
                match val {
                    Element::Element(val) => {
                        match val {
                            ParsedValue::Text(text, from, to) => {
                                decompose_text(&mut components, text, *from, *to);
                            }
                            val => {
                                components.push(DecomposedValue::Value(val.clone()));
                            }
                        }
                    }
                    Element::Separator => {
                        components.push(DecomposedValue::Space);
                    }
                }
            }
        }
        value => {
            components.push(DecomposedValue::Value(value.clone()));
        }
    }
    components
}

fn decompose_text(components: &mut Vec<DecomposedValue>, text: &ParsedText, from: Position, to: Position) {
    let mut string = String::new();
    let mut chars = text.as_str().chars();
    while let Some(c) = chars.next() {
        if c == '$' {
            if !string.is_empty() {
                components.push(DecomposedValue::Value(ParsedValue::Text(ParsedText{ str: Rc::from(string.as_str()), escapes: vec![false; string.len()] }, from, to))); // TODO Fix location // TODO Fix escapes
                string.clear();
            }
            components.push(DecomposedValue::InlineMathDelimiter);
        } else {
            string.push(c);
        }
    }
    if !string.is_empty() {
        components.push(DecomposedValue::Value(ParsedValue::Text(ParsedText{ str: Rc::from(string.as_str()), escapes: vec![false; string.len()] }, from, to))); // TODO Fix location // TODO Fix escapes
    }
}

fn recompose_level(components: &Vec<DecomposedValue>, from: Position, to: Position) -> Result<ParsedValue, String> {
    let mut result = vec![];
    let mut iter = components.iter();
    while let Some(comp) = iter.next() {
        match comp {
            DecomposedValue::Value(value) => {
                result.push(Element::Element(value.clone()));
            }
            DecomposedValue::InlineMathDelimiter => {
                let mut mathcomps = vec![];
                'readmath: loop {
                    if let Some(mathcomp) = iter.next() {
                        match mathcomp {
                            DecomposedValue::InlineMathDelimiter => {
                                break 'readmath;
                            }
                            DecomposedValue::Value(value) => {
                                mathcomps.push(Element::Element(value.clone()));
                            }
                            DecomposedValue::Space => {
                                mathcomps.push(Element::Separator);
                            }
                        }
                    } else {
                        return Err(format!("Missing ending $."));
                    }
                }
                let inner = reassemble_cat(mathcomps, from, to)?; // TODO Fix from to for all
                // TODO RC
                result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple{ tag: Some(ParsedTag { name: Rc::from("$"), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(inner)] }, from, to)));
            }
            DecomposedValue::Space => {
                result.push(Element::Separator);
            }
        }
    }
    reassemble_cat(result, from, to)
}

fn reassemble_cat(mut components: Vec<Element<ParsedValue>>, from: Position, to: Position) -> Result<ParsedValue, String> {
    let inner = if components.len() == 1 {
        match components.pop().unwrap() {
            Element::Element(e) => e,
            Element::Separator => unreachable!(),
        }
    } else if components.len() == 0 {
        ParsedValue::Nil(from, to)
    } else {
        ParsedValue::Catenation(ParsedCatenation { components }, from, to)
    };
    Ok(inner)
}

fn process_article_markup(output: &mut String, macros: &impl MacroMap, input: &ParsedValue) -> Result<(), String> {
    match input {
        ParsedValue::Text(s, _, _) => process_article_markup_text(output, macros, s)?,
        ParsedValue::TaggedTuple(t, _, _) => process_article_markup_tag(output, macros, t, input.from())?,
        ParsedValue::Dictionary(_, _, _) => return Err(format!("Unexpected dictionary in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::List(_, _, _) => return Err(format!("Unexpected list in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::Catenation(c, _, _) => {
            for c in c.iter() {
                match c {
                    Element::Element(e) => {
                        match e {
                            ParsedValue::Text(s, _, _) => process_article_markup_text(output, macros, s)?,
                            ParsedValue::TaggedTuple(t, _, _) => process_article_markup_tag(output, macros, t, input.from())?,
                            ParsedValue::Dictionary(_, _, _) => return Err(format!("Unexpected dictionary in content text at {}:{}.", input.from().line, input.from().column)),
                            ParsedValue::List(_, _, _) => return Err(format!("Unexpected list in content text at {}:{}.", input.from().line, input.from().column)),
                            ParsedValue::Catenation(_, _, _) => process_markup_level(output, macros, e)?, // Inner level
                            ParsedValue::Nil(..) => {},
                        }
                    },
                    Element::Separator => output.push(' '),
                }
            }
        }
        ParsedValue::Nil(..) => {},
    }
    Ok(())
}

fn process_article_markup_text(output: &mut String, macros: &impl MacroMap, text: &ParsedText) -> Result<(), String> {
    output.push_str(text.as_str());
    Ok(())
}

fn process_article_markup_tag(output: &mut String, macros: &impl MacroMap, tag: &ParsedTaggedTuple, from: Position) -> Result<(), String> {
    let name = tag.name().unwrap();
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
        Ok(())
    } else if name == "@" {
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
        Err(format!("Unexpected command in content text at {}:{}.", from.line, from.column))
    }
}

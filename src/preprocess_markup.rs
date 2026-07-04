use std::rc::Rc;
use khi::{Catenation, Element, Text, TextType, Value};
use khi::parse::pdm::{ParsedCatenation, ParsedTag, ParsedTaggedTuple, ParsedText, ParsedTupleElement, ParsedValue, Position};
use crate::makro::MacroMap;
use crate::markup::Markup;

pub fn process_unexpanded_markup(macros: &impl MacroMap, input: &ParsedValue) -> Result<Markup, String> {
    let mut output = String::new();
    process_markup_level(&mut output, macros, input)?;
    Ok(Markup::raw(&output))
}

pub fn process_markup_level(output: &mut String, macros: &impl MacroMap, input: &ParsedValue) -> Result<(), String> {
    let input = preprocess_markup_level(input)?;
    process_article_markup(output, macros, &input)
}

pub fn preprocess_markup_level(input: &ParsedValue) -> Result<ParsedValue, String> {
    let from = input.from();
    let to = input.to();
    let input = decompose_level(input);
    recompose_level(&input, from, to)
}

enum DecomposedValue {
    /// (Value, bracketed)
    Value(ParsedValue, bool),
    InlineMathOperator,
    ParameterOperator,
    //CaretOperator, // TODO
    //UnderscoreOperator,
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
                    Element::Element(val, b) => {
                        match val {
                            ParsedValue::Text(text, from, to) if !b => {
                                decompose_text(&mut components, text, *from, *to);
                            }
                            val => {
                                components.push(DecomposedValue::Value(val.clone(), true));
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
            components.push(DecomposedValue::Value(value.clone(), true));
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
                components.push(DecomposedValue::Value(ParsedValue::Text(ParsedText { str: Rc::from(string.as_str()), text_type: TextType::Open }, from, to), false)); // TODO Fix location // TODO Fix escapes
                string.clear();
            }
            components.push(DecomposedValue::InlineMathOperator);
        } else if c == '@' {
            if !string.is_empty() {
                components.push(DecomposedValue::Value(ParsedValue::Text(ParsedText { str: Rc::from(string.as_str()), text_type: TextType::Open }, from, to), false)); // TODO Fix location // TODO Fix escapes
                string.clear();
            }
            components.push(DecomposedValue::ParameterOperator);
        } else {
            string.push(c);
        }
    }
    if !string.is_empty() {
        components.push(DecomposedValue::Value(ParsedValue::Text(ParsedText{ str: Rc::from(string.as_str()), text_type: TextType::Open }, from, to), false)); // TODO Fix location // TODO Fix escapes
    }
}

fn recompose_level(components: &Vec<DecomposedValue>, from: Position, to: Position) -> Result<ParsedValue, String> {
    let mut result = vec![];
    let mut iter = components.iter();
    while let Some(comp) = iter.next() {
        match comp {
            DecomposedValue::Value(value, b) => {
                result.push(Element::Element(value.clone(), false)); //TODO Bool
            }
            DecomposedValue::InlineMathOperator => {
                if let Some(e) = iter.next() {
                    match e {
                        DecomposedValue::Value(v, b) => {
                            match v {
                                ParsedValue::Text(t, from, to) if !b => {
                                    let s = t.as_str();
                                    if let Some((s1, s2)) = s.split_once(" ") {
                                        result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("$")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(ParsedValue::Text(ParsedText { str: Rc::from(s1), text_type: TextType::Open }, *from, *to))] }, *from, *to), false));
                                        result.push(Element::Separator);
                                        result.push(Element::Element(ParsedValue::Text(ParsedText { str: Rc::from(s2), text_type: TextType::Open }, *from, *to), false));
                                    } else {
                                        result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("$")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(ParsedValue::Text(ParsedText { str: Rc::from(s), text_type: TextType::Open }, *from, *to))] }, *from, *to), false));
                                    }
                                }
                                v => {
                                    result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("$")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(v.clone())] }, v.from(), v.to()), false));
                                }
                            }
                        }
                        DecomposedValue::InlineMathOperator => return Err(format!("<$> command (inline math op) cannot be nested.")),
                        DecomposedValue::ParameterOperator => return Err(format!("<$> command (inline math op) cannot contain parameters (<@> command).")),
                        DecomposedValue::Space => return Err(format!("<$> command (inline math op) must take a value, not a space.")),
                    }
                } else {
                    return Err(format!("<$> command (inline math op) must take a value."));
                }
                //let mut mathcomps = vec![];
                //'readmath: loop {
                //    if let Some(mathcomp) = iter.next() {
                //        match mathcomp {
                //            DecomposedValue::InlineMathOperator => {
                //                break 'readmath;
                //            }
                //            DecomposedValue::Value(value) => {
                //                mathcomps.push(Element::Element(value.clone(), false)); //TODO Bool
                //            }
                //            DecomposedValue::ParameterOperator => {
//
                //            }
                //            DecomposedValue::Space => {
                //                mathcomps.push(Element::Separator);
                //            }
                //        }
                //    } else {
                //        return Err(format!("Missing ending $."));
                //    }
                //}
                //let inner = reassemble_cat(mathcomps, from, to)?; // TODO Fix from to for all
                //// TODO RC
                //result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple{ tag: Some(ParsedTag { name: Rc::from("$"), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(inner)] }, from, to), false)); // TODO Brcaketed bool
            }
            DecomposedValue::ParameterOperator => {
                if let Some(e) = iter.next() {
                    match e {
                        DecomposedValue::Value(v, b) => {
                            match v {
                                ParsedValue::Text(t, from, to) if !b => {
                                    let s = t.as_str();
                                    if let Some((s1, s2)) = s.split_once(" ") {
                                        result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("@")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(ParsedValue::Text(ParsedText { str: Rc::from(s1), text_type: TextType::Open }, *from, *to))] }, *from, *to), false));
                                        result.push(Element::Separator);
                                        result.push(Element::Element(ParsedValue::Text(ParsedText { str: Rc::from(s2), text_type: TextType::Open }, *from, *to), false));
                                    } else {
                                        result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("@")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(ParsedValue::Text(ParsedText { str: Rc::from(s), text_type: TextType::Open }, *from, *to))] }, *from, *to), false));
                                    }
                                }
                                v => {
                                    match v {
                                        ParsedValue::TaggedTuple(tuple, _, _) => {
                                            if let Some(tag) = &tuple.tag {
                                                result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("@")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(v.clone())] }, v.from(), v.to()), false));
                                            } else {
                                                result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("@")), attributes: vec![] }), elements: tuple.elements.clone() }, v.from(), v.to()), false));
                                            }
                                        }
                                        _ => {
                                            result.push(Element::Element(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: Some(ParsedTag { name: Rc::from(("@")), attributes: vec![] }), elements: vec![ParsedTupleElement::Positional(v.clone())] }, v.from(), v.to()), false));
                                        }
                                    }
                                }
                            }
                        }
                        DecomposedValue::InlineMathOperator => {
                            todo!() // TODO Allow or require brackets?
                        }
                        DecomposedValue::ParameterOperator => return Err(format!("<@> command (name operator) cannot be nested.")),
                        DecomposedValue::Space => return Err(format!("<@> command (name operator) must take a value, not a space.")),
                    }
                } else {
                    return Err(format!("<@> command (name operator) must take a value."));
                }
            }
            DecomposedValue::Space => {
                result.push(Element::Separator);
            },
        }
    }
    reassemble_cat(result, from, to)
}

fn reassemble_cat(mut components: Vec<Element<ParsedValue>>, from: Position, to: Position) -> Result<ParsedValue, String> {
    let inner = if components.len() == 1 {
        match components.pop().unwrap() {
            Element::Element(e, b) => e,
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
        ParsedValue::Text(s, _, _) => crate::markup::process_article_markup_text(output, macros, s)?,
        ParsedValue::TaggedTuple(t, _, _) => crate::markup::process_article_markup_tag(output, macros, t, input.from())?,
        ParsedValue::Dictionary(_, _, _) => return Err(format!("Unexpected dictionary in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::List(_, _, _) => return Err(format!("Unexpected list in content text at {}:{}.", input.from().line, input.from().column)),
        ParsedValue::Catenation(c, _, _) => {
            for c in c.iter() {
                match c {
                    Element::Element(e, b) => {
                        match e {
                            ParsedValue::Text(s, _, _) => crate::markup::process_article_markup_text(output, macros, s)?,
                            ParsedValue::TaggedTuple(t, _, _) => crate::markup::process_article_markup_tag(output, macros, t, input.from())?,
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

use std::rc::Rc;
use khi::{Catenation, Element, List, TaggedTuple, Value};
use khi::parse::pdm::ParsedValue;
use crate::article::{verify_parameter_match, Parameters};
use crate::name::{Name, NameElement};
use crate::makro::MacroMap;
use crate::markup::{Markup};
use crate::preprocess_markup::{preprocess_markup_level, process_unexpanded_markup};
use crate::tuple_split;

pub fn read_names(macros: &impl MacroMap, name_value: &ParsedValue) -> Result<(Vec<Name>, Parameters), String> {
    let mut names = Vec::new();
    if name_value.is_list() {
        let mut parameters = None;
        for name in name_value.as_list().unwrap().iter() {
            let (name, params) = read_name(macros, name)?;
            names.push(name);
            if let Some(parameters) = &parameters {
                if let Err(e) = verify_parameter_match(&parameters, &params) {
                    return Err(e);
                };
            } else {
                parameters = Some(params);
            }
        }
        Ok((names, parameters.unwrap()))
    } else {
        let (name, parameters) = read_name(macros, name_value)?;
        names.push(name);
        Ok((names, parameters))
    }
}

fn read_name(macros: &impl MacroMap, name_value: &ParsedValue) -> Result<(Name, Parameters), String> {
    let mut elements = vec![];
    let mut parameters = vec![];
    let name_value = preprocess_markup_level(name_value)?;
    match &name_value {
        ParsedValue::TaggedTuple(tuple, _, _) => {
            //if (tuple.tag.is_some()) {
            //    if tuple.is_empty() {
            //        elements.push(NameElement::Preposition(Markup::raw("?"))); // TODO: ? should be type name.
            //    } else {
            //        return Err(format!("Name must be empty tuple or compound of text, tags, nils and compounds."));
            //    }
            //} else {
            //    read_name_element(macros, &mut elements, &mut parameters, &name_value)?;
            //}
            read_name_element(macros, &mut elements, &mut parameters, &name_value)?;
        }
        ParsedValue::Nil(..) => {
            read_name_element(macros, &mut elements, &mut parameters, &name_value)?;
        }
        ParsedValue::Text(..) => {
            read_name_element(macros, &mut elements, &mut parameters, &name_value)?;
        }
        ParsedValue::Catenation(catenation, _, _) => {
            for i in catenation.iter() {
                match i {
                    Element::Element(i, b) => {
                        read_name_element(macros, &mut elements, &mut parameters, i)?;
                    }
                    Element::Separator => {
                        elements.push(NameElement::Preposition(Markup::raw(" ")));
                    }
                }
            }
        }
        ParsedValue::Dictionary(..) | ParsedValue::List(..) => {
            return Err(format!("Name must be empty tuple or compound of text, tags, nils and compounds."));
        }
    }
    if elements.len() == 1 {
        let element = elements.pop().unwrap();
        if let NameElement::Preposition(name) = element {
            elements.push(NameElement::Name(name));
        } else {
            elements.push(element);
        }
    }
    Ok((elements, parameters.into_boxed_slice()))
}

fn read_name_element(macros: &impl MacroMap, parametrization: &mut Vec<NameElement>, parameters: &mut Vec<Rc<str>>, element: &ParsedValue) -> Result<(), String> {
    match element {
        ParsedValue::TaggedTuple(tagged, _, _) => {
            let name = tagged.name().unwrap();
            if name.starts_with("@") { // TODO: @1, @2, @3, ... - Reorderable parameters
                let name = name.strip_prefix("@").unwrap();
                let (argument, named) = tuple_split(tagged);
                if argument.len() == 1 {
                    let argument = argument.get(0).unwrap();
                    let text = process_unexpanded_markup(macros, argument)?;
                    parametrization.push(NameElement::Name(text));
                } else if argument.len() == 2 {
                    let argument = argument.get(1).unwrap();
                    let text = process_unexpanded_markup(macros, argument)?;
                    let key = Rc::from(name);
                    parametrization.push(NameElement::Parameter { markup: text, class: key });
                    parameters.push(Rc::from(name));
                } else {
                    return Err(format!("Parameter takes 1 or 2 arguments."));
                }
            } else {
                let text = process_unexpanded_markup(macros, element)?;
                parametrization.push(NameElement::Preposition(text));
            }
        }
        ParsedValue::Text(..) => {
            let text = process_unexpanded_markup(macros, element)?;
            parametrization.push(NameElement::Preposition(text));
        }
        ParsedValue::Catenation(..) | ParsedValue::Nil(..) => {
            let text = process_unexpanded_markup(macros, element)?;
            parametrization.push(NameElement::Preposition(text));
        }
        ParsedValue::Dictionary(..) | ParsedValue::List(..) => {
            return Err(format!("Name element cannot be dictionary or list."));
        }
    }
    Ok(())
}

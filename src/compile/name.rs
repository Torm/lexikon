use std::rc::Rc;
use khi::{Catenation, Element, List, Tagged, Tuple, Value};
use khi::parse::pdm::ParsedValue;
use crate::article::{verify_parameter_match, Parameters};
use crate::name::{Name, NameElement};
use crate::makro::MacroMap;
use crate::markup::{process_markup, Markup};
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
    match name_value {
        ParsedValue::Tuple(tuple, _, _) => {
            if tuple.is_empty() {
                elements.push(NameElement::Preposition(Markup::raw("?"))); // TODO: ? should be type name.
            } else {
                return Err(format!("Name must be empty tuple or compound of text, tags, nils and compounds."));
            }
        }
        ParsedValue::Nil(..) | ParsedValue::Text(..) | ParsedValue::Tagged(..) => {
            read_name_element(macros, &mut elements, &mut parameters, name_value)?;
        }
        ParsedValue::Catenation(catenation, _, _) => {
            for i in catenation.iter() {
                match i {
                    Element::Element(i) => {
                        read_name_element(macros, &mut elements, &mut parameters, i)?;
                    }
                    Element::Whitespace => {
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
        ParsedValue::Tagged(tagged, _, _) => {
            let name = tagged.name.as_ref();
            if name.starts_with("@") { // TODO: @1, @2, @3, ... - Reorderable parameters
                let name = name.strip_prefix("@").unwrap();
                let argument = tagged.get();
                let (argument, named) = tuple_split(argument);
                if argument.len() != 1 {
                    return Err(format!("Markup takes 1 argument."));
                }
                let argument = argument.get(0).unwrap();
                if name.is_empty() {
                    let text = process_markup(macros, argument)?;
                    parametrization.push(NameElement::Name(text));
                } else {
                    let text = process_markup(macros, argument)?;
                    let key = Rc::from(name);
                    parametrization.push(NameElement::Parameter { markup: text, class: key });
                    parameters.push(Rc::from(name));
                }
            } else {
                let text = process_markup(macros, element)?;
                parametrization.push(NameElement::Preposition(text));
            }
        }
        ParsedValue::Catenation(..) | ParsedValue::Nil(..) | ParsedValue::Text(..) => {
            let text = process_markup(macros, element)?;
            parametrization.push(NameElement::Preposition(text));
        }
        ParsedValue::Tuple(..) | ParsedValue::Dictionary(..) | ParsedValue::List(..) => {
            return Err(format!("Name element cannot be tuple, dictionary or list."));
        }
    }
    Ok(())
}

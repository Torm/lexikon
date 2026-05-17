use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use khi::{Catenation, Dictionary, Element, List, TaggedTuple, Text, Value};
use khi::parse::pdm::{ParsedDictionary, ParsedList, ParsedValue};
use crate::relation::{RelationClass, Relation};
use crate::file::{read_excludable_file_to_string, read_file_content_to_dictionary};

pub type Templates = HashMap<String, Template>;

pub struct Template {
    pub default_relations: Vec<TemplatedRelation>,
    pub argument_relations: HashMap<String, Vec<TemplatedRelation>>,
    pub style: Option<Rc<str>>,
}

pub fn read_template_file(templates: &mut Templates, path: &Path) -> Result<(), String> {
    let content = match read_excludable_file_to_string(path, "template")? {
        None => return Ok(()),
        Some(c) => c,
    };
    let dictionary = read_file_content_to_dictionary(path, "template", &content)?;
    read_template_dictionary(templates, &dictionary)?;
    Ok(())
}

/// Read a template.
pub fn read_template_dictionary(templates: &mut Templates, templates_dictionary: &ParsedDictionary) -> Result<(), String> {
    for (key, template) in templates_dictionary.iter() {
        if !template.is_dictionary() {
            return Err(format!("Template must be dictionary."));
        }
        let template = template.as_dictionary().unwrap();
        let mut default_relations = vec![];
        let mut argument_relations = HashMap::new();
        let mut style = None;
        for (parameter, value) in template.iter() {
            if parameter == "Default" {
                if !value.is_list() {
                    return Err(format!("Value of Default in template must be a list."));
                }
                let value = value.as_list().unwrap();
                let relation_templates = read_relation_list(value)?;
                default_relations.extend(relation_templates);
            } else if parameter == "Style" {
                if !value.is_text() {
                    return Err(format!("Value of Style in template must be text."));
                }
                let text = value.as_text().unwrap();
                let text = Rc::from(text.as_str());
                style = Some(text);
            } else {
                if !value.is_list() {
                    return Err(format!("Value of Arg in template must be a list."));
                }
                let value = value.as_list().unwrap();
                let templated_relations = read_relation_list(value)?;
                argument_relations.insert(parameter.to_string(), templated_relations);
            }
        }
        let template = Template { default_relations, argument_relations, style };
        templates.insert(key.to_string(), template);
    }
    Ok(())
}

/// A template relation: <left> is <right>. Can contain tokens such as <this>
/// or <arg> which must be instantiated before becoming a concrete relation.
pub struct TemplatedRelation {
    left: TemplatedClass,
    right: TemplatedClass,
}

/// A class partially (or fully) applied to some of its parameters.
pub enum TemplatedClass {
    This,
    /// Placeholder for
    Arg,
    Name(Rc<str>),
    Qual {
        name: Rc<str>, arguments: Box<[TemplatedClass]>
    }
}

impl TemplatedRelation {

    pub fn realize(&self, this: &RelationClass, arg: Option<&RelationClass>) -> Result<Relation, String> {
        let relation = Relation {
            left: self.left.realize(this, arg)?,
            right: self.right.realize(this, arg)?,
        };
        Ok(relation)
    }

}

impl TemplatedClass {

    pub fn realize(&self, this: &RelationClass, arg: Option<&RelationClass>) -> Result<RelationClass, String> {
        match self {
            TemplatedClass::This => Ok(this.clone()),
            TemplatedClass::Arg => if let Some(arg) = arg {
                Ok(arg.clone())
            } else {
                Err(format!("Found <arg> in TemplateClass, but the relation is not part of a template argument."))
            },
            TemplatedClass::Name(n) => Ok(RelationClass::Name(n.clone())),
            TemplatedClass::Qual { name, arguments } => {
                let mut parameters = vec![];
                for argument in arguments {
                    parameters.push(argument.realize(this, arg));
                }
                Ok(RelationClass::Name(name.clone()))
            },
        }
    }

}

pub fn read_relation_list(statements: &ParsedList) -> Result<Vec<TemplatedRelation>, String> {
    let mut relations = vec![];
    for statement in statements.iter() {
        let relation = read_relation_value(statement)?;
        relations.push(relation);
    }
    Ok(relations)
}

pub fn read_relation_value(statement: &ParsedValue) -> Result<TemplatedRelation, String> {
    let mut tokens = tokenize_relation_statement(statement)?;
    let left = parse_class(&mut tokens)?;
    parse_is(&mut tokens)?;
    let right = parse_class(&mut tokens)?;
    parse_end(&mut tokens)?;
    Ok(TemplatedRelation { left, right })
}

pub fn read_relation_term_value(qu: &ParsedValue) -> Result<TemplatedClass, String> {
    let mut tokens = tokenize_relation_statement(qu)?;
    let class = parse_class(&mut tokens)?;
    parse_end(&mut tokens)?;
    Ok(class)
}

fn parse_class(tokens: &mut Vec<Token>) -> Result<TemplatedClass, String> {
    if let Some(t0) = remove_first(tokens) {
        match t0 {
            Token::Class(class) => {
                if let Some(t1) = remove_first(tokens) {
                    if let Token::List(tl) = t1 {
                        let mut params = vec![];
                        for mut le in tl.into_iter() {
                            let parclass = parse_class(&mut le)?;
                            params.push(parclass);
                        }
                        Ok(TemplatedClass::Qual { name: Rc::from(class.as_str()), arguments: params.into_boxed_slice() })
                    } else {
                        tokens.insert(0, t1);
                        Ok(TemplatedClass::Name(Rc::from(class.as_str())))
                    }
                } else {
                    Ok(TemplatedClass::Name(Rc::from(class.as_str())))
                }
            }
            Token::Arg => Ok(TemplatedClass::Arg),
            Token::This => Ok(TemplatedClass::This),
            _ => Err(format!("Expected class but found list or keyword.")),
        }
    } else {
        Err(format!("Expected class but found nothing."))
    }
}

fn parse_is(tokens: &mut Vec<Token>) -> Result<(), String> {
    if let Some(token) = remove_first(tokens) {
        if matches!(token, Token::Is) {
            Ok(())
        } else {
            Err(format!("Expected \"is\" but found something else."))
        }
    } else {
        Err(format!("Expected \"is\" but found nothing."))
    }
}

fn parse_end(tokens: &mut Vec<Token>) -> Result<(), String> {
    if tokens.len() != 0 {
        return Err(format!("Expected end but found something."));
    }
    Ok(())
}

enum Token {
    Class(String), Is, List(Vec<Vec<Token>>), Arg, This,
}

fn tokenize_relation_statement(statement: &ParsedValue) -> Result<Vec<Token>, String> {
    let mut tokens = vec![];
    if statement.is_text() {
        let text = statement.as_text().unwrap();
        for word in text.as_str().split(" ") {
            if word.is_empty() {
                continue;
            } else if word == "is" {
                tokens.push(Token::Is);
            } else {
                tokens.push(Token::Class(String::from(word)));
            }
        }
    } else if statement.is_catenation() {
        let cat = statement.as_catenation().unwrap();
        for e in cat.iter() {
            if let Element::Element(f) = e {
                let es = tokenize_relation_statement(f)?;
                tokens.extend(es);
            }
        }
    } else if statement.is_tagged_tuple() {
        let tag = statement.as_tagged_tuple().unwrap();
        let name = if let Some(name) = tag.name() {
            name
        } else {
            return Err(format!("Tag name error at {}:{}", statement.from().line, statement.from().column));
        }; // TODO Remove this check?
        if name == "this" {
            tokens.push(Token::This);
        } else if name == "arg" {
            tokens.push(Token::Arg);
        } else {
            return Err(format!("Unknown tag: {}", name));
        }
    }
    Ok(tokens)
}

fn remove_first<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        None
    } else {
        Some(vec.remove(0))
    }
}

use std::fmt::Write;
use khi::{Catenation, Dictionary, Element, List, TaggedTuple, Text, TextType, Value};
use khi::parse::pdm::{ParsedList, ParsedTaggedTuple, ParsedText, ParsedValue, Position};
use crate::makro::{MacroMap};
use crate::{tuple_split};

pub struct Writer<'a, M: MacroMap> {
    pub(crate) output: &'a mut String,
    pub(crate) column: usize,
    pub(crate) last_type: LastType,
    /// Last line read in the source file.
    pub(crate) line: usize,
    pub(crate) break_mode: BreakMode,
    pub(crate) macros: &'a M,
}

#[derive(Eq, PartialEq)]
pub(crate) enum LastType {
    Newline,
    Whitespace,
    Glyph,
    Caret,
    Underscore,
    Command,
}

impl<M: MacroMap> Writer<'_, M> {
    pub(crate) fn push(&mut self, char: char) {
        if char.is_whitespace() {
            if self.last_type == LastType::Command {
                self.output.push('{');
                self.output.push('}');
                self.output.push(' ');
                self.last_type = LastType::Whitespace;
            } else if self.last_type == LastType::Caret || self.last_type == LastType::Underscore {
                //
            } else if self.last_type == LastType::Whitespace || self.last_type == LastType::Newline {
                //
            } else {
                self.output.push(' ');
                self.last_type = LastType::Whitespace;
                self.column += 1;
            }
        } else if char == '^' {
            self.contract();
            self.output.push('^');
            self.last_type = LastType::Caret;
            self.column += 1;
        } else if char == '_' {
            self.contract();
            self.output.push('_');
            self.last_type = LastType::Underscore;
            self.column += 1;
        } else {
            self.output.push(char);
            self.last_type = LastType::Glyph;
            self.column += 1;
        };
    }

    fn push_str(&mut self, str: &str) {
        for char in str.chars() {
            self.push(char);
        }
    }

    fn contract(&mut self) {
        while self.output.ends_with(' ') {
            self.output.pop();
        }
    }

    pub(crate) fn normalize_and_push_str(&mut self, str: &str) {
        for c in str.chars() {
            self.normalize_and_push_char(c);
        }
    }

    /// Normalize and push character.
    ///
    /// Normalization escapes the character as follows:
    /// - '%' must be inserted as "\%". % indicates a TeX comment.
    /// - '$' => '\$'
    /// - '^' must be inserted as "\^{}" or "\textasciicircum". ^ is the superscript operator in math mode, and reserved in text mode.
    /// - '_' must be inserted as "\_". _ is the subscript operator in math mode, and reserved in text mode.
    /// - '&' must be inserted as "\&". & is the tabulation operator.
    /// - '#' must be inserted as "\#". # is the argument substitution operator.
    /// - '\' must be inserted as "\textbackslash" in text and "\backslash" or "\setminus" in math. "\\" indicates a line break.
    /// - '~' -> "\texttildelow"
    /// - '{' -> "\{"
    /// - '}' -> "\}"
    pub(crate) fn normalize_and_push_char(&mut self, c: char) {
        if c == '$' {
            self.output.push('\\');
            self.output.push('$');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else if c == '%' {
            self.output.push('\\');
            self.output.push('%');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else if c == '&' {
            self.output.push('\\');
            self.output.push('&');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else if c == '^' {
            self.output.push_str("\\^{}");
            self.last_type = LastType::Glyph;
            self.column += 4;
        } else if c == '_' {
            self.output.push('\\');
            self.output.push('_');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else if c == '#' {
            self.output.push('\\');
            self.output.push('#');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else if c == '\\' {
            self.output.push_str("\\textbackslash");
            self.last_type = LastType::Glyph;
            self.column += 14;
        } else if c == '~' {
            self.output.push_str("\\texttildelow");
            self.last_type = LastType::Glyph;
            self.column += 13;
        } else if c == '{' {
            self.output.push('\\');
            self.output.push('{');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else if c == '}' {
            self.output.push('\\');
            self.output.push('}');
            self.last_type = LastType::Glyph;
            self.column += 2;
        } else {
            self.push(c);
        }
    }

    /// Insert a line break under the right conditions.
    pub(crate) fn break_opportunity(&mut self, position: Position) {
        let at_line = position.line;
        match self.break_mode {
            BreakMode::Never => {}
            BreakMode::Margin(margin) => {
                if margin < self.column {
                    if !matches!(self.last_type, LastType::Newline) {
                        self.contract();
                        self.output.push('\n');
                        self.line += 1;
                        self.last_type = LastType::Newline;
                        self.column = 1;
                    }
                }
            }
            BreakMode::Mirror => {
                if self.line < at_line {
                    if matches!(self.last_type, LastType::Newline) {
                        self.output.push_str("%\n");
                    } else {
                        self.contract();
                        self.output.push('\n');
                    }
                    self.line += 1;
                    self.last_type = LastType::Newline;
                    self.column = 1;
                    while self.line < at_line {
                        self.output.push_str("%\n");
                        self.line += 1;
                    }
                }
            }
        }
    }

    /// If a command with no arguments was last written, insert a space.
    pub(crate) fn separate_command(&mut self) {
        if self.last_type == LastType::Command {
            self.output.push(' ');
            self.last_type = LastType::Whitespace;
            self.column += 1;
        }
    }

    pub(crate) fn write_raw(&mut self, raw: &str) {
        for c in raw.chars() {
            self.output.push(c);
            if c == '\n' {
                self.line += 1;
            }
        }
    }
}

pub fn write_tex(structure: &ParsedValue, macros: &impl MacroMap) -> Result<String, PreprocessorError> {
    write_tex_with(structure, macros, BreakMode::Mirror)
}

pub fn write_tex_with(structure: &ParsedValue, macros: &impl MacroMap, mode: BreakMode) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = Writer { output: &mut output, column: 1, last_type: LastType::Whitespace, line: 1, break_mode: mode, macros };
    writer.write_inner(structure)?;
    Ok(output)
}

pub struct Preprocessor<'a, M: MacroMap> {
    pub(crate) writer: Writer<'a, M>,
    pub(crate) break_mode: BreakMode,
}

pub enum BreakMode {
    /// Do not insert newlines.
    Never,
    /// Convert spaces to newlines after reaching a margin.
    Margin(usize),
    // ///
    // TODO: BeforeMargin(usize)
    /// Convert spaces to newlines to mirror the input Khi document.
    Mirror
}

impl<M: MacroMap> Writer<'_, M> {

    /// Write text, respecting reserved and escaped characters.
    fn write_text(&mut self, text: &ParsedText) {
        let mut string = text.str.chars();
        let e = text.text_type == TextType::Escaped;
        while let Some(c) = string.next() {
            if e {
                self.normalize_and_push_char(c);
            } else {
                if c == '^' {
                    self.output.push('^');
                } else if c == '_' {
                    self.output.push('_');
                } else {
                    self.normalize_and_push_char(c);
                }
            }
        }
    }

    fn write_inner(&mut self, value: &ParsedValue) -> Result<(), PreprocessorError> {
        match value {
            ParsedValue::Nil(at, _) => {
                self.break_opportunity(*at);
                self.push('{');
                self.push('}');
            }
            ParsedValue::Text(text, at, _) => {
                self.break_opportunity(*at);
                self.write_text(text);
            }
            ParsedValue::Dictionary(_, at, _) => {
                return Err(PreprocessorError::IllegalDictionary(*at));
            }
            ParsedValue::List(table, at, _) => {
                self.break_opportunity(*at);
                self.write_tabulation(table, *at)?;
            }
            ParsedValue::Catenation(catenation, at, _) => {
                self.break_opportunity(*at);
                self.push('{');
                for element in catenation.iter() {
                    match element {
                        Element::Element(solid, b) => {
                            self.write_inner(solid);
                        }
                        Element::Separator => {
                            self.break_opportunity(*at);
                            self.push(' ');
                        }
                    }
                };
                self.push('}');
            }
            ParsedValue::TaggedTuple(tag, at, _) => {
                if (tag.name() == None) {
                    if (tag.len() == 0) {
                        self.break_opportunity(*at);
                        self.push('{');
                        self.push('}');
                    } else {
                        return Err(PreprocessorError::IllegalTuple(*at));
                    }
                } else {
                    self.break_opportunity(*at);
                    self.write_command(tag, *at)?;
                }
            }
        }
        Ok(())
    }

    fn write_tabulation(&mut self, list: &ParsedList, at: Position) -> Result<(), PreprocessorError> {
        for element in list.iter() {
            let (columns, opts) = if let Some(t) = element.as_tagged_tuple() {
                tuple_split(t)
            } else {
                (vec![element], vec![])
            };
            let mut columns = columns.iter();
            if let Some(c) = columns.next() {
                self.write_inner(&c)?;
            };
            while let Some(c) = columns.next() {
                self.push('&');
                self.write_inner(&c)?;
            };
            self.push('\\');
            self.push('\\');
        };
        Ok(())
    }

    /// A command may be a user-defined macro or a special built in.
    fn write_command(&mut self, tag: &ParsedTaggedTuple, at: Position) -> Result<(), PreprocessorError> {
        let mut name = tag.name().unwrap();
        let (tuple, opts) = tuple_split(tag);
        if let Some(m ) = self.macros.get(name) {
            if m.arity as usize != tuple.len() {
                return Err(PreprocessorError::MacroError(at, format!("Macro <{name}> takes {} arguments but got only {} arguments.", m.arity, tuple.len())));
            }
            let mut arguments = vec![];
            for &element in tuple.iter() {
                arguments.push(element);
            }
            let mut expansion = m.expansion.clone();
            match self.expand_value(&mut expansion, arguments.as_slice()) {
                Ok(..) => {}
                Err(e) => return Err(PreprocessorError::MacroError(at, e)),
            };
            self.output.push('{');
            self.write_inner(&expansion)?;
            self.output.push('}');
        } else if let Ok(num) = name.parse::<u8>() {
            self.output.push('{');
            self.output.push('#');
            self.output.push_str(&format!("{}", num));
            self.output.push('}');
        } else if name == "def!" {
            if tuple.len() != 3 {
                return Err(PreprocessorError::MacroError(at, format!("def! must take 3 arguments.")));
            }
            let tag = tuple.get(0).unwrap().as_text().unwrap();
            let arity = tuple.get(1).unwrap();
            let substitute = tuple.get(2).unwrap();
            self.output.push_str("\\newcommand");
            self.output.push('\\');
            self.output.push_str(tag.as_str());
            self.output.push('[');
            self.write_inner(arity)?;
            self.output.push(']');
            self.output.push('{');
            self.write_inner(substitute)?;
            self.output.push('}');
            self.last_type = LastType::Glyph;
        } else if name == "raw!" {
            let lines = tag.get_attribute_by("l").is_some() || tag.get_attribute_by("lines").is_some();
            if tuple.len() != 1 {
                return Err(PreprocessorError::MacroError(at, format!("<raw!> takes 1 text argument.")));
            }
            let argument = tuple.get(0).unwrap();
            if !argument.is_text() {
                return Err(PreprocessorError::MacroError(at, format!("<raw!> takes 1 text argument.")));
            }
            let text = argument.as_text().unwrap();
            if lines {
                self.output.write_char('\n').or(Err(PreprocessorError::MacroError(at, format!("Error on writing to output in macro at {}:{}.", at.line, at.column))))?;
                self.line += 1;
                self.write_raw(text.as_str());
            } else {
                self.write_raw(text.as_str());
            }
        } else if name.eq("n") {
            self.normalize_and_push_str("\\\\");
        } else { // Regular command.
            let mut iter = tuple.iter();
            if name.ends_with("'") {
                name = &name[0..name.len() - 1];
                self.push('\\');
                self.write_raw(name);
                if let Some(argument) = iter.next() {
                    match argument {
                        ParsedValue::Nil(_, at) => {
                            self.break_opportunity(*at);
                            self.write_raw("[]");
                        }
                        ParsedValue::Text(text, at, from) => {
                            self.break_opportunity(*at);
                            self.push('[');
                            self.write_text(text);
                            self.push(']');
                        }
                        ParsedValue::Dictionary(dictionary, at, to) => {
                            return Err(PreprocessorError::IllegalDictionary(*at));
                        }
                        ParsedValue::List(list, at, to) => {
                            return Err(PreprocessorError::IllegalTable(*at));
                        }
                        ParsedValue::Catenation(catenation, at, to) => {
                            self.break_opportunity(*at);
                            self.push('[');
                            self.write_inner(&argument)?;
                            self.push(']');
                        }
                        ParsedValue::TaggedTuple(tag, at, to) => {
                            self.break_opportunity(*at);
                            self.push('[');
                            self.write_command(&tag, *at)?;
                            self.push(']');
                        }
                    }
                } else {
                    return Err(PreprocessorError::MissingOptionalArgument(at))
                }
            } else {
                self.push('\\');
                self.write_raw(name);
            }
            if tuple.is_empty() { // No arguments - if followed by whitespace, insert empty {} after due to LaTeX scanner consuming following whitespace.
                self.last_type = LastType::Command;
            }
            while let Some(argument) = iter.next() {
                match argument {
                    ParsedValue::Nil(at, _) => {
                        self.break_opportunity(*at);
                        self.write_raw("{}");
                    }
                    ParsedValue::Text(text, at, _) => {
                        self.break_opportunity(*at);
                        self.push('{');
                        self.write_text(text);
                        self.push('}');
                    }
                    ParsedValue::Dictionary(dictionary, at, to) => {
                        return Err(PreprocessorError::IllegalDictionary(*at));
                    }
                    ParsedValue::List(table, at, to) => {
                        return Err(PreprocessorError::IllegalTable(*at));
                    }
                    ParsedValue::Catenation(catenation, at, to) => {
                        self.break_opportunity(*at);
                        self.push('{');
                        self.write_inner(&argument)?;
                        self.push('}');
                    }
                    ParsedValue::TaggedTuple(t, at, to) => {
                        self.break_opportunity(*at);
                        if t.is_empty() {
                            self.write_command(&t, *at)?;
                        } else {
                            self.push('{');
                            self.write_command(&t, *at)?;
                            self.push('}');
                        }
                    }
                }
            }
        };
        Ok(())
    }

    /// Expand a parametrized section. Parameters occur in macro definitions.
    ///
    /// Might recurse on nested catenations or nested tags.
    fn expand_value(&mut self, parametrization: &mut ParsedValue, parameters: &[&ParsedValue]) -> Result<(), String> {
        match parametrization {
            ParsedValue::Text(..) => {}
            ParsedValue::TaggedTuple(tag, ..) => {
                if let Ok(num) = tag.name().unwrap().parse::<usize>() {
                    if num == 0 || num > parameters.len() {
                        return Err(format!("Parameter input number n in <n> must be between 1 and {}, found {}.", parameters.len(), num));
                    }
                    let argument = parameters[num - 1].clone();
                    *parametrization = argument;
                } else {
                    self.expand_tuple(tag, parameters)?;
                }
            }
            ParsedValue::Dictionary(dictionary, _, _) => {
                for entry in dictionary.iter_mut() {
                    self.expand_value(entry.1, parameters)?;
                }
            },
            ParsedValue::List(list, _, _) => {
                for element in list.iter_mut() {
                    self.expand_value(element, parameters)?;
                }
            },
            ParsedValue::Catenation(catenation, ..) => {
                for element in catenation.iter_mut() {
                    match element {
                        Element::Element(e, b) => self.expand_value(e, parameters)?,
                        Element::Separator => {}
                    }
                }
            }
            ParsedValue::Nil(..) => {}
        }
        Ok(())
    }

    fn expand_tuple(&mut self, parametrized_tuple: &mut ParsedTaggedTuple, parameters: &[&ParsedValue]) -> Result<(), String> {
        for element in parametrized_tuple.iter_mut() {
            match element {
                mut v => self.expand_value(&mut v, parameters)?,
            }
        }
        Ok(())
    }

    // /// Expand a parametrized section. Parameters occur in macro definitions.
    // ///
    // /// Might recurse on nested catenations or nested tags.
    // fn expand_parametrized_catenation(&mut self, parametrization: &ParsedValue, parameters: &[ParsedValue]) -> Result<ParsedValue, String> {
    //     match parametrization {
    //         ParsedValue::Text(t, _, _) => {
    //             self.push_str(t);
    //         }
    //         ParsedValue::Tagged(tag, at, _) => {
    //             let mut name = tag.name();
    //             let tuple = tag.get();
    //             if let Ok(num) = name.parse::<u8>() {
    //                 self.push('{');
    //                 let argument = parameters[num as usize];
    //                 self.write_inner(argument)?;
    //                 self.push('}');
    //             } else if let Some(inner_macro) = macros.get(name) {
    //                 self.expand_macro(inner_macro, parameters);
    //             } else {
    //                 self.write_math_special_command(tag, at);
    //             }
    //         }
    //         ParsedValue::Tuple(_, _, _) => {
    //             return Err(PreprocessorError::MacroError(at, format!("Parameter input number n in <n> must be between 1 and {}, found {}.", parameters.len(), num)));
    //         }
    //         ParsedValue::Dictionary(_, _, _) => {
    //             return Err(PreprocessorError::MacroError(at, format!("Parameter input number n in <n> must be between 1 and {}, found {}.", parameters.len(), num)));
    //         }
    //         ParsedValue::List(_, _, _) => {
    //             return Err(PreprocessorError::MacroError(at, format!("Parameter input number n in <n> must be between 1 and {}, found {}.", parameters.len(), num)));
    //         }
    //         ParsedValue::Catenation(c, _, _) => {
    //             self.push('{');
    //             for cat in c.iter() {
    //                 match cat {
    //                     Element::Element(e) => {
    //                         self.expand_parametrized_catenation(e, parameters);
    //                     }
    //                     Element::Whitespace => {
    //                         self.push(' ');
    //                     }
    //                 }
    //             }
    //             self.push('}');
    //         }
    //         ParsedValue::Nil(_, _) => { }
    //     }
//
//
    //     match parametrization {
    //         MacroDefinition::Latex(t) => {
//
    //         }
    //         MacroDefinition::Parameter(n) => {
    //             self.push('{');
    //             self.push_str(&parameters[*n as usize]);
    //             self.push('}');
    //         }
    //         MacroDefinition::Application(name, inner_arguments) => {
    //             let inner_macro = macros.get(name);
    //             if inner_macro.is_none() {
    //                 return Err(format!("Macro {} does not exist.", name));
    //             }
    //             let inner_macro = inner_macro.unwrap();
    //             let mut inner_expanded_args = vec![];
    //             for arg in inner_arguments {
    //                 let expanded_arg = arg.expand(macros, parameters)?;
    //                 inner_expanded_args.push(expanded_arg);
    //             }
    //             let inner_expansion = inner_macro.expand(macros, inner_expanded_args.as_slice())?;
    //             expansion.push_str(&inner_expansion);
    //         }
    //         MacroDefinition::Group(c) => {
    //             expansion.push('{');
    //             for cat in c.iter() {
    //                 let exp = cat.expand(macros, parameters)?;
    //                 expansion.push_str(&exp);
    //             }
    //             expansion.push('}');
    //         }
    //     }
    // }

}

pub enum PreprocessorError {
    IllegalTable(Position),
    IllegalDictionary(Position),
    IllegalTuple(Position),
    ZeroTable(Position),
    MacroError(Position, String),
    MissingOptionalArgument(Position),
}

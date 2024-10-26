use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use include_dir::include_dir;
use khi::{Compound, Dictionary, Element, List, Tagged, Text, Tuple, Value};
use khi::parse::parse_dictionary_str;
use khi::parse::parser::error_to_string;
use khi::pdm::{ParsedDictionary, ParsedList, ParsedValue};
use khi::tex::{BreakMode, PreprocessorError, write_tex_with};
use zeroarg::{Argument, parse_arguments};

type Html = String;

fn main() {
    let mut args = env::args();
    args.next();
    let arguments = match parse_arguments(args) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("Error: Command line syntax error.");
            return;
        }
    };
    let mut help = false;
    let mut file_name = None;
    for argument in arguments {
        match argument {
            Argument::Operand(operand) => {
                if file_name.is_none() {
                    file_name = Some(operand);
                } else {
                    eprintln!("Error: Found another file operand {operand}. Only one directory/file supported.");
                    return;
                }
            }
            Argument::Attribute(k, _) => {
                eprintln!("Error: Attribute {} not supported", &k);
                return;
            }
            Argument::Flag(flag) => {
                if flag == "h" || flag == "help" || flag == "?" {
                    help = true;
                } else {
                    eprintln!("Error: Flag {} not supported", &flag);
                    return;
                }
            }
        }
    }
    if help {
        eprintln!("notarium [-h] (file)\n\nProcess current dictionary with \"notarium .\".");
        return;
    }
    if file_name.is_none() {
        eprintln!("No file was specified.");
        return;
    }
    let file_name = file_name.unwrap();
    let path = PathBuf::from(&file_name);
    if path.is_dir() {
        eprintln!("Processing dictionary {file_name}."); //TODO
        return;
        // for f in path.read_dir().unwrap() {
        //     f.un
        // }
    } else if path.is_file() {
        eprintln!("Processing file {file_name}.");
        match process_file(&file_name) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", &e);
                return;
            },
        };
        let mut path = PathBuf::from(file_name);
        path.pop();
        path.push("assets");
        path.push("notarium");
        write_assets(&path).unwrap();
    } else {
        eprintln!("{file_name} is not a file or directory.");
        return;
    }
    eprintln!("Generated page successfully");
}

fn write_assets(path: &PathBuf) -> Result<(), String> {
    let assets = include_dir!("assets-include");
    std::fs::create_dir_all(&path);
    for asset in assets.files() {
        let mut path = path.clone();
        path.push(asset.path());
        let mut file = File::create(&path).unwrap();
        if let Err(err) = file.write_all(asset.contents()) {
            return Err(format!("Error writing asset file {}", path.to_str().unwrap()));
        }
    }
    Ok(())
}

fn process_file(file_name: &str) -> Result<(), String> {
    if !file_name.ends_with(".nta") {
        return Err(format!("{file_name} must have extension \".nta\"."));
    }
    let mut title = String::new();
    let mut atypes = vec![];
    let mut articles = vec![];
    let mut structure = vec![];
    let mut progress_types = vec![];
    let mut preamble = None;

    parse_file(file_name, &mut title, &mut atypes, &mut articles, &mut structure, &mut preamble)?;
    let result_name = format!("{}.html", &file_name[0..file_name.len() - 4]); // - .nta
    let result = generate_page(&title, &articles, &atypes, &structure, &progress_types, &preamble);
    let mut result_file = File::create(&result_name).unwrap();
    match result_file.write_all(result.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => return Err(format!("Error writing file {}", &result_name)),
    }
}

fn parse_file(
    file_name: &str,
    title: &mut String,
    atypes: &mut Vec<Rc<ArticleType>>,
    articles: &mut Vec<Rc<Article>>,
    structures: &mut Vec<Rc<Structure>>,
    preamble: &mut Option<String>
) -> Result<(), String> {
    let file = File::open(file_name);
    if file.is_err() {
        return Err(format!("Error opening file {file_name}; does it exist?"));
    }
    let mut file = file.unwrap();
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err(format!("Error reading file {file_name}."));
    };
    let parse = parse_dictionary_str(&contents);
    if let Err(errs) = parse {
        let mut errors = String::new();
        errors.push_str(&format!("Error parsing file {file_name}:\n\n"));
        for err in errs { //todo
            errors.push_str(&format!("{}\n", error_to_string(&err)));
        }
        return Err(errors);
    }
    let parse = parse.unwrap();

    *title = if let Some(u) = parse.get("Document") {
        if !u.is_dictionary() {
            return Err(format!("The \"Document\" section must be a dictionary."));
        }
        let document_attributes = u.as_dictionary().unwrap();
        let title = document_attributes.get("Title");
        if title.is_none() {
            return Err("Title of document missing.".into());
        }
        let title = title.unwrap();
        if !title.is_text() {
            return Err("Title of document missing.".into());
        }
        let _author = document_attributes.get("Author");
        title.as_text().unwrap().as_str().into()

    } else {
        return Err(format!("A \"Document\" dictionary section must exist."));
    };

    *atypes = if let Some(u) = parse.get("Types") {
        if !u.is_dictionary() {
            return Err(format!("The \"Types\" section must be a dictionary."));
        }
        let article_types = u.as_dictionary().unwrap();
        read_types(article_types)?
    } else {
        return Err(format!("A \"Types\" dictionary section must exist."));
    };

    if let Some(u) = parse.get("Articles") {
        if !u.is_list() {
            eprintln!("The \"Articles\" section must be a list.");
        }
        let article_list = u.as_list().unwrap();
        read_articles(article_list, atypes, articles, structures)?
    } else {
        return Err(format!("An \"Articles\" list section must exist."));
    };

    *preamble = if let Some(preamble) = parse.get("Preamble") {
        let tex = match write_tex_with(preamble, BreakMode::Never) {
            Ok(t) => t,
            Err(err) => return Err(tex_error_to_text(err)?),
        };
        Some(tex)
    } else {
        None
    };

    for (k, _v) in parse.iter() {
        if k != "Document" && k != "Types" && k != "Articles" && k != "Preamble" {
            return Err(format!("Found unexpected section {k}."));
        }
    }

    Ok(())
}

fn unwrap_text(value: &ParsedValue) -> Result<&str, String> {
    if !value.is_text() {
        return Err(format!("Value at {}:{} is not a string.", value.from().line, value.from().column));
    }
    Ok(value.as_text().unwrap().as_str())
}

fn read_types(types: &ParsedDictionary) -> Result<Vec<Rc<ArticleType>>, String> {
    let mut a = Vec::new();

    for (ak, av) in types.iter() {
        let name = ak;
        if !av.is_dictionary() {
            eprintln!("The type \"{ak}\" at {}:{} must be a dictionary.", av.from().line, av.from().column);
        }
        let d = av.as_dictionary().unwrap();
        let colour = if let Some(c) = d.get("Colour") {
            let colour = decode_colour(c);
            if colour.is_err() {
                eprintln!("Error: Invalid Colour in type {ak}.");
                continue;
            }
            colour.unwrap()
        } else {
            "#636363".into()
        };
        let label = if let Some(l) = d.get("Label") {
            if !l.is_text() {
                eprintln!("Error: Label must be text.");
                continue;
            };
            let l = l.as_text().unwrap().as_str();
            Some(String::from(l))
        } else {
            None
        };
        let symbol = if let Some(s) = d.get("Symbol") {
            if !s.is_text() {
                eprintln!("Error: Symbol must be text.");
                continue;
            };
            let s = s.as_text().unwrap().as_str();
            Some(String::from(s))
        } else {
            None
        };
        a.push(Rc::new(ArticleType { name: name.into(), colour, label, symbol }))
    }

    Ok(a)
}

fn read_articles(article_list: &ParsedList, atypes: &Vec<Rc<ArticleType>>, articles: &mut Vec<Rc<Article>>, structure: &mut Vec<Rc<Structure>>) -> Result<(), String> {
    let mut article_map = HashMap::new();

    for article in article_list.iter() {
        if !article.is_tagged() {
            return Err(format!("Element of Articles section at {}:{} must be a tagged value", article.from().line, article.from().column));
        }
        let tag = article.as_tagged().unwrap();
        let name = tag.name.as_ref();
        let value = tag.get();
        if name == "H1" || name == "H2" || name == "H3" || name == "H4" || name == "H5" || name == "H6" {
            let (index, heading) = if value.is_tuple() {
                let value = value.as_tuple().unwrap();
                if value.len() != 2 {
                    return Err(format!("Heading at {}:{} must be tuple with 1 or 2 elements.", tag.value.from().line, tag.value.from().column));
                }
                (Some(parse_content_text(value.get(0).unwrap())?), parse_content_text(value.get(1).unwrap())?)
            } else {
                (None, parse_content_text(value)?)
            };
            if name == "H1" {
                structure.push(Rc::new(Structure::Heading(1, index, heading)));
            } else if name == "H2" {
                structure.push(Rc::new(Structure::Heading(2, index, heading)));
            } else if name == "H3" {
                structure.push(Rc::new(Structure::Heading(3, index, heading)));
            } else if name == "H4" {
                structure.push(Rc::new(Structure::Heading(4, index, heading)));
            } else if name == "H5" {
                structure.push(Rc::new(Structure::Heading(5, index, heading)));
            } else if name == "H6" {
                let label_heading = Label::Heading(heading, index);
                if structure.len() > 0 {
                    if matches!(structure.last().unwrap().as_ref(), Structure::Articles(..)) {
                        if let Structure::Articles(mut v) = Rc::into_inner(structure.pop().unwrap()).unwrap() {
                            v.push(label_heading);
                            structure.push(Rc::new(Structure::Articles(v)));
                        } else {
                            unreachable!();
                        }
                    } else {
                        structure.push(Rc::new(Structure::Articles(vec![label_heading])));
                    }
                } else {
                    structure.push(Rc::new(Structure::Articles(vec![label_heading])));
                }
            }
        } else if name == "P" {
            let tex = parse_content_text(tag.value.as_ref())?;
            structure.push(Rc::new(Structure::Paragraph(tex)));
        } else {
            let mut atype = None;
            let mut tname = tag.name.as_ref();
            let opt = if tname.ends_with('\'') {
                tname = &tname[0 .. tname.len() - 1];
                true
            } else {
                false
            };
            for t in atypes {
                if tname == t.name {
                    atype = Some(t);
                }
            }
            if atype.is_none() {
                return Err(format!("Article type {tname} at {}:{} does not exist.", article.from().line, article.from().column));
            }
            let atype = atype.unwrap().clone();
            if tag.attributes.len() != 1 {
                return Err(format!("Article at {}:{} must have a single key attribute.", article.from().line, article.from().column));
            }
            let keyattr = tag.attributes.get(0).unwrap();
            let key = if keyattr.0.as_ref() == "key" || keyattr.0.as_ref() == "k" {
                if let Some(key) = &keyattr.1 {
                    key.as_ref()
                } else {
                    return Err(format!("Key attribute in article at {}:{} does not have a value.", article.from().line, article.from().column));
                }
            } else {
                return Err(format!("Unknown attribute {} in article at {}:{}.", keyattr.0.as_ref(), article.from().line, article.from().column));
            };
            let mut argiter = tag.value.iter_as_tuple();
            let index = if opt {
                if let Some(index) = argiter.next() {
                    Some(write_label_index(index)?)
                } else {
                    return Err(format!("Article at {}:{} is indicated to have an optional argument but none was found. Please specify the index optional argument.", article.from().line, article.from().column));
                }
            } else {
                None
            };
            let title = argiter.next().ok_or(format!("Article at {}:{} must have a title argument.", article.from().line, article.from().column))?;
            let title = parse_content_text(title)?;
            let content = if let Some(vvv) = argiter.next() {
                parse_article_content(vvv)?
            } else {
                vec![]
            };
            if let Some(vvvv) = argiter.next() {
                return Err(format!("Found unexpected argument at {}:{}.", vvvv.from().line, vvvv.from().column));
            }
            let newart = Rc::new(Article {
                key: key.into(),
                title,
                article_type: atype,
                content,
            });
            if article_map.contains_key(key) {
                eprintln!("Found duplicate article keys {}.", key);
            }
            article_map.insert(newart.key.clone(), newart.clone());
            articles.push(newart.clone());
            let label = Label::Article(newart, index);
            if structure.len() > 0 {
                if matches!(structure.last().unwrap().as_ref(), Structure::Articles(..)) {
                    if let Structure::Articles(mut v) = Rc::into_inner(structure.pop().unwrap()).unwrap() {
                        v.push(label);
                        structure.push(Rc::new(Structure::Articles(v)));
                    } else {
                        unreachable!();
                    }
                } else {
                    structure.push(Rc::new(Structure::Articles(vec![label])));
                }
            } else {
                structure.push(Rc::new(Structure::Articles(vec![label])));
            }
        }
    }
    Ok(())
}

fn parse_article_content(input: &ParsedValue) -> Result<Vec<ArticleContent>, String> {
    let mut article_elements = vec![];
    if input.is_list() {
        let content = input.as_list().unwrap();
        for c in content.iter() {
            if !c.is_tagged() {
                // return Err(format!("Element of article content list at {}:{} must be a tagged value", c.from().line, c.from().column));
                // TODO: Below is very temporary workaround.
                let txt = parse_content_text(c)?;
                article_elements.push(ArticleContent::Paragraph(txt));
                continue;
            }
            let tag = c.as_tagged().unwrap();
            let name = tag.name.as_ref();
            if name == "H1" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Heading(1, tex));
            } else if name == "H2" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Heading(2, tex));
            } else if name == "H3" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Heading(3, tex));
            } else if name == "H4" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Heading(4, tex));
            } else if name == "H5" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Heading(5, tex));
            } else if name == "H6" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Heading(6, tex));
            } else if name == "P" {
                let tex = parse_content_text(tag.value.as_ref())?;
                article_elements.push(ArticleContent::Paragraph(tex));
            } else if name == "$$" {
                let tex = write_tex_with(tag.value.as_ref(), BreakMode::Never).or_else(tex_error_to_text)?;
                let tex = format!("\\[{tex}\\]");
                article_elements.push(ArticleContent::Paragraph(tex));
            } else if name == "Ol" {
                if !tag.value.is_list() {
                    return Err(format!("Expected list at {}:{}.", tag.value.from().line, tag.value.from().column));
                }
                let list = tag.value.as_list().unwrap();
                let mut html = String::new();
                html.push_str("<ol>");
                for v in list.iter() {
                    html.push_str(&format!("<li>{}</li>", parse_content_text(v)?));
                }
                html.push_str("</ol>");
                article_elements.push(ArticleContent::Html(html));
            } else if name == "Ul" {
                if !tag.value.is_list() {
                    return Err(format!("Expected list at {}:{}.", tag.value.from().line, tag.value.from().column));
                }
                let list = tag.value.as_list().unwrap();
                let mut html = String::new();
                html.push_str("<ul>");
                for v in list.iter() {
                    html.push_str(&format!("<li>{}</li>", parse_content_text(v)?));
                }
                html.push_str("</ul>");
                article_elements.push(ArticleContent::Html(html));
            } else {
                return Err(format!("Element of article content list at {}:{} must be a heading or paragraph.", c.from().line, c.from().column));
            }
        }
    } else {
        let tex = parse_content_text(input)?;
        article_elements.push(ArticleContent::Paragraph(tex));
    }
    Ok(article_elements)
}

fn parse_content_text(input: &ParsedValue) -> Result<String, String> {
    let mut output = String::new();
    write_content_text(&mut output, input)?;
    Ok(output)
}

fn write_content_text(output: &mut String, input: &ParsedValue) -> Result<(), String> {
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
                            write_content_text(output, fv)?;
                            for v in iter {
                                output.push_str("<br>");
                                write_content_text(output, v)?;
                            }
                        }
                    }
                    _ => return Err(format!("<n> command at {}:{} must take 0 or more than 2 arguments. Cannot take just 1.", t.value.from().line, t.value.from().column)),
                }
                Ok(())
            } else if name == "@" {
                output.push_str("<a>link</a>");
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
                    Element::Element(e) => write_content_text(output, e)?,
                    Element::Whitespace => output.push(' '),
                }
            }
            Ok(())
        }
        ParsedValue::Nil(..) => Ok(()),
    }
}

fn write_label_index(value: &ParsedValue) -> Result<String, String> {
    let mut html = String::new();
    //let template = include_str!("../templates/label-index.html");
    //let (p0, p1) = template.split_once("{INDEX}").unwrap();
    //html.push_str(p0);
    write_content_text(&mut html, value)?;
    //html.push_str(p1);
    Ok(html)
}

fn tex_error_to_text<T>(error: PreprocessorError) -> Result<T, String> {
    let err = match error {
        PreprocessorError::IllegalTable(p) => format!("TeX: Illegal list at {}:{}.", p.line, p.column),
        PreprocessorError::IllegalDictionary(p) => format!("TeX: Illegal dictionary at {}:{}.", p.line, p.column),
        PreprocessorError::IllegalTuple(p) => format!("TeX: Illegal tuple at {}:{}.", p.line, p.column),
        PreprocessorError::ZeroTable(p) => format!("TeX: Empty table at {}:{}.", p.line, p.column),
        PreprocessorError::MacroError(p, e) => format!("TeX: Macro error at {}:{}:\n{}", p.line, p.column, &e),
        PreprocessorError::MissingOptionalArgument(p) => format!("TeX: Missing optional argument at {}:{}.", p.line, p.column),
    };
    Err(err)
}

fn decode_colour(value: &ParsedValue) -> Result<String, String> {
    match value {
        ParsedValue::Text(s, _, _) => Ok(s.as_str().into()),
        ParsedValue::Tagged(t, _, _) => {
            let name = t.name.as_ref();
            if let Some(c) = premade_colour(name) {
                Ok(c.into())
            } else {
                Err(format!("{name} at {}:{} is not a valid colour.", value.from().line, value.from().column))
            }
        }
        _ => Err(format!("Colour value at {}:{} must be text or a tagged value.", value.from().line, value.from().column))
    }
}

fn premade_colour(colour: &str) -> Option<&str> {
    match colour {
        "Red" => Some("#703d3d"),
        "Gold" => Some("#717107"),
        "Orange" => Some("#b36f46"),
        "Green" => Some("#466441"),
        "Purple" => Some("#5f2a68"),
        _ => None,
    }
}

struct ArticleType {
    name: String,
    colour: String,
    label: Option<String>,
    symbol: Option<String>,
}

struct Article {
    key: String,
    title: String,
    article_type: Rc<ArticleType>,
    content: Vec<ArticleContent>,
}

enum Structure {
    Heading(u8, Option<String>, String),
    Paragraph(String),
    Articles(Vec<Label>),
}

/// A progress type.
/// Only levels are supported.
struct ProgressType {
    key: String,
    levels: Box<[Level]>,
}

struct Level {
    key: String,
    name: String,
    description: String,
    icon: String,
    weight: u8,
}

enum Label {
    /// An inline heading within the labels.
    Heading(String, Option<String>),
    /// A label containing an article and optional label index.
    Article(Rc<Article>, Option<String>),
}

enum ArticleContent {
    Heading(u8, String),
    Paragraph(String),
    Html(String),
}

/// Generate the page.
fn generate_page(
    title: &str,
    articles: &Vec<Rc<Article>>,
    article_types: &Vec<Rc<ArticleType>>,
    structure: &Vec<Rc<Structure>>,
    progress_types: &Vec<ProgressType>,
    preamble: &Option<String>,
) -> Html {
    let mut page = String::new();
    let str = include_str!("../templates/template.html");
    let progress_types = progress_to_json(progress_types);
    let style = generate_style(article_types);
    let preamble = match preamble {
        None => "",
        Some(s) => s.as_str(),
    };
    let overview = generate_overview_tab(structure);
    let details = generate_details_tab(articles);
    let (p0, p) = str.split_once("{TITLE}").unwrap();
    page.push_str(p0);
    page.push_str(title);
    let (p1, p) = p.split_once("{PROGRESS-TYPES}").unwrap();
    page.push_str(p1);
    page.push_str(&progress_types);
    let (p2, p) = p.split_once("{STYLE}").unwrap();
    page.push_str(p2);
    page.push_str(&style);
    let (p3, p) = p.split_once("{PREAMBLE}").unwrap();
    page.push_str(p3);
    page.push_str(preamble);
    let (p4, p) = p.split_once("{TITLE}").unwrap();
    page.push_str(p4);
    page.push_str(title);
    let (p5, p) = p.split_once("{OVERVIEW}").unwrap();
    page.push_str(p5);
    page.push_str(&overview);
    let (p6, p) = p.split_once("{DETAILS}").unwrap();
    page.push_str(p6);
    page.push_str(&details);
    page.push_str(p);
    page
}

fn progress_to_json(progress_types: &Vec<ProgressType>) -> String {
    let mut json = String::new();
    json.push('[');
    for pt in progress_types {
        json.push('{');
        json.push_str("key:\"");
        json.push_str(&pt.key);
        for level in pt.levels.iter() {
            //level.key;
        }

        json.push_str("\",name:\"");
        //json.push_str(&pt.);
        json.push_str("\",");
        json.push('}');
    }
    json.push(']');
    json
}

/// Generate the styles for the article types.
fn generate_style(atypes: &Vec<Rc<ArticleType>>) -> String {
    let mut style = String::new();
    for atype in atypes {
        let name = &atype.name;
        let colour = &atype.colour;
        let symbol = &atype.symbol;
        style.push_str(&format!(r#"
          .{name}-label {{
            background-color: {colour};
          }}
        "#));
        if let Some(symbol) = &atype.symbol {
            style.push_str(&format!(r#"
              .{name}-symbol {{
                background-image: url(assets/notarium/{symbol});
              }}
        "#));
        }
    }
    style
}

/// Generate overview.
fn generate_overview_tab(structure: &Vec<Rc<Structure>>) -> String {
    let mut view = String::new();
    for v in structure {
        match v.as_ref() {
            Structure::Heading(l, i, s) => {
                if let Some(i) = i {
                    view.push_str(&format!("<h{l}><span class=\"label-tab-text\">{i}</span> <span class=\"label-tab-text\">{s}</span></h{l}>"));
                } else {
                    view.push_str(&format!("<h{l}><span class=\"label-tab-text\">{s}</span></h{l}>"));
                }
            }
            Structure::Paragraph(p) => {
                view.push_str(&format!("<p><span class=\"label-tab-text\">{p}</span></p>"));
            }
            Structure::Articles(articles) => {
                view.push_str(&generate_labels(articles))
            }
        }
    }
    view
}

/// Generate a label box.
fn generate_labels(labels: &Vec<Label>) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"labels\">");
    for label in labels {
        match label {
            Label::Heading(h, index) => { // TODO: Accessibility: Do not skip heading levels.
                if let Some(i) = index {
                    html.push_str(&format!("<h6><span class=\"label-tab-text\">{i}</span> <span class=\"label-tab-text\">{h}</span></h6>"));
                } else {
                    html.push_str(&format!("<h6><span class=\"label-tab-text\">{h}</span></h6>"));
                }
            }
            Label::Article(a, o) => {
                let h = generate_article_label(a, o.as_ref());
                html.push_str(&h);
            }
        }
    }
    html.push_str("</div>");
    html
}

/// Generate an article label.
fn generate_article_label(article: &Article, index: Option<&String>) -> String {
    let key = &article.key;
    let content_indicator = if !article.content.is_empty() {
        "<div class=\"cind\"></div>"
    } else {
        ""
    };
    let atype = &article.article_type.name;
    let title = &article.title;
    let s = String::from("");
    let index_overlabel = if let Some(index) = index {
        format!("<span class=\"label-atl\">{index}</span>")
    } else {
        String::new()
    };
    let symbol = if let Some(symbol) = &article.article_type.symbol {
        format!("<div class=\"label-symbol {atype}-symbol\"></div>")
    } else {
        String::from("")
    };
    format!(r#"
        <div article-key="{key}" class="label {atype}-label">
          <div class="progress-box"><div class="progress"></div></div>
          <div class=" header">
            {symbol}
            <div class="label-text"><span class="label-title">{title}</span></div>
          </div>
          {index_overlabel}
          {content_indicator}
        </div>
    "#).into()
}

fn generate_details_tab(articles: &Vec<Rc<Article>>) -> String {
    let mut html = String::new();
    for article in articles {
        html.push_str(&generate_article(article));
    }
    html
}

fn generate_article(article: &Article) -> String {
    let mut html = String::new();
    let key = &article.key;
    let content = generate_article_content(&article.content);
    let label = article.article_type.label.as_ref().unwrap_or(&String::from(""));
    let p = include_str!("../templates/article.html");
    let (p0, p) = p.split_once("{KEY}").unwrap();
    html.push_str(p0);
    html.push_str(key);
    let (p1, p) = p.split_once("{TYPE}").unwrap();
    html.push_str(p1);
    html.push_str(article.article_type.name.as_str());
    let (p2, p) = p.split_once("{TITLE}").unwrap();
    html.push_str(p2);
    html.push_str(&article.title);
    let (p3, p4) = p.split_once("{CONTENT}").unwrap();
    html.push_str(p3);
    html.push_str(&content);
    html.push_str(p4);
    html
}

fn generate_article_content(content: &Vec<ArticleContent>) -> String {
    let mut html = String::new();
    for c in content {
        match c {
            ArticleContent::Heading(l, h) => {
                html.push_str(&format!("<h{l}>{h}</h{l}>"));
            }
            ArticleContent::Paragraph(p) => {
                html.push_str(&format!("<p>{p}</p>"));
            }
            ArticleContent::Html(h) => {
                html.push_str(&h);
            }
        }
    }
    html
}

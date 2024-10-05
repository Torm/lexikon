use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{PathBuf};
use std::rc::Rc;
use include_dir::include_dir;
use khi::{Compound, Dictionary, Element, List, Text, Tuple, Value};
use khi::parse::parse_dictionary_str;
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
                }
            }
            Argument::Attribute(k, _) => {
                eprintln!("Error: Attribute {} not supported", &k);
            }
            Argument::Flag(flag) => {
                if flag == "h" || flag == "help" || flag == "?" {
                    help = true;
                } else {
                    eprintln!("Error: Flag {} not supported", &flag);
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
            Err(e) => eprintln!("{}", &e),
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
    eprintln!("Finished");
}

fn write_assets(path: &PathBuf) -> Result<(), String> {
    let assets = include_dir!("assets-include");
    std::fs::create_dir_all(&path);
    for asset in assets.files() {
        let mut path = path.clone();
        path.push(asset.path());
        eprintln!("{}", path.to_str().unwrap());
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

    parse_file(file_name, &mut title, &mut atypes, &mut articles, &mut structure)?;
    let result_name = format!("{}.html", &file_name[0..file_name.len() - 4]); // - .nta
    let result = generate_page(&title, &articles, &atypes, &structure);
    let mut result_file = File::create(&result_name).unwrap();
    match result_file.write_all(result.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => return Err(format!("Error writing file {}", &result_name)),
    }
}

fn parse_file(file_name: &str, title: &mut String, atypes: &mut Vec<Rc<ArticleType>>, articles: &mut Vec<Rc<Article>>, structures: &mut Vec<Rc<Structure>>) -> Result<(), String> {
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
    if let Err(_errs) = parse {
        let mut errors = String::new();
        errors.push_str(&format!("Error parsing file {file_name}:\n\n"));
        // for err in errs { //todo
        //     errors.push_str(&format!("{err}\n"));
        // }
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

    for (k, _v) in parse.iter() {
        if k != "Document" && k != "Types" && k != "Articles" {
            return Err(format!("Found unexpected section {k}."));
        }
    }

    Ok(())
}

fn unwrap_text(value: &ParsedValue) -> Result<&str, String> {
    if !value.is_text() {
        return Err("Not a string.".into());
    }
    Ok(value.as_text().unwrap().as_str())
}

fn read_types(types: &ParsedDictionary) -> Result<Vec<Rc<ArticleType>>, String> {
    let mut a = Vec::new();

    for (ak, av) in types.iter() {
        let name = ak;
        if !av.is_dictionary() {
            eprintln!("The type \"{ak}\" must be a dictionary.");
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
            "#4a4a4a".into()
        };
        a.push(Rc::new(ArticleType { name: name.into(), colour }))
    }

    Ok(a)
}

fn read_articles(article_list: &ParsedList, atypes: &Vec<Rc<ArticleType>>, articles: &mut Vec<Rc<Article>>, structure: &mut Vec<Rc<Structure>>) -> Result<(), String> {
    for article in article_list.iter() {
        if !article.is_tagged() {
            return Err(format!("Element of Articles section must be a tagged value"));
        }
        let tag = article.as_tagged().unwrap();
        let name = tag.name.as_ref();
        if name == "H1" || name == "H2" || name == "H3" || name == "H4" || name == "H5" || name == "H6" {
            let tex = parse_content_text(tag.value.as_ref())?;
            if name == "H1" {
                structure.push(Rc::new(Structure::Heading(1, tex)));
            } else if name == "H2" {
                structure.push(Rc::new(Structure::Heading(2, tex)));
            } else if name == "H3" {
                structure.push(Rc::new(Structure::Heading(3, tex)));
            } else if name == "H4" {
                structure.push(Rc::new(Structure::Heading(4, tex)));
            } else if name == "H5" {
                structure.push(Rc::new(Structure::Heading(5, tex)));
            } else if name == "H6" {
                structure.push(Rc::new(Structure::Heading(6, tex)));
            }
        } else if name == "P" {
            let tex = parse_content_text(tag.value.as_ref())?;
            structure.push(Rc::new(Structure::Paragraph(tex)));
        } else {
            let mut atype = None;
            let tname = tag.name.as_ref();
            for t in atypes {
                if tname == t.name {
                    atype = Some(t);
                }
            }
            if atype.is_none() {
                return Err(format!("Article type {tname} does not exist."));
            }
            let atype = atype.unwrap().clone();
            if !tag.value.is_tuple() {
                return Err(format!("Article must be a tuple."));
            }
            let tuple = tag.value.as_tuple().unwrap();
            let key = tuple.get(0).ok_or(format!("Article key must exist."))?;
            let key = key.as_text().ok_or(format!("Article key must be text."))?;
            let title = tuple.get(1).ok_or(format!("Article title must exist."))?;
            let title = parse_content_text(title)?;
            let content = if let Some(vvv) = tuple.get(2) {
                parse_article_content(vvv)?
            } else {
                vec![]
            };
            let newart = Rc::new(Article {
                key: key.as_str().into(),
                title,
                article_type: atype,
                content,
            });
            articles.push(newart.clone());
            if structure.len() > 0 {
                if matches!(structure.last().unwrap().as_ref(), Structure::Articles(..)) {
                    if let Structure::Articles(mut v) = Rc::into_inner(structure.pop().unwrap()).unwrap() {
                        v.push(newart);
                        structure.push(Rc::new(Structure::Articles(v)));
                    } else {
                        unreachable!();
                    }
                } else {
                    structure.push(Rc::new(Structure::Articles(vec![newart])));
                }
            } else {
                structure.push(Rc::new(Structure::Articles(vec![newart])));
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
                return Err(format!("Element of article content must be a tagged value"));
            }
            let tag = c.as_tagged().unwrap();
            let name = tag.name.as_ref();
            let tex = parse_content_text(tag.value.as_ref())?;
            if name == "H1" {
                article_elements.push(ArticleContent::Heading(1, tex));
            } else if name == "H2" {
                article_elements.push(ArticleContent::Heading(2, tex));
            } else if name == "H3" {
                article_elements.push(ArticleContent::Heading(3, tex));
            } else if name == "H4" {
                article_elements.push(ArticleContent::Heading(4, tex));
            } else if name == "H5" {
                article_elements.push(ArticleContent::Heading(5, tex));
            } else if name == "H6" {
                article_elements.push(ArticleContent::Heading(6, tex));
            } else if name == "P" {
                article_elements.push(ArticleContent::Paragraph(tex));
            } else {
                return Err(format!("Element of article content must be a heading or paragraph."));
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
                let tex = write_tex_with(input, BreakMode::Never).or_else(tex_error_to_text)?;
                output.push('$');
                output.push_str(&tex);
                output.push('$');
                Ok(())
            } else if name == "$$" {
                let tex = write_tex_with(input, BreakMode::Never).or_else(tex_error_to_text)?;
                output.push('$');
                output.push('$');
                output.push_str(&tex);
                output.push('$');
                output.push('$');
                Ok(())
            } else if name == "n" {
                output.push_str("<br>");
                Ok(())
            } else if name == "@" {
                output.push_str("<a>link</a>");
                Ok(())
            } else {
                Err(format!("Unexpected command in content text"))
            }
        }
        ParsedValue::Tuple(_, _, _) => return Err(format!("Unexpected tuple in content text")),
        ParsedValue::Dictionary(_, _, _) => return Err(format!("Unexpected dictionary in content text")),
        ParsedValue::List(_, _, _) => return Err(format!("Unexpected list in content text")),
        ParsedValue::Compound(c, _, _) => {
            for c in c.iter() {
                match c {
                    Element::Element(e) => write_content_text(output, e)?,
                    Element::Whitespace => output.push_str(" "),
                }
            }
            Ok(())
        }
        ParsedValue::Nil(..) => Ok(()),
    }
}

fn tex_error_to_text<T>(error: PreprocessorError) -> Result<T, String> {
    Err("TeX error".into())
}

fn decode_colour(value: &ParsedValue) -> Result<String, String> {
    match value {
        ParsedValue::Text(s, _, _) => Ok(s.as_str().into()),
        ParsedValue::Tagged(t, _, _) => {
            let name = t.name.as_ref();
            if let Some(c) = premade_colour(name) {
                Ok(c.into())
            } else {
                Err(format!("{name} is not a valid colour."))
            }
        }
        _ => Err(format!("Colour must be text or a tagged value."))
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
}

struct Article {
    key: String,
    title: String,
    article_type: Rc<ArticleType>,
    content: Vec<ArticleContent>,
}

enum Structure {
    Heading(u8, String),
    Paragraph(String),
    Articles(Vec<Rc<Article>>),
}

enum ArticleContent {
    Heading(u8, String),
    Paragraph(String),
}

/// Generate the page.
fn generate_page(title: &str, articles: &Vec<Rc<Article>>, article_types: &Vec<Rc<ArticleType>>, structure: &Vec<Rc<Structure>>) -> Html {
    let mut page = String::new();
    let str = include_str!("../templates/template.html");
    let (p0, p) = str.split_once("{TITLE}").unwrap();
    page.push_str(p0);
    page.push_str(title);
    let (p1, p) = p.split_once("{STYLE}").unwrap();
    page.push_str(p1);
    let style = generate_style(article_types);
    page.push_str(&style);
    let (p2, p) = p.split_once("{TITLE}").unwrap();
    page.push_str(p2);
    page.push_str(title);
    let (p3, p) = p.split_once("{OVERVIEW}").unwrap();
    page.push_str(p3);
    let overview = generate_overview_tab(structure);
    page.push_str(&overview);
    let (p4, p) = p.split_once("{DETAILS}").unwrap();
    page.push_str(p4);
    let details = generate_details_tab(articles);
    page.push_str(&details);
    page.push_str(p);
    page
}

/// Generate the styles for the article types.
fn generate_style(atypes: &Vec<Rc<ArticleType>>) -> String {
    let mut style = String::new();
    for atype in atypes {
        let name = &atype.name;
        let colour = &atype.colour;
        style.push_str(&format!(r#"
          .{name}-label {{
            background-color: {colour};
          }}
        "#));
    }
    style
}

/// Generate overview.
fn generate_overview_tab(structure: &Vec<Rc<Structure>>) -> String {
    let mut view = String::new();
    for v in structure {
        match v.as_ref() {
            Structure::Heading(l, s) => {
                view.push_str(&format!("<h{l}>{s}</h{l}>"));
            }
            Structure::Paragraph(p) => {
                view.push_str(&format!("<p>{p}</p>"));
            }
            Structure::Articles(articles) => {
                view.push_str(&generate_labels(articles))
            }
        }
    }
    view
}

/// Generate a label box.
fn generate_labels(articles: &Vec<Rc<Article>>) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"labels\">");
    for article in articles {
        let h = generate_label(article);
        html.push_str(&h);
    }
    html.push_str("</div>");
    html
}

/// Generate an article label.
fn generate_label(article: &Article) -> String {
    let key = &article.key;
    let atype = &article.article_type.name;
    let title = &article.title;
    format!(r#"
        <div article-key="{key}" class="label">
          <div class="progress"></div>
          <div class="header {atype}-label">{title}</div>
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
                html.push_str(&format!("<p>{p}</p>"))
            }
        }
    }
    html
}

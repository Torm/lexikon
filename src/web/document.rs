use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use crate::article::{Article, Articles, Class};
use crate::compile::project::ResolutionPaths;
use crate::document::{Document, DocumentElement, PanelElement};
use crate::markup::Markup;
use crate::name::{Name, NameElement};
use crate::style::Styles;
use crate::web::class::generate_article_content;

pub fn write_documents(styles: &Styles, resolve_paths: &ResolutionPaths, articles: &Articles, root_path: &Path, documents: &[Document]) -> Result<(), String> {
    let document_dir_path = root_path.join("documents");
    fs::create_dir(&document_dir_path); // Create the documents directory.
    for document in documents {
        write_document(styles, resolve_paths, articles, &document_dir_path, document)?;
    }
    Ok(())
}

fn write_document(styles: &Styles, resolve_paths: &ResolutionPaths, articles: &Articles, document_dir_path: &Path, document: &Document) -> Result<(), String> {
    let mut document_path = document_dir_path.to_path_buf();
    for node in &document.dir_crumbs {
        document_path.push(&node.dir_name);
        if !fs::exists(&document_path).unwrap() { // Create the directory if it does not exist.
            fs::create_dir(&document_path).unwrap();
        }
    }
    let file_name = document.file_name.as_str();
    let file_name = if file_name.ends_with(".doc.khi") {
        file_name.trim_end_matches(".doc.khi")
    } else if file_name.ends_with(".document.khi") {
        file_name.trim_end_matches(".document.khi")
    } else {
        unreachable!();
    };
    let file_name = format!("{}.html", file_name);
    document_path.push(file_name);
    let document_page = generate_document_page(styles, resolve_paths, articles, document)?;
    let mut file = File::create(&document_path).unwrap();
    file.write_all(document_page.as_bytes()).unwrap();
    Ok(())
}

pub fn generate_document_page(styles: &Styles, resolve_paths: &ResolutionPaths, articles: &Articles, document: &Document) -> Result<String, String> {
    let mut html = vec![];
    let mut template = include_str!("../../templates/template.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TITLE}") {
            let title = &document.title.as_bytes();
            html.extend_from_slice(title);
            template = &template[7..];
        } else if template.starts_with(b"{NAV}") { // TODO: Replace w Nav path
            let mut path = String::from("/documents");
            for node in &document.dir_crumbs {
                path.push('/');
                path.push_str(&node.dir_name);
                let crumb = node.crumb.as_ref().unwrap_or(&node.dir_name);
                let link = format!(r#"<a href="{}">{}</a>"#, &path, crumb);
                html.extend_from_slice(link.as_bytes());
            }
            template = &template[5..];
        } else if template.starts_with(b"{RESOLUTION-PATHS}") {
            let mut paths = Vec::new();
            for path in &document.resolution_paths {
                paths.push(serde_json::Value::String(path.to_string()));
            }
            let json = serde_json::to_string(&paths).unwrap();
            html.extend_from_slice(json.as_bytes());
            template = &template[18..];
        } else if template.starts_with(b"{OVERVIEW}") {
            generate_overview_tab_content(styles, resolve_paths, articles, &mut html, document.structure.as_slice());
            template = &template[10..];
        } else if template.starts_with(b"{DETAILS}") {
            generate_details_tab(resolve_paths, &articles, &mut html, document.structure.as_slice(), document.resolution_paths.as_slice());
            template = &template[9..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
    Ok(String::from_utf8(html).unwrap())
}

/// Generate overview tab content.
fn generate_overview_tab_content(styles: &Styles, resolve_paths: &ResolutionPaths, articles: &Articles, html: &mut Vec<u8>, document_elements: &[DocumentElement]) {
    for element in document_elements {
        match element {
            DocumentElement::Heading { level, heading, index } => {
                if let Some(index) = index {
                    html.extend_from_slice(&format!(r#"<h{level}><span>{index}</span> <span>{}</span></h{level}>"#, &heading.0).as_bytes());
                } else {
                    html.extend_from_slice(&format!(r#"<h{level}><span>{}</span></h{level}>"#, &heading.0).as_bytes());
                }
            }
            DocumentElement::Paragraph(text) => {
                html.extend_from_slice(&format!(r#"<p>{}</p>"#, &text.0).as_bytes());
            }
            DocumentElement::Panel(article_elements) => {
                generate_links_panel(styles, resolve_paths, articles, html, article_elements);
            }
        }
    }
}

/// Generate a label panel.
fn generate_links_panel(styles: &Styles, resolve_paths: &ResolutionPaths, articles: &Articles, html: &mut Vec<u8>, elements: &[PanelElement]) {
    html.extend_from_slice(br#"<div class="links">"#);
    for element in elements {
        match element {
            PanelElement::Heading { level, heading, index } => {
                if let Some(index) = index {
                    html.extend_from_slice(format!(r#"<h{level}><span>{index}</span> <span>{}</span></h{level}>"#, heading.0).as_bytes());
                } else {
                    html.extend_from_slice(format!(r#"<h{level}><span>{}</span></h{level}>"#, heading.0).as_bytes());
                }
            }
            PanelElement::ArticleLink { key, index } => {
                let article = articles.article_map.get(key).unwrap();
                generate_article_link(styles, html, &article.borrow(), index.as_ref());
            }
            PanelElement::ClassLink { key, index } => {
                let class = match articles.class_map.get(key) {
                    Some(c) => c,
                    None => {
                        eprintln!("Could not find class with key {}", key);
                        panic!();
                    }
                };
                let class = class.borrow();
                let article = class.resolve(resolve_paths);
                generate_article_link(styles, html, &article.borrow(), index.as_ref()); // TODO Send class key or resolve to article key?
            }
        }
    }
    html.extend_from_slice(b"</div>");
}

/// Generate an article label. // TODO: Get rid of progress box, generate in JS
/// // TODO: Allow symbol instead of abbr.
fn generate_article_link(styles: &Styles, html: &mut Vec<u8>, article: &Article, index: Option<&String>) {
    let mut template = include_str!("../../templates/article-link.html").as_bytes();
    let class = article.class.upgrade().unwrap();
    let class = class.borrow();
    let style = if let Some(style) = &class.style {
        if let Some(style) = styles.get(style.as_ref()) {
            Some(style)
        } else {
            None
        }
    } else {
        None
    };
    while template.len() > 0 {
        if template.starts_with(b"{STYLE}") {
            if let Some(style) = style {
                let name = style.name.as_bytes();
                html.extend_from_slice(name);
                html.extend_from_slice(b"-style");
            }
            template = &template[7..];
        } else if template.starts_with(b"{ARTICLE}") {
            let article_key = article.key.as_bytes();
            html.extend_from_slice(article_key);
            template = &template[9..];
        } else if template.starts_with(b"{ABBREVIATION}") {
            if let Some(style) = style {
                if let Some(abbreviation) = &style.abbreviation { // Generate abbreviation if it is specified.
                    generate_article_link_abbreviation(html, abbreviation);
                }
            }
            template = &template[14..];
        } else if template.starts_with(b"{NAME}") {
            let name = &article.names.first().unwrap(); // TODO: Check name exists
            for ne in name.iter() {
                match ne {
                    NameElement::Name(m) => {
                        write_markup(html ,m); // TODO WRITE NAME WITH <strong>
                    }
                    _ => {},
                }
            }
            template = &template[6..];
        } else if template.starts_with(b"{INDEX}") {
            if let Some(index) = index {
                generate_article_link_index(html, index.as_ref());
            }
            template = &template[7..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
}

fn write_markup(html: &mut Vec<u8>, markup: &Markup) {
    html.extend_from_slice(markup.0.as_bytes());
}

fn generate_article_link_abbreviation(html: &mut Vec<u8>, abbreviation: &str) {
    let mut template = include_str!("../../templates/article-link-abbreviation.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TEXT}") {
            html.extend_from_slice(abbreviation.as_bytes());
            template = &template[6..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
}

fn generate_article_link_index(html: &mut Vec<u8>, index: &str) {
    let mut template = include_str!("../../templates/article-link-index.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TEXT}") {
            html.extend_from_slice(index.as_bytes());
            template = &template[6..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
}

/// Generate the articles in the details tab.
fn generate_details_tab(resolve_paths: &ResolutionPaths, articles: &Articles, html: &mut Vec<u8>, document_elements: &[DocumentElement], resolution_paths: &[String]) {
    for element in document_elements {
        if let DocumentElement::Panel(elements) = element {
            for element in elements {
                match element {
                    PanelElement::ArticleLink { key, index } => {
                        let article = articles.article_map.get(key).unwrap();
                        let article = article.borrow();
                        generate_prerendered_article(html, &article, resolution_paths);
                    }
                    PanelElement::ClassLink { key, index } => {
                        let class = articles.class_map.get(key).unwrap();
                        let class = class.borrow();
                        let article = class.resolve(resolve_paths);
                        let article = article.borrow();
                        generate_prerendered_article(html, &article, resolution_paths);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Generate a prerendered article.
///
/// These are the articles embedded into the initial HTML article files and
/// which are not generated by JavaScript.
fn generate_prerendered_article(html: &mut Vec<u8>, article: &Article, resolution_paths: &[String]) {
    let template = include_str!("../../templates/article-preload.html");
    let mut template = template.as_bytes();
    let class = &article.class.upgrade().unwrap();
    let class = class.borrow();
    let class= class.deref();
    while template.len() != 0 {
        if template.starts_with(b"{ARTICLE}") {
            let article_key = &article.key;
            let article_key = article_key.as_bytes();
            html.extend_from_slice(article_key);
            template = &template[9..];
        } else if template.starts_with(b"{CLASS}") {
            let class_key = class.key.as_bytes();
            html.extend_from_slice(class_key);
            template = &template[7..];
        } else if template.starts_with(b"{STYLE}") {
            if let Some(style) = &class.style {
                let style = style.as_bytes();
                html.extend_from_slice(style);
                html.extend_from_slice(b"-style");
            }
            template = &template[7..];
        } else if template.starts_with(b"{NAMES}") {

            let primary_name = &article.names[0];
            html.extend_from_slice(b"<h1>");
            make_long_name(html, primary_name);
            html.extend_from_slice(b"</h1>");
            //let mut i = 1;
            //while i < name.names.len() {
            //    let name = name.names[i].get_full_html();
            //    let name = name.as_str();
            //    html.extend_from_slice(b"<p>");
            //    html.extend_from_slice(name.as_bytes());
            //    html.extend_from_slice(b"</p>");
            //    i += 1;
            //}
            template = &template[7..];
        } else if template.starts_with(b"{CONTENT}") {
            generate_article_content(html, article.content.as_slice());
            template = &template[9..];
        } else if template.starts_with(b"{CLASS_LINK}") {
            let class_key = class.key.as_ref();
            html.extend_from_slice(format!("/classes/{class_key}.html").as_bytes());
            template = &template[12..];
        } else if template.starts_with(b"{LINKS}") {
            //generate_article_links(html, class, resolution_paths);///////////////////////////////////////////////////
            template = &template[7..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
}

fn make_long_name(html: &mut Vec<u8>, name: &Name) {
    for ne in name {
        match ne {
            NameElement::Name(n) => {
                html.extend_from_slice(b"<strong>");
                write_markup(html, n);
                html.extend_from_slice(b"</strong>");
            }
            NameElement::Preposition(t) => {
                write_markup(html, t);
            }
            NameElement::Parameter { markup, class } => {
                html.extend_from_slice(format!("<b data-class=\"{}\">", class).as_bytes());
                write_markup(html, markup);
                html.extend_from_slice(b"</b>");
            }
        }
    }
}

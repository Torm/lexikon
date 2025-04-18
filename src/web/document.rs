
use crate::compile::class::{Article, Class};
use crate::compile::document::{DocumentElement, FsDocument, LinksElement};
use crate::read::article::ArticleElement;

pub fn generate_document_page(document: &FsDocument) -> Result<String, String> {
    let mut html = vec![];
    let mut template = include_str!("../../templates/template.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TITLE}") {
            let title = &document.title.as_bytes();
            html.extend_from_slice(title);
            template = &template[7..];
        } else if template.starts_with(b"{NAV}") { // TODO: Replace w Nav path
            let mut path = String::from("/documents");
            for node in &document.dir_path {
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
            generate_overview_tab(&mut html,  document.structure.as_slice());
            template = &template[10..];
        } else if template.starts_with(b"{DETAILS}") {
            generate_details_tab(&mut html, document.structure.as_slice(), document.resolution_paths.as_slice());
            template = &template[9..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
    Ok(String::from_utf8(html).unwrap())
}

/// Generate overview tab content.
fn generate_overview_tab(html: &mut Vec<u8>, document_elements: &[DocumentElement]) {
    for element in document_elements {
        match element {
            DocumentElement::Heading(level, index, text) => {
                if let Some(index) = index {
                    html.extend_from_slice(&format!(r#"<h{level}><span>{index}</span> <span>{text}</span></h{level}>"#).as_bytes());
                } else {
                    html.extend_from_slice(&format!(r#"<h{level}><span>{text}</span></h{level}>"#).as_bytes());
                }
            }
            DocumentElement::Paragraph(text) => {
                html.extend_from_slice(&format!(r#"<p>{text}</p>"#).as_bytes());
            }
            DocumentElement::Links(article_elements) => {
                generate_links_panel(html, article_elements);
            }
        }
    }
}

/// Generate a label panel.
fn generate_links_panel(html: &mut Vec<u8>, elements: &[LinksElement]) {
    html.extend_from_slice(br#"<div class="links">"#);
    for element in elements {
        match element {
            LinksElement::Heading(level, text, index) => {
                if let Some(index) = index {
                    html.extend_from_slice(format!(r#"<h{level}><span>{index}</span> <span>{text}</span></h{level}>"#).as_bytes());
                } else {
                    html.extend_from_slice(format!(r#"<h{level}><span>{text}</span></h{level}>"#).as_bytes());
                }
            }
            LinksElement::Link(article, index) => {
                generate_article_link(html, article, index.as_ref());
            }
        }
    }
    html.extend_from_slice(b"</div>");
}

//// Link

/// Generate an article label. // TODO: Get rid of progress box, generate in JS
/// // TODO: Allow symbol instead of abbr.
fn generate_article_link(html: &mut Vec<u8>, article: &Article, index: Option<&String>) {
    let mut template = include_str!("../../templates/article-link.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TYPE}") {
            let type_key = article.class.article_type.key.as_bytes();
            html.extend_from_slice(type_key);
            template = &template[6..];
        } else if template.starts_with(b"{ARTICLE}") {
            let article_key = article.key.as_bytes();
            html.extend_from_slice(article_key);
            template = &template[9..];
        } else if template.starts_with(b"{ABBREVIATION}") {
            let article_type = article.class.article_type;
            if let Some(abbreviation) = &article_type.abbreviation { // Generate abbreviation if it is specified.
                generate_article_link_abbreviation(html, abbreviation);
            }
            template = &template[14..];
        } else if template.starts_with(b"{NAME}") {
            let default_name = "???".into();
            let name = article.names.first().unwrap_or(&default_name);
            html.extend_from_slice(name.as_ref());
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
fn generate_details_tab(html: &mut Vec<u8>, document_elements: &[DocumentElement], resolution_paths: &[String]) {
    for element in document_elements {
        match element {
            DocumentElement::Heading(_, _, _) => {}
            DocumentElement::Paragraph(_) => {}
            DocumentElement::Links(articles) => {
                for element in articles {
                    match element {
                        LinksElement::Heading(_, _, _) => {}
                        LinksElement::Link(article, _) => {
                            generate_prerendered_article(html, article, resolution_paths);
                        }
                    }
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
    let class = article.class;
    let type_ = class.article_type;
    while template.len() != 0 {
        if template.starts_with(b"{ARTICLE}") {
            let article_key = article.key.as_bytes();
            html.extend_from_slice(article_key);
            template = &template[9..];
        } else if template.starts_with(b"{CLASS}") {
            let class_key = class.key.as_bytes();
            html.extend_from_slice(class_key);
            template = &template[7..];
        } else if template.starts_with(b"{TYPE}") {
            let type_key = type_.key.as_bytes();
            html.extend_from_slice(type_key);
            template = &template[6..];
        } else if template.starts_with(b"{NAMES}") {
            let primary_name = article.names[0].as_bytes();
            html.extend_from_slice(b"<h1>");
            html.extend_from_slice(primary_name);
            html.extend_from_slice(b"</h1>");
            let mut i = 1;
            while i < article.names.len() {
                let name = article.names[i].as_str();
                html.extend_from_slice(b"<p>");
                html.extend_from_slice(name.as_bytes());
                html.extend_from_slice(b"</p>");
                i += 1;
            }
            template = &template[7..];
        } else if template.starts_with(b"{CONTENT}") {
            generate_article_content(html, article.content.as_slice());
            template = &template[9..];
        } else if template.starts_with(b"{CLASS_LINK}") {
            let class_key = class.key.as_ref();
            html.extend_from_slice(format!("/classes/{class_key}.html").as_bytes());
            template = &template[12..];
        } else if template.starts_with(b"{LINKS}") {
            generate_article_links(html, class, resolution_paths);
            template = &template[7..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
}

fn generate_article_links(html: &mut Vec<u8>, class: &Class, resolution_paths: &[String]) {
    for (link_type, linked_classes) in class.links_out.borrow().iter() {
        let link_type_key = &link_type.key;
        let link_type_name = &link_type.origin_name;
        let header_html = format!(r#"<span class="type" data-type="{link_type_key}">{link_type_name}</span>"#);
        html.extend_from_slice(header_html.as_bytes());
        for linked_class in linked_classes {
            let linked_class_key = &*linked_class.key;
            let resolved_article = linked_class.resolve(resolution_paths);
            let resolved_article_key = &resolved_article.key;
            let resolved_article_name = &resolved_article.names[0];
            let link_html = format!(r#"<button class="link" data-class="{linked_class_key}" data-article="{resolved_article_key}">{resolved_article_name}</button>"#);
            html.extend_from_slice(link_html.as_bytes());
        }
    }
    for (link_type, linked_classes) in class.links_in.borrow().iter() {
        let origin_type = link_type.article_type;
        let origin_type_key = &origin_type.key;
        let link_type_key = &link_type.key;
        let link_type_name = &link_type.target_name;
        let header_html = format!(r#"<span class="type" data-type="{origin_type_key}:{link_type_key}">{link_type_name}</span>"#);
        html.extend_from_slice(header_html.as_bytes());
        for linked_class in linked_classes {
            let linked_class_key = &*linked_class.key;
            let resolved_article = linked_class.resolve(resolution_paths);
            let resolved_article_key = &resolved_article.key;
            let resolved_article_name = &resolved_article.names[0];
            let link_html = format!(r#"<button class="link" data-class="{linked_class_key}" data-article="{resolved_article_key}">{resolved_article_name}</button>"#);
            html.extend_from_slice(link_html.as_bytes());
        }
    }
}

pub(crate) fn generate_article_content(html: &mut Vec<u8>, content: &[ArticleElement]) {
    for element in content {
        match element {
            ArticleElement::Heading(level, text) => {
                html.extend_from_slice(&format!("<h{level}>{text}</h{level}>").as_bytes());
            }
            ArticleElement::Paragraph(text) => {
                html.extend_from_slice(&format!(r#"<p>{text}</p>"#).as_bytes());
            }
            ArticleElement::Html(text) => {
                html.extend_from_slice(&text.as_bytes());
            }
        }
    }
}

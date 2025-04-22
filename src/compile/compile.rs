//! Compilation

use std::cell::RefCell;
use std::fs;
use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use include_dir::include_dir;
use khi::{Dictionary, Text, Value};
use khi::parse::parse_dictionary_str;
use khi::parse::parser::{error_to_string};
use khi::pdm::ParsedDictionary;
use rand::{random, Rng};
use crate::compile::class::{Article, Class, Classes};
use crate::compile::model::{ArticleType, LinkType, Model};
use crate::{read, web};
use crate::compile::dependency::include_dependency;
use crate::compile::document::{DirNode, DocumentElement, FsDocument, LinksElement};
use crate::read::article::ReadArticle;
use crate::read::document::{read_document, ReadDocument, ReadElement, ReadInlineElement};
use crate::read::model::{read_model, ReadModel};
use crate::read::project::ReadDependencyEntry;
use crate::web::model::generate_model_css;

/// Compile the project.
/// Assumes file system layout
pub fn compile() -> Result<(), String> {
    let project = read_project_file(&Path::new("project.khi"))?;
    let (model_file_path, resolution_path, commands, read_dependency_entries) = read::project::read_project(&project)?;
    let parsed_model = read_model_file(Path::new(&model_file_path))?;
    let parsed_model = read_model(&parsed_model)?;
    let model = Model::new();
    process_model(&model, &parsed_model);
    let mut read_documents = read_document_dir(Path::new("documents"))?;
    let read_dependencies = process_dependencies(&model, read_dependency_entries.as_slice())?;
    let classes = Classes::new();
    process_documents_1(&model, &classes, read_documents.as_slice(), resolution_path.as_slice(), false)?;
    for read_dependency in &read_dependencies {
        let obfuscate = match read_dependency.include {
            Include::All => false,
            Include::Articles => false,
            Include::Obfuscated => true,
        };
        process_documents_1(&model, &classes, read_dependency.documents.as_slice(), resolution_path.as_slice(), obfuscate)?;
    }
    let mut documents = process_documents_2(&model, &classes, &read_documents, resolution_path.as_slice())?;
    for read_dependency in &read_dependencies {
        if matches!(read_dependency.include, Include::All) {
            let dep_docs = process_documents_2(&model, &classes, &read_documents, resolution_path.as_slice())?;
            documents.extend(dep_docs);
        }
    }
    write_website(&model, &classes, &documents)?;
    Ok(())
}

/// For the dependencies,
fn process_dependencies(model: &Model, read_dependencies: &[ReadDependencyEntry]) -> Result<Vec<ReadDependency>, String> {
    let mut dependencies = vec![];
    for dependency in read_dependencies {
        let path = dependency.path.as_path();
        let include = dependency.include.as_str();
        let include = match include {
            "All" => Include::All,
            "Articles" => Include::Articles,
            "Obfuscated" => Include::Obfuscated,
            _ => return Err(format!("Include method {} is not allowed.", include)),
        };
        let dependency_documents = include_dependency(&model, path)?;
        dependencies.push(ReadDependency { include, documents: dependency_documents });
    }
    Ok(dependencies)
}

pub struct ReadDependency {
    pub include: Include,
    pub documents: Vec<ReadFsDocument>,
}

pub enum Include {
    All, Articles, Obfuscated,
}

//// Project

fn process_model<'a>(model: &'a Model<'a>, read_model: &ReadModel) {
    for read_type in read_model.iter() {
        let key = read_type.key.clone();
        let name = read_type.name.clone();
        let description = read_type.description.clone().unwrap_or(name.clone()); // TODO
        let colour = Some(read_type.colour.clone()); // TODO
        let abbreviation = read_type.abbreviation.clone();
        let symbol = None; // TODO
        let links = RefCell::new(vec![]);
        let article_type = ArticleType { key, name, description, colour, abbreviation, symbol, links };
        let article_type = model.type_arena.alloc(article_type);
        model.type_map.borrow_mut().insert(article_type.key.clone(), article_type);
        // Read links and insert in type.
        for read_link in &read_type.links {
            let key = read_link.key.clone();
            let origin_name = read_link.origin_name.clone();
            let origin_description = read_link.origin_description.clone();
            let target_name = read_link.target_name.clone();
            let target_description = read_link.target_description.clone();
            let target_show = read_link.target_show.clone();
            let link_type = LinkType { article_type, key, origin_name, origin_description, target_name, target_description, target_show };
            let link_type = &*model.link_arena.alloc(link_type);
            article_type.links.borrow_mut().push(link_type);
        }
    }
}

/// Read and parse project file.
pub fn read_project_file(path: &Path) -> Result<ParsedDictionary, String> {
    let file = File::open(path);
    if file.is_err() {
        return Err(format!("Error opening project file {}; does it exist?", path.to_str().unwrap()));
    }
    let mut file = file.unwrap();
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err(format!("Error reading project file '{}'.", path.to_str().unwrap()));
    }
    let parse = parse_dictionary_str(&contents);
    if let Err(errs) = parse {
        let mut errors = String::new();
        errors.push_str(&format!("Error parsing project file {}:\n\n", path.to_str().unwrap()));
        for err in errs { //todo
            errors.push_str(&format!("{}\n", error_to_string(&err)));
        }
        return Err(errors);
    }
    Ok(parse.unwrap())
}

//// Model

pub fn read_model_file(model_file_path: &Path) -> Result<ParsedDictionary, String> {
    let mut model_file = File::open(model_file_path).unwrap();
    let mut readt_model = String::new();
    model_file.read_to_string(&mut readt_model).unwrap();
    let parsed_model = match parse_dictionary_str(&readt_model) {
        Ok(s) => s,
        Err(e) => return Err(format!("Error in model file.")),
    };
    Ok(parsed_model)
}

//// Documents directory

pub fn read_document_dir(path: &Path) -> Result<Vec<ReadFsDocument>, String> {
    let mut read_documents = vec![];
    let document_dir_path = PathBuf::from(path);
    let mut dir_path = vec![];
    read_document_dir_inner(document_dir_path.as_path(), &mut read_documents, &mut dir_path)?;
    Ok(read_documents)
}

fn read_document_dir_inner(dir_path: &Path, read_documents: &mut Vec<ReadFsDocument>, crumbs: &mut Vec<Rc<DirNode>>) -> Result<(), String> {
    {
        let dir_file_path = dir_path.join("dir.khi");
        if dir_file_path.exists() {
            let dir_crumb = read_dir_file(&dir_file_path)?;
            if crumbs.len() > 0 {
                let crumb = crumbs.pop().unwrap();
                let crumb = DirNode { dir_name: crumb.dir_name.clone(), crumb: Some(dir_crumb) };
                crumbs.push(Rc::new(crumb));
            }
        }
    }
    for dir_entry in read_dir(&dir_path).unwrap() {
        let dir_entry = dir_entry.unwrap();
        let file_name = dir_entry.file_name();
        let entry_type = dir_entry.file_type().unwrap();
        if entry_type.is_file() {
            if file_name.as_encoded_bytes().ends_with(b".d.khi") || file_name.as_encoded_bytes().ends_with(b".d") {
                let document_path = dir_path.join(&file_name);
                let document = read_document_file(&document_path)?;
                let document = read_document(&document)?;
                let document = ReadFsDocument { file_name: file_name.to_str().unwrap().to_string(), dir_path: crumbs.clone(), document };
                read_documents.push(document);
            }
        } else if entry_type.is_dir() {
            let dir_path = dir_path.join(&file_name);
            let pathn = file_name.to_str().unwrap().to_string();
            crumbs.push(Rc::new(DirNode { dir_name: pathn , crumb: None }));
            read_document_dir_inner(&dir_path, read_documents, crumbs)?;
            crumbs.pop();
        }
    }
    Ok(())
}

pub struct ReadFsDocument {
    pub file_name: String,
    pub dir_path: Vec<Rc<DirNode>>,
    pub document: ReadDocument,
}

/// Read a document file.
pub fn read_document_file(file_path: &Path) -> Result<ParsedDictionary, String> {
    let file = File::open(file_path);
    if file.is_err() {
        return Err(format!("Error opening file {}; does it exist?", file_path.to_str().unwrap()));
    }
    let mut file = file.unwrap();
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err(format!("Error reading file {}.", file_path.to_str().unwrap()));
    };
    let parse = parse_dictionary_str(&contents);
    if let Err(errs) = parse {
        let mut errors = String::new();
        errors.push_str(&format!("Error parsing file {}:\n\n", file_path.to_str().unwrap()));
        for err in errs { //todo
            errors.push_str(&format!("{}\n", error_to_string(&err)));
        }
        return Err(errors);
    }
    Ok(parse.unwrap())
}

/// Read the dir file and return the name stored in it.
/// If no dir file, return directory name.
fn read_dir_file(dir_file_path: &Path) -> Result<String, String> {
    if dir_file_path.exists() {
        let mut file = File::open(dir_file_path).unwrap();
        let mut filebuf = String::new();
        file.read_to_string(&mut filebuf).unwrap();
        match parse_dictionary_str(filebuf.as_str()) {
            Ok(d) => {
                if let Some(name) = d.get("Name") {
                    if !name.is_text() {
                        return Err(format!("Name in dir file '{}' must be text.", dir_file_path.to_str().unwrap()));
                    }
                    let name = name.as_text().unwrap().as_str();
                    Ok(name.to_string())
                } else {
                    return Err(format!("Name not found in dir file '{}'.", dir_file_path.to_str().unwrap()));
                }
            }
            Err(e) => {
                return Err(format!("Error parsing dir file '{}'.", dir_file_path.to_str().unwrap()));
                // TODO
            }
        }
    } else {
        return Err(format!("Dir file '{}' does not exist.", dir_file_path.to_str().unwrap()));
    }
}

//// Processing

/// Process the [ReadDocument]s.
/// First pass - initialize classes and register articles.
///
/// 1) Initializes classes that do not exist.
/// 2) Registers articles.
pub fn process_documents_1<'a>(model: &'a Model<'a>, classes: &'a Classes<'a>, read_documents: &[ReadFsDocument], resolution: &[String], obfuscate: bool) -> Result<(), String> {
    for read_document in read_documents {
        for read_article in &read_document.document.read_articles {
            let article = register_article(model, classes, read_article)?;
            if obfuscate {
                let rand = rand::rng().random::<u16>();
                let rand = format!("{:x}", rand);
                *article.key.borrow_mut() = format!("{}#{}@?", article.class.key.as_ref(), &rand);
            }
        }
    }
    Ok(())
}

/// Process the [ReadDocument]s.
/// Second pass - process documents and update links.
///
/// 3) Registers class links.
/// 4) Processes each [ReadDocument].
pub fn process_documents_2<'a>(model: &'a Model<'a>, classes: &'a Classes<'a>, read_documents: &[ReadFsDocument], resolution: &[String]) -> Result<Vec<FsDocument<'a>>, String> {
    let mut documents = Vec::new();
    for read_document in read_documents {
        let document = process_document(classes, read_document, resolution)?;
        for read_article in &read_document.document.read_articles {
            update_class_links(classes, read_article)?;
        }
        documents.push(document);
    }
    Ok(documents)
}

/// Process a read document.
///
/// Assumes that all classes and articles have been registered.
fn process_document<'a>(classes: &Classes<'a>, read_fs_document: &ReadFsDocument, project_resolution_paths: &[String]) -> Result<FsDocument<'a>, String> {
    let read_document = &read_fs_document.document;
    let key = read_document.key.to_string();
    let dir_path = read_fs_document.dir_path.clone();
    let file_name = read_fs_document.file_name.clone();
    let title = read_document.title.to_string();
    let description = read_document.description.clone();
    let preamble = read_document.preamble.clone();
    let mut resolution_paths = vec![];
    resolution_paths.extend_from_slice(&read_document.resolution_paths);
    resolution_paths.extend_from_slice(project_resolution_paths);
    let mut structure = Vec::new();
    for element in &read_document.read_elements {
        match element {
            ReadElement::Heading(level, index, text) => {
                structure.push(DocumentElement::Heading(*level, index.clone(), text.to_string()));
            }
            ReadElement::Paragraph(text) => {
                structure.push(DocumentElement::Paragraph(text.to_string()));
            }
            ReadElement::Panel(links) => {
                let mut panel = Vec::new();
                for link in links {
                    match link {
                        ReadInlineElement::Heading(level, text, index) => {
                            panel.push(LinksElement::Heading(*level, text.to_string(), index.clone()));
                        }
                        ReadInlineElement::Link(link_key, index) => {
                            let article = classes.get_article(&link_key);
                            if article.is_none() {
                                return Err(format!("Article {} does not exist.", &link_key));
                            }
                            let article = article.unwrap();
                            panel.push(LinksElement::Link(article, index.clone()));
                        }
                    }
                }
                structure.push(DocumentElement::Links(panel));
            }
        }
    }
    Ok(FsDocument { key, dir_path, file_name, title, description, preamble, resolution_paths, structure })
}

/// Register an article and initialize the associated class if it does not already exist.
fn register_article<'a>(model: &'a Model<'a>, classes: &'a Classes<'a>, read_article: &ReadArticle) -> Result<&'a Article<'a>, String> {
    // Get type and ensure it exists.
    let type_key = read_article.type_key.as_str();
    let type_ = model.get_type(type_key);
    if type_.is_none() {
        return Err(format!("Found non-declared type {}.", type_key));
    }
    let type_ = type_.unwrap();
    // Check article does not already exist.
    let article_key = read_article.article_key.as_str();
    if let Some(_article) = classes.get_article(article_key) {
        return Err(format!("The article {} already exists.", article_key));
    }
    // Get or create class.
    let class_key = read_article.class_key.as_str();
    let class = if let Some(class) = classes.get_class(class_key) {
        class
    } else {
        &*classes.insert_class(Class::new(class_key, type_))
    };
    // Check that types match.
    if class.article_type as *const ArticleType != type_ as *const ArticleType {
        return Err(format!("Article {} declared with different type.", read_article.article_key.as_str()));
    }
    // Register article in class.
    let names = read_article.names.clone();
    let content = read_article.content.clone();
    let article = Article { class, key: RefCell::new(article_key.to_string()), names, content };
    let article = classes.insert_article(article);
    Ok(article)
}

/// Register the read links.
fn update_class_links<'a>(classes: &'a Classes<'a>, read_article: &ReadArticle) -> Result<(), String> {
    let class_key = &read_article.class_key;
    let class = classes.get_class(class_key).unwrap(); // Must exist as result of first pass.
    let article_type = class.article_type;
    for (link_type_key, linked_class_keys) in &read_article.links {
        // Get link type and check that is exists.
        let link_type = article_type.get_link(link_type_key);
        if link_type.is_none() {
            return Err(format!("Link type {}:{} does not exist.", &article_type.key, link_type_key));
        }
        let link_type = link_type.unwrap();
        // Register all links of this type.
        for linked_class_key in linked_class_keys {
            let linked_class = classes.get_class(linked_class_key);
            if linked_class.is_none() {
                return Err(format!("Linked class {} does not exist.", linked_class_key));
            }
            let linked_class = linked_class.unwrap();
            classes.insert_link(link_type, class, linked_class);
        }
    }
    Ok(())
}

//// Write

/// Write website.
fn write_website(model: &Model, classes: &Classes, documents: &[FsDocument]) -> Result<(), String> {
    let temp_path = Path::new(".website.tmp");
    let target_path = Path::new("website");
    // Clear and create temp target directory if it was for some reason not cleaned.
    if fs::exists(temp_path).unwrap() {
        fs::remove_dir_all(temp_path);
    }
    fs::create_dir(temp_path);
    // Write website files.
    write_model_file(temp_path, model)?;
    write_model_css(temp_path, model)?;
    write_classes(temp_path, classes)?;
    write_documents(temp_path, documents)?;
    include_static_assets(temp_path)?;
    include_assets(temp_path)?;
    include_index_and_icon(temp_path)?;
    update_modification_dates(target_path, temp_path)?;
    // Replace the old target directory with the newly generated files.
    fs::remove_dir_all(target_path);
    fs::rename(temp_path, target_path);
    // Clear and create temp target directory.
    if fs::exists(temp_path).unwrap() {
        fs::remove_dir_all(temp_path);
    }
    Ok(())
}

/// Write or update the model file.
pub fn write_model_file(root_path: &Path, model: &Model) -> Result<(), String> {
    let model_path = root_path.join("model.json");
    let mut file = File::create_new(&model_path).or(
        Err(format!("Error creating model data file {}.", model_path.to_str().unwrap()))
    )?;
    let model_data = web::model::generate_model_data(model)?;
    file.write_all(model_data.as_bytes()).or(
        Err(format!("Error writing to model data file {}.", model_path.to_str().unwrap()))
    )?;
    Ok(())
}

fn write_model_css(root_path: &Path, model: &Model) -> Result<(), String> {
    let model_css_path = root_path.join("model.css");
    let mut file = File::create_new(&model_css_path).or(
        Err(format!("Error creating model CSS file {}.", model_css_path.to_str().unwrap()))
    )?;
    let model_css = generate_model_css(model)?;
    file.write_all(model_css.as_bytes()).or(
        Err(format!("Error writing to model CSS file {}.", model_css_path.to_str().unwrap()))
    )?;
    Ok(())
}

/// Write class files to the class directory.
fn write_classes(root_path: &Path, classes: &Classes) -> Result<(), String> {
    let class_dir_path = root_path.join("classes");
    if let Err(_) = fs::create_dir(&class_dir_path) {
        return Err(format!("Error creating class directory {}.", class_dir_path.to_str().unwrap())); // Create the temporary class directory.
    }
    // Write articles.
    for (_, class) in classes.get_classes().iter() {
        write_class_data(&class_dir_path, class)?;
        write_class_page(&class_dir_path, class)?;
    }
    Ok(())
}

/// Write or update a class file.
///
/// If the class has not changed, the file is not modified.
fn write_class_data(class_dir_path: &Path, class: &Class) -> Result<(), String> {
    let class_key = class.key.as_ref();
    let class_file_name = format!("{}.json", class_key);
    let class_path = class_dir_path.join(&class_file_name);
    let class_data = web::class::generate_class_data(class)?;
    let mut file = File::create(&class_path).unwrap();
    if let Err(_) = file.write_all(class_data.as_bytes()) {
        return Err(format!("Error writing to class file {}.", class_path.to_str().unwrap()));
    }
    Ok(())
}

fn write_class_page(class_dir_path: &Path, class: &Class) -> Result<(), String> {
    Ok(()) // TODO
}

pub fn write_documents(root_path: &Path, documents: &[FsDocument]) -> Result<(), String> {
    let document_dir_path = root_path.join("documents");
    fs::create_dir(&document_dir_path); // Create the documents directory.
    for document in documents {
        write_document(&document_dir_path, document)?;
    }
    Ok(())
}

fn write_document(document_dir_path: &Path, document: &FsDocument) -> Result<(), String> {
    let mut document_path = document_dir_path.to_path_buf();
    for node in &document.dir_path {
        document_path.push(&node.dir_name);
        if !fs::exists(&document_path).unwrap() { // Create the directory if it does not exist.
            fs::create_dir(&document_path).unwrap();
        }
    }
    let file_name = document.file_name.as_str();
    let file_name = if file_name.ends_with(".d") {
        file_name.trim_end_matches(".d")
    } else if file_name.ends_with(".d.khi") {
        file_name.trim_end_matches(".d.khi")
    } else {
        unreachable!();
    };
    let file_name = format!("{}.html", file_name);
    document_path.push(file_name);
    let document_page = web::document::generate_document_page(document)?;
    let mut file = File::create(&document_path).unwrap();
    file.write_all(document_page.as_bytes()).unwrap();
    Ok(())
}

/// Write out all assets that should be included.
fn include_static_assets(root_path: &Path) -> Result<(), String> {
    let static_assets_dir_path = root_path.join("assets/static");
    let assets = include_dir!("assets-include");
    fs::create_dir_all(&static_assets_dir_path);
    for asset in assets.files() {
        let asset_path = static_assets_dir_path.join(asset.path());
        let mut file = File::create(&asset_path).unwrap();
        if let Err(err) = file.write_all(asset.contents()) {
            return Err(format!("Error writing asset file {}.", asset_path.to_str().unwrap()));
        }
    }
    Ok(())
}

fn include_assets(root_path: &Path) -> Result<(), String> {
    Ok(()) // TODO
}

fn include_index_and_icon(root_path: &Path) -> Result<(), String> {
    Ok(()) // TODO
}

/// Update the modification times of the files.
///
/// If a file is identical to a previous version, set the modification time to
/// the old time.
fn update_modification_dates(old_path: &Path, new_path: &Path) -> Result<(), String> {
    Ok(()) // TODO
}

/// Check if some content is identical to some file's content.
/// Returns false if the file does not exist or if it is not identical.
fn is_file_identical(path: &Path, content: &str) -> Result<bool, String> {
    if fs::exists(&path).unwrap() {
        let mut current_class_file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Unable to read current class file {}.", path.to_str().unwrap())),
        };
        let mut current_class_content = String::new();
        current_class_file.read_to_string(&mut current_class_content); // TODO
        Ok(current_class_content == content)
    } else {
        Ok(false)
    }
}

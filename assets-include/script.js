

/// Buckets

class Buckets {
    inner = new Map();

    insert(key, element) {
        let bucket = this.inner.get(key);
        if (bucket !== undefined) {
            bucket.push(element);
        } else {
            this.inner.set(key, [element]);
        }
    }
    delete(key, element) {
        let bucket = this.inner.get(key);
        if (bucket !== undefined) {
            let index = bucket.lastIndexOf(element);
            if (index !== -1) {
                bucket.splice(index, 1);
            }
        }
    }
    has(key, element) {
        let bucket = this.inner.get(key);
        if (bucket !== undefined) {
            return bucket.includes(element);
        } else {
            return false;
        }
    }
    count(key) {
        let bucket = this.inner.get(key);
        if (bucket !== undefined) {
            return bucket.length;
        } else {
            return 0;
        }
    }
}

/// Initialize

/**
 * Always incremented when opening an article. Used to order last opened article
 * last.
 */
//let index = 0;

/**
 * The progress stored for each article. Should be persisted.
 */
let progress = new Map();

/**
 * Key of last clicked article. Used to store the subject of a context menu.
 */
let lastClickedKey = null;

/**
 * The progress types found in this document.
 */
let progressTypes = new Map();

let articleTypes;

let linkTypes = new Map();

let model = new Map();

let activeTooltip = null;

/**
 * Currently open class dialog, null if none is open.
 */
let activeClassDialog = null;
let activeClass = null;
let activeArticle = null;

/**
 * A map of the article classes loaded so far.
 */
let loadedClasses = new Map();

let loadingClasses = new Map();

/**
 * Articles rendered statically.
 */
let prerenders = new Map();

/**
 * The collection of currently opened articles.
 */
let openArticles = new Buckets();

let resolutionPath = [];

/**
 * Read the article types, progress types and articles from data.
 */
function init() {
    minimizeArticles();

    readModel();

    readDocumentResolutionData();

    readDocumentArticles();

    touchLabels();

    // Click article header - Expand/collapse article
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches(".article > header")) return;
        let article = element.parentElement;
        if (article.classList.contains("collapsed")) {
            article.classList.remove("collapsed");
        } else {
            article.classList.add("collapsed");
        }
    });
    // Click link group - Expand/collapse link group
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches(".links > .type")) return;
        let links = element.parentElement;
        let inRange = false;
        for (let child of links.children) {
            if (inRange) {
                if (child.classList.contains("type")) break;
                if (child.style.display === "none") {
                    child.style.display = null;
                } else {
                    child.style.display = "none";
                }
            } else {
                if (child === element) {
                    inRange = true;
                }
            }
        }
    });
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches(".links.unloaded.collapsed")) return;
        let article = element.parentElement;
        populateLinks(article);
    });
    // Article close button - Close article
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches(".article > header > menu > .close-button")) return;
        let article = element.parentElement.parentElement.parentElement;
        article.parentElement.removeChild(article);
    });
    // Article class button - Open class dialog
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches(".article > header > menu > .class-button")) return;
        let article = element.parentElement.parentElement.parentElement;
        let key = article.getAttribute("data-class");
        let x = element.getBoundingClientRect().left;
        let y = element.getBoundingClientRect().bottom;
        activeArticle = article;
        openClassDialog(key, x, y);
    });
    // Click label header - Open article
    document.addEventListener("click", event => {
        let element = event.target;
        while (element.matches(".link > .header *")) {
            element = element.parentElement;
        }
        if (!element.matches(".link > .header")) return;
        let label = element.parentElement;
        let key = label.getAttribute("data-article");
        openPrerenderedArticle(key);
    });
    // Click outside class dialog - delete dialog.
    document.addEventListener("click", event => {
        let element = event.target;
        if (element.matches("#class-dialog, #class-dialog *, .article > header > menu > .class-button")) return;
        let dialog = document.getElementById("class-dialog");
        if (dialog !== null) document.body.removeChild(dialog);
        activeClassDialog = null;
    });
    // Click class dialog entry - Open article.
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches("#class-dialog > ul > li, #class-dialog > ul > li *")) return;
        while (!element.matches("#class-dialog > ul > li")) {
            element = element.parentElement;
        }
        let articleKey = element.getAttribute("data-article");
        //let order = activeArticle.style.order;
        let articleData = loadedClasses.get(activeClass).articles[articleKey];
        let typeKey = loadedClasses.get(activeClass).type;
        let article = generateArticle(typeKey, activeClass, articleKey, articleData);
        //article.style.order = order + 1;
        // TODO: Insert in buckets, update indicators
        activeArticle.after(article);
    });
    // Click article links panel - expand/collapse links
    document.addEventListener("click", event => {
        let element = event.target;
        if (!element.matches(".article > .links")) return;
        if (element.matches(".article > .links:not(.collapsed):empty")) return; // Do not collapse empty.
        element.classList.toggle("collapsed");
    });
    // Article button tooltips.
    document.addEventListener("mouseover", event => {
        if (event.target.matches(".article > header > menu > .close-button")) {
            createTooltip(event.target, document.createTextNode("Minimize article"));
        } else if (event.target.matches(".article > header > menu > a")) {
            createTooltip(event.target, document.createTextNode("Go to class page"));
        } else if (event.target.matches(".article > header > menu > .class-button")) {
            createTooltip(event.target, document.createTextNode("View articles in class"));
        }
    });

    document.addEventListener("mouseover", event => {
        let target = event.target;
        let tooltip = target.querySelector(".tooltip:not(.temp)");
        if (tooltip === null) return;
        clearTooltip();
        tooltip.style.display = "block";
        let box = target.getBoundingClientRect();
        tooltip.style.top = box.bottom + "px";
        tooltip.style.left = box.right + "px";
        activeTooltip = tooltip;
    });

    document.addEventListener("mouseout", event => {
        if (activeTooltip === null) return;
        let parent = activeTooltip.parentElement;
        let target = event.target;
        if (parent !== target) return;
        clearTooltip();
    });
    document.addEventListener("mouseover", handleLinkGroupTooltip);
    document.addEventListener("click", handleArticleLinkFollow);
}

/**
 * Tooltip for link group. Shows description.
 */
function handleLinkGroupTooltip(event) {
    let target = event.target;
    if (!target.matches(".article > .links > .type")) return;
    let article = event.target.parentElement.parentElement;
    let articleType;
    if (article.hasAttribute("data-type")) {
        // Can set data-type on prerendered articles so their class does not need to be loaded, only model.json.
        articleType = article.getAttribute("data-type");
    } else {
        let articleClass = loadedClasses.get(article.getAttribute("data-class"));
        if (articleClass === undefined) {
            return;
        }
        articleType = articleClass.type;
    }
    let type = target.getAttribute("data-type");
    type = type.split(":");
    if (type.length === 1) {
        let lType = type[0];
        let linkType = model[articleType].links[lType];
        let description = linkType["origin.description"];
        createTooltip(target, description);
    } else if (type.length === 2) {
        let aType = type[0];
        let lType = type[1];
        let linkType = model[aType].links[lType];
        let description = linkType["target.description"];
        createTooltip(target, description);
    } else {
        console.log("Incorrect format of link header type attribute: " + type);
        return;
    }
}

/**
 * Handle click on article link.
 */
async function handleArticleLinkFollow(event) {
    let target = event.target;
    if (!target.matches(".article > .links > .link")) return;
    let classKey = target.getAttribute("data-class");
    let articleKey = target.getAttribute("data-article");
    let loadedClass = await loadClass(classKey);
    let typeKey = loadedClass.type;
    let article = generateArticle(typeKey, classKey, articleKey, loadedClass.articles[articleKey]);
    target.parentElement.parentElement.after(article);
}

async function readModel() {
    let modelFile = await fetch("/model.json");
    if (modelFile.status !== 200) {
        console.log("Failed to load model file.");
        return;
    }
    model = await modelFile.json();
}

function clearTooltip() {
    if (activeTooltip === null) return;
    if (activeTooltip.classList.contains("temp")) {
        activeTooltip.parentElement.removeChild(activeTooltip);
    } else {
        activeTooltip.style.display = null;
    }
    activeTooltip = null;
}

async function populateLinks(article) {
    let classKey = article.getAttribute("data-class");
    let c = loadedClasses.get(classKey);
    if (c === null) {
        console.log("Class " + classKey + " is not loaded.");
        return;
    }
    let promises = [];
    if (c["links"] !== undefined) {
        for (let entry of Object.entries(c["links"])) {
            let targets = entry[1];
            for (let target of targets) {
                promises.push(loadClass(target));
            }
        }
    }
    await Promise.allSettled(promises);
    let linkSection = article.querySelector(".links");
    linkSection.replaceChildren(); // Clear node
    let classType = c.type;
    if (c["links"] !== undefined) {
        for (let entry of Object.entries(c["links"])) {
            let linkTypeCompound = entry[0];
            linkTypeCompound = linkTypeCompound.split(":");
            let linkTypeLinks = entry[1];
            if (linkTypeCompound.length === 2) {
                let originClass = linkTypeCompound[0];
                let linkType = linkTypeCompound[1];
                let linkTypeLinks = entry[1];
                let targetTypeData = model[originClass];
                if (targetTypeData === null) continue;
                let targetTypeLinkData = targetTypeData["links"][linkType];
                let linkGroupTag = document.createElement("span");
                linkGroupTag.classList.add("type");
                linkGroupTag.textContent = targetTypeLinkData["target.name"];
                linkGroupTag.setAttribute("data-type", originClass + ":" + linkType);
                linkSection.appendChild(linkGroupTag);
                for (let target of linkTypeLinks) {
                    let targetClass = loadedClasses.get(target);
                    if (targetClass === null) {
                        console.log("No class " + target + " found.");
                        continue;
                    }
                    let [articleKey, articleData] = resolve(targetClass, resolutionPath);
                    let articleName = articleData.names[0];
                    let linkTag = document.createElement("button");
                    linkTag.classList.add("link");
                    linkTag.setAttribute("data-class", target);
                    linkTag.setAttribute("data-article", articleKey);
                    linkTag.textContent = articleName;
                    linkSection.appendChild(linkTag);
                }
            } else {
                let linkType = linkTypeCompound[0];
                let linkTypeData = model[classType].links[linkType];
                let linkGroupTag = document.createElement("span");
                linkGroupTag.classList.add("type");
                linkGroupTag.textContent = linkTypeData["origin.name"];
                linkGroupTag.setAttribute("data-type", linkType);
                linkSection.appendChild(linkGroupTag);
                for (let target of linkTypeLinks) {
                    let targetClass = loadedClasses.get(target);
                    if (targetClass === null) {
                        console.log("No class " + targetClass + " found.");
                        continue;
                    }
                    let [articleKey, articleData] = resolve(targetClass, resolutionPath);
                    let articleName = articleData.names[0];
                    let linkTag = document.createElement("button");
                    linkTag.classList.add("link");
                    linkTag.setAttribute("data-class", target);
                    linkTag.setAttribute("data-article", articleKey);
                    linkTag.textContent = articleName;
                    linkSection.appendChild(linkTag);
                }
            }
        }
    }
    linkSection.classList.remove("unloaded");
}

function readDocumentResolutionData() {
    let tag = document.getElementById("resolve-list");
    resolutionPath = JSON.parse(tag.textContent);
}

/**
 * Resolve an article of a class given a list of resolve paths.
 * If class does not have any of the paths, return the first article in the class.
 */
function resolve(c, paths) {
    let articles = Object.entries(c.articles);
    for (let path of paths) {
        for (let articleEntry of articles) {
            let key = articleEntry[0];
            if (!key.endsWith("@" + path)) continue;
            return articleEntry;
        }
    }
    return articles[0]; // None found, just return the first one.
}

function readDocumentArticles() {
    let articles = document.querySelectorAll(".article");
    for (let article of articles) {
        let key = article.getAttribute("data-article");
        prerenders.set(key, article.cloneNode(true));
        openArticles.insert(key, article);
    }
}

function touchLabels() {
    let labels = document.querySelectorAll("#overview-tab > .links > .link");
    for (let label of labels) {
        let key = label.getAttribute("data-article");
        if (openArticles.count(key) > 0) {
            label.classList.add("open");
        }
    }
}

document.addEventListener("DOMContentLoaded", event => {
    init();
});

/**
 * Minimize all articles.
 */
function minimizeArticles() {
    document.addEventListener("DOMContentLoaded", event => {
        let articles = document.getElementsByTagName("article");
        for (let article of articles) {
            article.classList.add("minimized");
        }
    });
}

/// Classes

/**
 * Start the loading of a class.
 */
async function loadClass(key) {
    console.log("Loading class " + key);
    let c = loadedClasses.get(key);
    if (c === undefined) {
        let loading = loadingClasses.get(key);
        if (loading !== undefined) {
            return await loading;
        } else {
            let classPath = "/classes/" + key + ".json";
            let future = fetch(classPath).then(file => {
                if (!file.ok) {
                    return Promise.reject("Failed loading class " + key);
                }
                return file.json();
            });
            loadingClasses.set(key, future);
            let json = await future;
            loadingClasses.delete(key);
            loadedClasses.set(key, json);
            console.log("Loaded class " + key);
            return json;
        }
    } else {
        return c;
    }
}

/// Articles

function generateArticle(typeKey, classKey, articleKey, articleData) {
    let article = document.createElement("article");
    article.setAttribute("data-article", articleKey);
    article.setAttribute("data-class", classKey);
    article.classList.add("article", typeKey + "-type");
    // Header
    let header = document.createElement("header");
    header.classList.add("header");
    article.appendChild(header);
    // header > hgroup
    let hgroup = document.createElement("hgroup");
    header.appendChild(hgroup);
    {
        let h1 = document.createElement("h1");
        h1.textContent = articleData.names[0];
        hgroup.appendChild(h1);
        let i = 1;
        while (i < articleData.names.length) {
            let p = document.createElement("p");
            p.textContent = articleData.names[i];
            hgroup.appendChild(p);
            i++;
        }
    }
    // header > menu
    let menu = document.createElement("menu");
    header.appendChild(menu);
    {
        let progressBox = document.createElement("button");
        progressBox.classList.add("progress-box");
        menu.appendChild(progressBox);
        let classButton = document.createElement("button");
        classButton.classList.add("class-button");
        menu.appendChild(classButton);
        let classLink = document.createElement("a");
        classLink.href = "/classes/" + classKey + ".html";
        classLink.target = "_blank";
        menu.appendChild(classLink);
        let closeButton = document.createElement("button");
        closeButton.classList.add("close-button");
        menu.appendChild(closeButton);
    }
    // Content
    if (articleData.content !== undefined && articleData.content !== "") {
        let content = document.createElement("div");
        content.classList.add("content");
        article.appendChild(content);
        content.innerHTML = articleData.content;
        // let section = document.createElement("div");
        // section.classList.add("section");
        // content.appendChild(section);
        // section.textContent = articleData.content;
    }
    // Links
    let links = document.createElement("footer");
    links.classList.add("links", "unloaded", "collapsed");
    article.appendChild(links);
    MathJax.typeset([article]);
    return article;
}

class Article {
    constructor(key, type, label, article) {
        this.key = key;
        this.type = type;
        this.label = label;
        this.article = article;
    }
}

class ArticleType {

}

/**
 * Open an article.
 *
 * This creates an article in the article tab, registers the article in the
 * openArticles map and adds the open class to this article's labels.
 */
function openPrerenderedArticle(key) {
    // Ensure the article is loaded.
    let rendered = prerenders.get(key);
    if (rendered === undefined) {
        console.log("Article not loaded.");
        return;
    }
    // Add the article to the articles tab.
    rendered = rendered.cloneNode(true);
    //rendered.style.order = index;
    //index++;
    let articles = document.getElementById("articles");
    articles.appendChild(rendered);
    // MathJAX process element.
    MathJax.typeset([rendered]);
    // Add the article to the openArticles map.
    openArticles.insert(key, rendered);
    // Add the open class to the labels of this article.
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        if (key !== label.getAttribute("data-article")) continue;
        label.classList.add("open");
    }
}

function registerArticle() {

}

/**
 * Close an article.
 *
 * This deletes the article from the article tab, removes the article from the
 * openArticles map and updates the labels' class if necessary.
 */
function closeArticle(key, element) {
    // Delete the article from the article tab.
    let articles = element.parentElement;
    articles.removeChild(element);
    // Delete the article from the map.
    openArticles.delete(key, element);
    // Remove the open class from the labels if all of these articles have been closed.
    if (openArticles.count(key) !== 0) return;
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        if (key !== label.getAttribute("data-article")) continue;
        label.classList.remove("open");
    }
}

function getArticle(key) {
    let articles = document.getElementsByClassName("article");
    for (let article of articles) {
        if (article.getAttribute("data-article") === key) {
            return article;
        }
    }
    return null;
}

// Register click event listener on link category toggle
function setupArticles() {

    // document.addEventListener("click", event => {
    //
    //
    //
    //     let element = event.currentTarget;
    //     if (element.matches(".link-pane > .lh")) {
    //         let pane = element.parentElement;
    //         for
    //     }
    // });

    let contextPanels = document.getElementsByClassName("context-panel");
    for (let panel of contextPanels) {
        let parent = panel.parentElement;
        parent.addEventListener("contextmenu", event => {
            let box = parent.getBoundingClientRect();
            tooltip.style.top = box.bottom + "px";
            tooltip.style.left = box.right + "px";
            tooltip.style.display = "block";
        });
    }

    let progressBoxes = document.getElementsByClassName("progress-box");
    for (let box of progressBoxes) {
        box.addEventListener("contextmenu", event => {
            if (getSelection().type === "Range") {
                return;
            }
            event.preventDefault();
            let label = event.currentTarget;
            // placeProgressPanel(label); TODO key
        })
    }

}

/// Article class dialog

function lCl() {

}

async function openClassDialog(key, x, y) {
    let dialog = document.createElement("dialog");
    dialog.id = "class-dialog";
    dialog.style.right = "calc(100% - " + x + "px)";
    dialog.style.top = y + "px";
    dialog.style.justifySelf = "end";
    dialog.classList.add("loading");
    if (activeClassDialog !== null) {
        document.body.replaceChild(dialog, activeClassDialog);
    } else {
        document.body.appendChild(dialog);
    }
    activeClassDialog = dialog;
    activeClass = key;
    if (loadedClasses.has(key)) {
        let c = loadedClasses.get(key);
        populateClassDialog(dialog, c);
    } else {
        let c = await loadClass(key);
        if (dialog === activeClassDialog) { // Check that the dialog is still open.
            populateClassDialog(dialog, c);
        }
    }
}

function populateClassDialog(dialog, c) {
    let articles = c.articles;
    let list = document.createElement("ul");
    dialog.appendChild(list);
    for (const entry of Object.entries(articles)) {
        let key = entry[0];
        let article = entry[1];
        let dialogEntry = generateClassDialogEntry(key, article);
        list.appendChild(dialogEntry);
    }
    dialog.classList.remove("loading");
}

function generateClassDialogEntry(key, article) {
    let li = document.createElement("li");
    li.setAttribute("data-article", key);
    let keySpan = document.createElement("span");
    keySpan.classList.add("key");
    keySpan.textContent = key;
    li.appendChild(keySpan);
    let nameSpan = document.createElement("span");
    nameSpan.classList.add("name");
    nameSpan.textContent = article.names[0];
    MathJax.typeset([nameSpan]);
    li.appendChild(nameSpan);
    return li;
}

/// Labels

function setupLabels() {
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        // Click
        // label.addEventListener("click", event => {
        //     let label = event.currentTarget;
        //     let key = label.getAttribute("article-key");
        //     if (openArticles.has(key)) {
        //         closeArticle(key);
        //     } else {
        //         openArticle(key);
        //     }
        // });
        // Hovering
        label.addEventListener("hover", event => {

        })
        // Right click
        label.addEventListener("contextmenu", event => {
            let label = event.currentTarget;
            let key = label.getAttribute("data-article");
            lastClickedKey = key;
            if (getSelection().type === "Range") {
                return;
            }
            event.preventDefault();
            placeProgressPanel(key, label);
        })
    }
}

/// Progress

class Level {
    constructor(key, name, description, icon) {
        this.key = key;
        this.name = name;
        this.description = description;
        this.icon = icon;
    }
}

/**
 * A progress type. Currently only Level progress types are supported.
 */
class LevelProgressType {
    constructor(key, levels) {
        this.key = key;
        this.levels = levels;
    }
}

class ProgressState {
    constructor(level) {
        this.level = level;
        this.log = [];
    }
    setLevel(timestamp, level) {
        let entry = new ProgressLogEntry(timestamp, level);
        this.log.push(entry);
        this.level = level;
    }
}

class ProgressLogEntry {
    constructor(timestamp, level) {
        this.timestamp = timestamp;
        this.level = level;
    }
}

/// Progress window

function setupProgressWindow() {
    // Clicking outside the progress window closes it.
    document.addEventListener("click", event => {
        let selector = document.getElementById("progress-panel");
        if (selector.contains(event.target)) {
            return;
        }
        selector.style.display = "none";
    });

    let levels = document.getElementsByClassName("progress-level");
    for (let level of levels) {
        level.addEventListener("click", event => {
            if (lastClickedKey === null) return;
            let level = event.currentTarget;
            let l = level.getAttribute("data-level");
            setProgress(lastClickedKey, l, new Date());
            fillProgressLog(lastClickedKey);
        });
    }
}

function generateProgressDialog() {

}

function placeProgressPanel(key, element) {
    let bounds = element.getBoundingClientRect();
    let x = bounds.right;
    let y = bounds.bottom;
    let selector = document.getElementById("progress-panel");
    selector.style.left = x + "px";
    selector.style.top = y + "px";
    selector.style.display = "grid";
    fillProgressLog(key);
}

function fillProgressLog(key) {
    let log = document.getElementById("progress-log");
    log.replaceChildren();
    let articleProgress = progress.get(key);
    if (articleProgress !== undefined) {
        for (let entry of articleProgress.log) {
            let timestamp = new Date(entry.timestamp);
            let level = entry.level;
            let timestr = timestamp.getFullYear();
            timestr += "-";
            timestr += monthstr(timestamp.getMonth());
            timestr += "-";
            timestr += timestamp.getDay().toString().padStart(2, "0");
            timestr += " ";
            timestr += timestamp.getHours().toString().padStart(2, "0");
            timestr += ":";
            timestr += timestamp.getMinutes().toString().padStart(2, "0");
            timestr += "  ·  ";
            timestr += ((Date.now() - timestamp.getTime()) / (1000 * 60 * 60)).toFixed(1);
            timestr += "h ago";
            log.insertAdjacentHTML("afterbegin", `
            <div class="progress-log-entry">
              <div class="progress-log-icon level${level}"></div>${timestr}<div class="close-button">C</div>
            </div>
          `);
        }
    }
}

function monthstr(n) {
    switch (n) {
        case 0: return "Jan";
        case 1: return "Feb";
        case 2: return "Mar";
        case 3: return "Apr";
        case 4: return "Mai";
        case 5: return "Jun";
        case 6: return "Jul";
        case 7: return "Aug";
        case 8: return "Sep";
        case 9: return "Okt";
        case 10: return "Nov";
        case 11: return "Des";
    }
}

function setProgress(key, levelstr, timestamp) {
    let level = Number(levelstr);
    let progresst = progress.get(key);
    if (progresst === undefined) {
        progresst = new ProgressState(level);
        progress.set(key, progresst);
    } else {
        progresst.level = level;
    }
    let logEntry = new ProgressLogEntry(timestamp, level);
    progresst.log.push(logEntry);
    localStorage.setItem("progress", JSON.stringify(Array.from(progress.entries())));
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        if (key !== label.getAttribute("data-article")) {
            continue;
        }
        let progressBox = label.getElementsByClassName("progress-box")[0];
        let progress = progressBox.getElementsByClassName("progress")[0];

        //// TODO
        progress.classList.remove("level1", "level2", "level3", "level4", "level5");
        if (level === 0) {

        } else if (level === 1) {
            progress.classList.add("level1");
        } else if (level === 2) {
            progress.classList.add("level2");
        } else if (level === 3) {
            progress.classList.add("level3");
        } else if (level === 4) {
            progress.classList.add("level4");
        } else if (level === 5) {
            progress.classList.add("level5");
        }

        if (level === 5) {
            progress.classList.add("progress-completed");
        } else {
            progress.classList.remove("progress-completed");
        }

    }

    let articles = document.getElementsByClassName("article");
    for (let article of articles) {
        if (key !== article.getAttribute("data-article")) {
            continue;
        }
        for (let child of article.children) {
            if (!child.classList.contains("content")) {
                continue;
            }
            for (let child3 of child.children) {
                if (!child3.classList.contains("progress-box")) {
                    continue;
                }
                let child2 = child3.getElementsByClassName("progress")[0];
                //// TODO
                if (level === 0) {
                    child2.classList.remove("level1", "level2", "level3", "level4", "level5");
                } else if (level === 1) {
                    child2.classList.add("level1");
                } else if (level === 2) {
                    child2.classList.add("level2");
                } else if (level === 3) {
                    child2.classList.add("level3");
                } else if (level === 4) {
                    child2.classList.add("level4");
                } else if (level === 5) {
                    child2.classList.add("level5");
                }
                ////

            }
        }
    }

}

function updateProgress() {
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        let key = label.getAttribute("data-article");
        let p = progress.get(key);
        if (p === undefined) continue;
        let level = p.level;
        for (let child of label.children) {
            if (!child.classList.contains("progress-box")) {
                continue;
            }
            let child2 = child.getElementsByClassName("progress")[0];
            //// TODO
            child2.classList.remove("level1", "level2", "level3", "level4", "level5");
            if (level === 0) {

            } else if (level === 1) {
                child2.classList.add("level1");
            } else if (level === 2) {
                child2.classList.add("level2");
            } else if (level === 3) {
                child2.classList.add("level3");
            } else if (level === 4) {
                child2.classList.add("level4");
            } else if (level === 5) {
                child2.classList.add("level5");
            }

            if (level === 5) {
                child2.classList.add("progress-completed");
            } else {
                child2.classList.remove("progress-completed");
            }

            ////

        }
    }

    let articles = document.getElementsByClassName("article");
    for (let article of articles) {
        let key = article.getAttribute("data-article");
        let p = progress.get(key);
        if (p === undefined) continue;
        let level = p.level;
        for (let child of article.children) {
            if (!child.classList.contains("menu")) {
                continue;
            }
            for (let childh of child.children) {
                if (!childh.classList.contains("progress-box")) {
                    continue;
                }
                let child2 = childh.getElementsByClassName("progress")[0];
                //// TODO
                if (level === 0) {
                    child2.classList.remove("level1", "level2", "level3", "level4", "level5");
                } else if (level === 1) {
                    child2.classList.add("level1");
                } else if (level === 2) {
                    child2.classList.add("level2");
                } else if (level === 3) {
                    child2.classList.add("level3");
                } else if (level === 4) {
                    child2.classList.add("level4");
                } else if (level === 5) {
                    child2.classList.add("level5");
                }
                ////

            }
        }
    }
}

/// Toolbar

function setupToolbar() {
    document.getElementById("article-view-button").addEventListener("click", event => {
        document.querySelector("main").classList.remove("main-overview");
        document.querySelector("main").classList.add("main-details");
        document.querySelector("main").classList.remove("main-both");
        document.getElementById("overview-tab").style.display = "none";
        document.getElementById("article-tab").style.display = null;
    });

    document.getElementById("label-view-button").addEventListener("click", event => {
        document.querySelector("main").classList.add("main-overview");
        document.querySelector("main").classList.remove("main-details");
        document.querySelector("main").classList.remove("main-both");
        document.getElementById("overview-tab").style.display = null;
        document.getElementById("article-tab").style.display = "none";
    });

    document.getElementById("both-view-button").addEventListener("click", event => {
        document.querySelector("main").classList.remove("main-overview");
        document.querySelector("main").classList.remove("main-details");
        document.querySelector("main").classList.add("main-both");
        document.getElementById("overview-tab").style.display = null;
        document.getElementById("article-tab").style.display = null;
    });

    // document.getElementById("highlight-off-button").addEventListener("click", event => {
    //     let labels = document.getElementsByClassName("label");
    //     for (let label of labels) {
    //         label.classList.remove("lowlighted-label");
    //         label.classList.remove("highlighted-label");
    //     }
    // });
    //
    // document.getElementById("review-highlight-button").addEventListener("click", event => {
    //     let labels = document.getElementsByClassName("label");
    //     let now = Date.now();
    //     for (let label of labels) {
    //         let key = label.getAttribute("article-key");
    //         let progress = progress.get(key);
    //         let last = progress !== undefined ? progress.log[progress.log.length - 1] : undefined;
    //         let timestamp = null;
    //         if (last !== undefined) timestamp = last.timestamp;
    //         if (timestamp !== null && now - timestamp < 20000) { //TODO
    //             label.classList.add("lowlighted-label");
    //         } else {
    //             label.classList.add("highlighted-label")
    //             label.classList.remove("lowlighted-label");
    //         }
    //     }
    // });

    // document.getElementById("highlight-incomplete-button").addEventListener("click", event => {
    //     let labels = document.getElementsByClassName("label");
    //
    //     for (let label of labels) {
    //         let key = label.getAttribute("article-key");
    //         let progress = progress.get(key);
    //         if (progress !== undefined && progress.level === 5) {
    //             label.classList.add("lowlighted-label");
    //         } else {
    //             label.classList.remove("lowlighted-label");
    //         }
    //     }
    // });

    document.getElementById("open-all-button").addEventListener("click", event => {
        let labels = document.getElementsByClassName("label");
        for (let label of labels) {
            let key = label.getAttribute("data-article");
            openPrerenderedArticle(key);
        }
    });

    document.getElementById("close-all-button").addEventListener("click", event => {
        let articles = document.getElementById("articles");
        index = 0;
        while (articles.firstElementChild !== null) {
            let child = articles.firstElementChild;
            let key = child.getAttribute("data-article");
            closeArticle(key, child);
        }
    });
}

/// Tooltips

/**
 * Create a temporary tooltip that will be deleted when it is hidden.
 */
function createTooltip(target, content) {
    clearTooltip();
    let tooltip= document.createElement("div");
    tooltip.classList.add("tooltip");
    tooltip.classList.add("temp");
    if (typeof content === "string") {
        content = document.createTextNode(content);
    }
    tooltip.appendChild(content);
    let box = target.getBoundingClientRect();
    if (box.left < document.defaultView.innerWidth / 2.0) {
        tooltip.style.left = box.right + "px";
    } else {
        tooltip.style.right = "calc(100% - " + box.left + "px)";
        tooltip.style.justifySelf = "end";
    }
    tooltip.style.top = box.bottom + "px";
    tooltip.style.display = "block";
    target.appendChild(tooltip);
    activeTooltip = tooltip;
}

function setupStaticTooltips() {
    // document.addEventListener("mouseover", event => {
    //     let target = event.target; // TODO event.currentTarget
    //     if (target.matches(".link-panel > .lh")) {
    //         let typeName = target.getAttribute("data-type");
    //         let linkType = linkTypes.get(typeName);
    //         let origin = target.classList.contains("origin");
    //         let description = origin ? linkType.originDescription : linkType.targetDescription;
    //         createTooltip(target, description);
    //     } else {
    //         for (let child of target.children) {
    //             if (child.classList.contains("tooltip")) {
    //                 createTooltip(target, child)
    //                 break;
    //             }
    //         }
    //     }
    // });

    // document.addEventListener("mouseout", event => {
    //     let i = openTooltips.length;
    //     while (i > 0) {
    //         i--;
    //         let tooltip = openTooltips[i];
    //         if (tooltip.parentElement.matches(":hover")) {
    //             break;
    //         } else {
    //             openTooltips.pop();
    //         }
    //     }
    // });
}

document.addEventListener("DOMContentLoaded", event => {

    // Check localstorage progress
    let progressStored = localStorage.getItem("progress");
    if (progressStored !== null) {
        progress = new Map(JSON.parse(progressStored));
        updateProgress();
    } else {
        progress = new Map();
    }

    setupToolbar();

    setupLabels();

    setupStaticTooltips();

    setupProgressWindow();

    setupArticles();

});

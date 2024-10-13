
/**
 * The collection of currently opened articles.
 */
let openArticles = new Set();

/**
 * Always incremented when opening an article. Used to order last opened article
 * last.
 */
let index = 0;

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

class Level {
    constructor(key, name, description, icon) {
        this.key = key;
        this.name = name;
        this.description = description;
        this.icon = icon;
    }
}

/**
 * Read the article types, progress types and articles from data.
 */
function readData() {

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
        if (key !== label.getAttribute("article-key")) {
            continue;
        }
        for (let child of label.children) {
            if (!child.classList.contains("progress")) {
                continue;
            }

            //// TODO
            child.classList.remove("level1", "level2", "level3", "level4", "level5");
            if (level === 0) {

            } else if (level === 1) {
                child.classList.add("level1");
            } else if (level === 2) {
                child.classList.add("level2");
            } else if (level === 3) {
                child.classList.add("level3");
            } else if (level === 4) {
                child.classList.add("level4");
            } else if (level === 5) {
                child.classList.add("level5");
            }

            if (level === 5) {
                child.classList.add("progress-completed");
            } else {
                child.classList.remove("progress-completed");
            }

            ////

        }
    }

    let articles = document.getElementsByClassName("article");
    for (let article of articles) {
        if (key !== article.getAttribute("article-key")) {
            continue;
        }
        for (let child of article.children) {
            if (!child.classList.contains("content")) {
                continue;
            }
            for (let child of child.children) {
                if (!child.classList.contains("progress-box")) {
                    continue;
                }
                let child2 = child.getElementsByClassName("progress")[0];
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
        let key = label.getAttribute("article-key");
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
        let key = article.getAttribute("article-key");
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

function openArticle(key) {
    openArticles.add(key);
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        if (key === label.getAttribute("article-key")) {
            label.classList.add("opened-label");
            let articles = document.getElementById("articles");
            for (let child of articles.children) {
                let candidateKey = child.getAttribute("article-key");
                if (key === candidateKey) {
                    child.classList.add("opened-article");
                    index += 1;
                    child.style.order = index;
                }
            }
        }
    }
}

function closeArticle(key) {
    openArticles.delete(key);
    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        if (key === label.getAttribute("article-key")) {
            label.classList.remove("opened-label");
            let articles = document.getElementById("articles");
            for (let child of articles.children) {
                let candidateKey = child.getAttribute("article-key");
                if (key === candidateKey) {
                    child.classList.remove("opened-article");
                }
            }
        }
    }
}

function getArticle(key) {
    let articles = document.getElementsByClassName("article");
    for (let article of articles) {
        if (article.getAttribute("article-key") === key) {
            return article;
        }
    }
    return null;
}

function toggleArticleExpand(key) {
    let article = getArticle(key);
    if (article === null) return;
    if (article.classList.contains("article-closed")) {
        article.classList.remove("article-closed");
        article.classList.add("article-open");
        let content = article.getElementsByClassName("content")[0];
        content.style.display = null;
    } else {
        article.classList.remove("article-open");
        article.classList.add("article-closed");
        let content = article.getElementsByClassName("content")[0];
        content.style.display = "none";
    }
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

    document.getElementById("article-view-button").addEventListener("click", event => {
        document.getElementById("main").classList.remove("main-overview");
        document.getElementById("main").classList.add("main-details");
        document.getElementById("main").classList.remove("main-both");
        document.getElementById("label-tab").style.display = "none";
        document.getElementById("article-tab").style.display = null;
    });

    document.getElementById("label-view-button").addEventListener("click", event => {
        document.getElementById("main").classList.add("main-overview");
        document.getElementById("main").classList.remove("main-details");
        document.getElementById("main").classList.remove("main-both");
        document.getElementById("label-tab").style.display = null;
        document.getElementById("article-tab").style.display = "none";
    });

    document.getElementById("both-view-button").addEventListener("click", event => {
        document.getElementById("main").classList.remove("main-overview");
        document.getElementById("main").classList.remove("main-details");
        document.getElementById("main").classList.add("main-both");
        document.getElementById("label-tab").style.display = null;
        document.getElementById("article-tab").style.display = null;
    });

    document.getElementById("highlight-off-button").addEventListener("click", event => {
        let labels = document.getElementsByClassName("label");
        for (let label of labels) {
            label.classList.remove("lowlighted-label");
            label.classList.remove("highlighted-label");
        }
    });

    document.getElementById("review-highlight-button").addEventListener("click", event => {
        let labels = document.getElementsByClassName("label");
        let now = Date.now();
        for (let label of labels) {
            let key = label.getAttribute("article-key");
            let progress = progress.get(key);
            let last = progress !== undefined ? progress.log[progress.log.length - 1] : undefined;
            let timestamp = null;
            if (last !== undefined) timestamp = last.timestamp;
            if (timestamp !== null && now - timestamp < 20000) { //TODO
                label.classList.add("lowlighted-label");
            } else {
                label.classList.add("highlighted-label")
                label.classList.remove("lowlighted-label");
            }
        }
    });

    document.getElementById("highlight-incomplete-button").addEventListener("click", event => {
        let labels = document.getElementsByClassName("label");

        for (let label of labels) {
            let key = label.getAttribute("article-key");
            let progress = progress.get(key);
            if (progress !== undefined && progress.level === 5) {
                label.classList.add("lowlighted-label");
            } else {
                label.classList.remove("lowlighted-label");
            }
        }
    });

    let labels = document.getElementsByClassName("label");
    for (let label of labels) {
        // Click
        label.addEventListener("click", event => {
            let label = event.currentTarget;
            let key = label.getAttribute("article-key");
            if (openArticles.has(key)) {
                closeArticle(key);
            } else {
                openArticle(key);
            }
        });
        // Hovering
        label.addEventListener("hover", event => {

        })
        // Right click
        label.addEventListener("contextmenu", event => {
            let label = event.currentTarget;
            let key = label.getAttribute("article-key");
            lastClickedKey = key;
            if (getSelection().type === "Range") {
                return;
            }
            event.preventDefault();
            placeProgressPanel(key, label);
        })
    }

    let tooltips = document.getElementsByClassName("tooltip");
    for (let tooltip of tooltips) {
        let parent = tooltip.parentElement;
        parent.addEventListener("mouseover", event => {
            let box = parent.getBoundingClientRect();
            tooltip.style.top = box.bottom + "px";
            tooltip.style.left = box.right + "px";
            tooltip.style.display = "block";
        });
        parent.addEventListener("mouseout", event => {
            tooltip.style.display = "none";
        });
    }

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

    let closeButtons = document.getElementsByClassName("close-button");
    for (let button of closeButtons) {
        button.addEventListener("click", event => {
            let button = event.currentTarget;
            let article = button.parentElement.parentElement;
            let key = article.getAttribute("article-key");
            closeArticle(key);
        });
    }

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

    document.getElementById("open-all-button").addEventListener("click", event => {
        let labels = document.getElementsByClassName("label");
        for (let label of labels) {
            let key = label.getAttribute("article-key");
            openArticle(key);
        }
    });

    document.getElementById("close-all-button").addEventListener("click", event => {
        for (let open of openArticles) {
            closeArticle(open);
        }
    });

    let menus = document.getElementsByClassName("menu");
    for (let menu of menus) {
        menu.addEventListener("click", event => {
            let key = menu.parentElement.getAttribute("article-key");
            toggleArticleExpand(key);
        });
    }

});

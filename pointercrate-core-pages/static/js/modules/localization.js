class LanguageSelector {
    constructor(group) {
        this.group = $(group);

        this.activeLanguage = document.getElementById("active-language");
        this.preferenceCookie = this.activeLanguage.parentNode.dataset.cookie;

        // add selection listeners to language items
        Array.from(group.querySelectorAll("ul > li > a > span"))
            .map(language => {
                this.addSelectionListener(language.parentNode, "click");
            });
    }

    setLanguage(code) {
        let exp = new Date();
        exp.setFullYear(exp.getFullYear() + 1);

        document.cookie = `preference-${this.preferenceCookie}=${code}; expires=${exp.toUTCString()}; path=/;`;

        window.location.reload();
    }

    addSelectionListener(button, event) {
        console.log(button, event)
        let code = button.querySelector("[data-lang]").dataset.lang;

        button.addEventListener(event, () => {
            this.setLanguage(code);
        });
    }
}

import { FluentBundle } from "https://cdn.jsdelivr.net/npm/@fluent/bundle@0.18.0/esm/bundle.js";
import { FluentResource } from "https://cdn.jsdelivr.net/npm/@fluent/bundle@0.18.0/esm/resource.js";

window.fluentBundle = new FluentBundle(document.documentElement.lang);
window.loadedResources = [];

export function loadResource(resource) {
    if (window.loadedResources.includes(resource)) {
        return;
    }

    let xhr = new XMLHttpRequest();
    xhr.open("GET", `/static/ftl/${document.documentElement.lang}/${resource}.ftl`, false);
    xhr.send();

    let fluentResource = new FluentResource(xhr.responseText);

    window.fluentBundle.addResource(fluentResource);
    window.loadedResources.push(resource);
}

export function tr(resource, text_id) {
    loadResource(resource);

    let [id, attribute] = text_id.split(".");
    let message = window.fluentBundle.getMessage(id);

    return message ?
        attribute ? message.attributes[attribute] : message.value
        : undefined;
}

export function trp(resource, text_id, args) {
    loadResource(resource);

    let pattern = tr(resource, text_id);
    return window.fluentBundle.formatPattern(pattern, args);
}

$(window).on("load", function () {
    let languageSelectorGroup = document.getElementById("language-selector");

    if (languageSelectorGroup) {
        let languageSelector = new LanguageSelector(languageSelectorGroup);

        Array.from(document.querySelectorAll("span[data-lang]"))
            .map((element) => element.parentElement)
            .forEach((button) => languageSelector.addSelectionListener(button, "touchend"))
    }
});

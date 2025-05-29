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

// load a specific .ftl file
// the correct language is retrieved thanks to cookies
export function loadResource(resourceName) {
    if (window.loadedResources.includes(resourceName)) {
        return Promise.resolve();
    }

    return fetch(`/static/core/ftl/${resourceName}${document.location.pathname}`)
        .then(response => response.text())
        .then(text => {
            let resource = new FluentResource(text);

            window.fluentBundle.addResource(resource);
            window.loadedResources.push(resourceName);
        })
        .catch(error => console.error(error))
}

// utility function for loading a specific translation
export function tr(text_id) {
    let [id, attribute] = text_id.split(".");

    let message = window.fluentBundle.getMessage(id);

    return message ?
        attribute ? message.attributes[attribute] : message.value
        : undefined;
}

// utility function for loading a specific translation with variable text
export function trp(text_id, args) {
    let pattern = tr(text_id);
    return window.fluentBundle.formatPattern(pattern, args);
}

// once all of the fluent resources specified in the <head> of this
// page finish loading, this event will be dispatched
const resourcesLoadedEvent = new CustomEvent("fluentresourcesloaded");

$(window).on("load", function () {
    let languageSelectorGroup = document.getElementById("language-selector");

    if (languageSelectorGroup) {
        let languageSelector = new LanguageSelector(languageSelectorGroup);

        Array.from(document.querySelectorAll("span[data-lang]"))
            .map((element) => element.parentElement)
            .forEach((button) => languageSelector.addSelectionListener(button, "touchend"))
    }

    let resourcePromises = [];
    window.ftlResources.forEach((resource) => {
        resourcePromises.push(loadResource(resource));
    });
    
    Promise.all(resourcePromises).then(() => {
        document.dispatchEvent(resourcesLoadedEvent);
    });
});

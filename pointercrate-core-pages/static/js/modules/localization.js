class LanguageSelector {
    constructor(group) {
        this.group = $(group);

        this.activeLanguage = document.getElementById("active-language");
        this.preferenceCookie = this.activeLanguage.parentNode.dataset.cookie;

        // add selection listeners to language items
        Array.from(group.querySelectorAll("ul > li > a > span"))
            .map(language => {
                this.addSelectionListener(language.parentNode);
            });
    }

    setLanguage(code) {
        let exp = new Date();
        exp.setFullYear(exp.getFullYear() + 1);

        document.cookie = `preference-${this.preferenceCookie}=${code}; expires=${exp.toUTCString()}; path=/;`;

        window.location.reload();
    }

    addSelectionListener(button) {
        let code = button.querySelector("[data-lang]").dataset.lang;

        button.addEventListener("click", () => {
            this.setLanguage(code);
        })
    }
}

import { FluentBundle } from "https://cdn.jsdelivr.net/npm/@fluent/bundle@0.18.0/esm/bundle.js";
import { FluentResource } from "https://cdn.jsdelivr.net/npm/@fluent/bundle@0.18.0/esm/resource.js";

window.fluentBundle = new FluentBundle(document.documentElement.lang);

// load a specific .ftl file
// the correct language is retrieved thanks to cookies
export function loadResource(resource) {
    return fetch(`/static/core/ftl/${resource}`)
        .then(response => response.text())
        .then(text => {
            let resource = new FluentResource(text);

            window.fluentBundle.addResource(resource);
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
// page, this event will be dispatched
const resourcesLoadedEvent = new CustomEvent("fluentresourcesloaded");

$(document).ready(function () {
    new LanguageSelector(document.getElementById("language-selector"));

    let resourcePromises = [];
    window.ftlResources.forEach((resource) => {
        resourcePromises.push(loadResource(resource));
    })
    
    Promise.all(resourcePromises).then(() => {
        document.dispatchEvent(resourcesLoadedEvent);
    })
});

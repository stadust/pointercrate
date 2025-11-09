class LanguageSelector {
  constructor(group) {
    // add selection listeners to language items
    Array.from(group.querySelectorAll("[data-lang]")).map((language) => {
      this.addSelectionListener(language.parentNode, "click");
    });
  }

  setLanguage(code) {
    let exp = new Date();
    exp.setFullYear(exp.getFullYear() + 1);

    document.cookie = `preference-locale=${code}; expires=${exp.toUTCString()}; path=/;`;

    window.location.reload();
  }

  addSelectionListener(button, event) {
    let code = button.querySelector("[data-lang]").dataset.lang;

    button.addEventListener(event, () => {
      this.setLanguage(code);
    });
  }
}

import { FluentBundle } from "https://cdn.jsdelivr.net/npm/@fluent/bundle@0.18.0/esm/bundle.js";
import { FluentResource } from "https://cdn.jsdelivr.net/npm/@fluent/bundle@0.18.0/esm/resource.js";

window.fluentBundle = new FluentBundle(document.documentElement.lang);
window.requestedResources = [];

export function loadResource(category, resource) {
  // check if the resource file for this category/resource pair was already requested, so we
  // dont request it again
  if (
    window.requestedResources.some(
      (item) => item[0] == category && item[1] == resource
    )
  ) {
    return;
  }

  let xhr = new XMLHttpRequest();
  xhr.open(
    "GET",
    `/static${
      category ? "/" + category : ""
    }/ftl/${document.documentElement.lang.toLowerCase()}/${resource}.ftl`,
    false
  );
  xhr.send();
  window.requestedResources.push([category, resource]);

  if (xhr.status != 200) return console.error(xhr.status, xhr.statusText);

  let fluentResource = new FluentResource(xhr.responseText);
  window.fluentBundle.addResource(fluentResource);
}

export function tr(category, resource, text_id) {
  loadResource(category, resource);

  let [id, attribute] = text_id.split(".");
  let message = window.fluentBundle.getMessage(id);

  if (message) {
    return attribute ? message.attributes[attribute] : message.value;
  }
  return text_id;
}

export function trp(category, resource, text_id, args) {
  loadResource(category, resource);

  let pattern = tr(category, resource, text_id);
  return window.fluentBundle.formatPattern(pattern, args);
}

$(window).on("load", function () {
  let languageSelectorGroup = document.getElementById("language-selector");

  if (languageSelectorGroup) {
    let languageSelector = new LanguageSelector(languageSelectorGroup);

    Array.from(document.querySelectorAll("span[data-lang]"))
      .map((element) => element.parentElement)
      .forEach((button) =>
        languageSelector.addSelectionListener(button, "touchend")
      );
  }
});

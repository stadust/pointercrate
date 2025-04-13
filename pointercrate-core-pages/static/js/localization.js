class LanguageSelector {
    constructor(group) {
        this.group = $(group);

        // load languages list
        let ul = group.querySelectorAll("ul > li > a > span");

        this.activeLanguage = document.getElementById("active-language");
        this.activeFlag = group.querySelector("a > span > .flag-icon");

        this.languages = Array.from(ul);

        // hide the currently active language from the list
        Array.from(ul)
            .map(language => {
                if (language.dataset.lang == document.documentElement.lang) {
                    language.parentNode.style = "display: none";
                } else {
                    language.parentNode.style = "display: block";
                    this.addSelectionListener(language.parentNode);
                }
            });
        
        // update displayed active language
        this.activeLanguage.textContent = document.documentElement.lang.toUpperCase();

        this.languages.forEach(language => {
            if (language.dataset.lang == document.documentElement.lang) {
                this.activeFlag.style = `background-image: url("/static/demonlist/images/flags/${language.dataset.flag}.svg`;
            }
        })
    }

    setLanguage(code) {
        let exp = new Date();
        exp.setFullYear(exp.getFullYear() + 1);

        document.cookie = `preference-locale=${code}; expires=${exp.toUTCString()}; path=/;`;

        window.location.reload();
    }

    addSelectionListener(button) {
        let code = button.querySelector("[data-lang]").dataset.lang;

        button.addEventListener("click", () => {
            this.setLanguage(code);
        })
    }
}

$(document).ready(function () {
    new LanguageSelector(document.getElementById("language-selector"));
});

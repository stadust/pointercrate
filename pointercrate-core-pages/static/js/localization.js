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

$(document).ready(function () {
    new LanguageSelector(document.getElementById("language-selector"));
});

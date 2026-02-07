export const THEME = {
    LIGHT: "light",
    DARK: "dark",
}

export function currentTheme() {
    let theme = document.documentElement.dataset.theme;

    if (theme == THEME.DARK) return THEME.DARK;
    return THEME.LIGHT;
}

$(window).on("load", function () {
    let toggle = document.getElementById("theme-toggle");

    toggle.addEventListener("click", () => {
        let newTheme = currentTheme() == THEME.LIGHT ? THEME.DARK : THEME.LIGHT;

        let exp = new Date();
        exp.setFullYear(exp.getFullYear() + 1);

        document.cookie = `preference-theme=${newTheme}; expires=${exp.toUTCString()}; path=/;`;

        // disable css transitions
        let css = document.createElement("style");
        css.innerText = "*{ transition: none !important }";
        document.head.appendChild(css);

        document.documentElement.dataset.theme = newTheme;

        ThemedElement.toggleAll();
        
        // re-enable css transitions
        setTimeout(() => { document.head.removeChild(css); }, 1);
    })
});

// useful for elements whose colors cannot be modified directly through css
// e.g. discord panel (theme is based on a query param of iframe)
export class ThemedElement {
    static all = [];

    /**
     * @param {HTMLElement} html - the affected element
     * @param {function(HTMLElement, string): void} toggleFn - custom event handler for theme changes for this particular element
     */
    constructor(html, toggleFn) {
        this.html = html;
        this.toggleFn = toggleFn;
        
        ThemedElement.all.push(this);
    }

    static toggleAll() {
        for (const element of ThemedElement.all) {
            element.toggleFn(element.html, currentTheme());
        }
    }
}
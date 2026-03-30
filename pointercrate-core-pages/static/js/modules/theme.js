export const THEME = {
    LIGHT: "light",
    DARK: "dark",
}

export function currentTheme() {
    let theme = document.documentElement.dataset.theme;

    if (theme == THEME.DARK) return THEME.DARK;
    return THEME.LIGHT;
}

// disables transitions temporarily so color transitions dont make the site look funny when toggling light/dark mode
export function transitionTheme(newTheme, doc, head, toggleFn) {
    // disable css transitions
    const original = head.querySelector("style")?.textContent;
    let css = head.querySelector("style") ?? doc.createElement("style");
    css.textContent += "*{ transition: none !important }";
    head.appendChild(css);

    doc.documentElement.dataset.theme = newTheme;

    toggleFn();
    
    // re-enable css transitions
    requestAnimationFrame(() => original ? css.textContent = original : head.removeChild(css));
}

$(window).on("load", function () {
    let toggle = document.getElementById("theme-toggle");

    toggle.addEventListener("click", () => {
        let newTheme = currentTheme() == THEME.LIGHT ? THEME.DARK : THEME.LIGHT;

        let exp = new Date();
        exp.setFullYear(exp.getFullYear() + 1);

        document.cookie = `preference-theme=${newTheme}; expires=${exp.toUTCString()}; path=/;`;
        
        transitionTheme(newTheme, document, document.head, () => ThemedElement.toggleAll());
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
import { THEME, ThemedElement } from "/static/core/js/modules/theme.js";

$(window).on("load", function () {
    const gsi = document.querySelector(".g_id_signin");
    if (!gsi) return;

    new ThemedElement(
        gsi,
        (html, theme) => {
            html.dataset.theme = theme == THEME.LIGHT ? "outline" : "filled_black";

            google.accounts.id.renderButton(
                html,
                html.dataset,
            );
        },
    )
});
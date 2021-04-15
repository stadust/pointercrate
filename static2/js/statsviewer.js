import {StatsViewer} from "./modules/demonlist.mjs";

$(document).ready(function () {
    let worldMapWrapper = document.getElementById("world-map-wrapper");
    let worldMap = document.getElementById("world-map");

    window.statsViewer = new StatsViewer(document.getElementById("statsviewer"));
    window.statsViewer.initialize();

    document.addEventListener('scroll', () => {
        let scrollRatio = window.scrollY / worldMapWrapper.clientHeight;

        worldMapWrapper.style.filter = "blur(" + (scrollRatio * .25) + "rem)";
    });

    let nationIndicator = document.getElementById("current-nation");
    let currentlySelected = undefined;

    // TODO: investigate loading (ready is sometimes fired before page is loaded)
    for (let clickable of worldMap.contentDocument.querySelectorAll(".land, .island")) {
        clickable.addEventListener('click', () => {
            if(currentlySelected !== undefined)
                currentlySelected.classList.remove("selected");

            if(clickable !== currentlySelected) {
                statsViewer.updateQueryData('nation', clickable.id.toUpperCase());
                nationIndicator.innerText = clickable.getElementsByTagName("title")[0].innerHTML;

                currentlySelected = clickable;
                currentlySelected.classList.add("selected");
            } else {
                statsViewer.updateQueryData('nation', undefined);
                nationIndicator.innerText = 'International';
                currentlySelected = undefined;
            }
        })
    }
});
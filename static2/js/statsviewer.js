import {StatsViewer} from "./modules/demonlist.mjs";
import {Dropdown} from "./modules/form.mjs";

$(window).on("load", function () {
    let worldMapWrapper = document.getElementById("world-map-wrapper");
    let worldMap = document.getElementById("world-map");
    let svg = worldMap.contentDocument.children[0];

    window.statsViewer = new StatsViewer(document.getElementById("statsviewer"));
    window.statsViewer.initialize();

    new Dropdown(
        document
            .getElementById("continent-dropdown")
    ).addEventListener(selected => {
        if(selected === "All") {
            window.statsViewer.updateQueryData("continent", undefined);
            for(let continent of svg.getElementsByClassName("continent")) {
                continent.classList.add("selectable");
            }
        } else {
            window.statsViewer.updateQueryData("continent", selected);

            for(let continent of svg.getElementsByClassName("continent")) {
                if(continent.id !== selected.toLowerCase().replaceAll(' ', "-")) {
                    continent.classList.remove("selectable");
                } else {
                    continent.classList.add("selectable");
                }
            }
        }
    });

    document.addEventListener('scroll', () => {
        let scrollRatio = window.scrollY / worldMapWrapper.clientHeight;

        worldMapWrapper.style.filter = "blur(" + (scrollRatio * .25) + "rem)";
    });

    let currentlySelected = undefined;

    let zoom = 1.0;
    let translateX = 0;
    let translateY = 0;

    let mouseXrelativeToMap;
    let mouseYrelativeToMap;

    let isDragging = false;

    // approximation of line integral
    let dragDistance = 0;

    svg.addEventListener("mousedown", event => {
        isDragging = true;
    });

    svg.addEventListener("mousemove", event => {
        if (isDragging) {
            translateX += event.movementX / zoom;
            translateY += event.movementY / zoom;

            dragDistance += Math.sqrt(translateX * translateX + translateY * translateY);

            svg.style.transform = "scale(" + zoom + ") translate(" + translateX + "px, " + translateY + "px)";
        }

        mouseXrelativeToMap = event.clientX - svg.getBoundingClientRect().left + translateX * zoom;
        mouseYrelativeToMap = event.clientY - svg.getBoundingClientRect().top + translateY * zoom;
    });

    svg.addEventListener("mouseleave", event => {
        isDragging = false;
    });

    svg.addEventListener("mouseup", event => {
        isDragging = false;

        if (dragDistance >= 5) {
            function captureClick(event) {
                event.stopPropagation();
                svg.removeEventListener('click', captureClick, true);
            }

            svg.addEventListener(
                'click',
                captureClick,
                true
            );
        }

        dragDistance = 0;
    });

    svg.addEventListener('wheel', event => {
        if (event.shiftKey) {
            let unzoomedMouseX = mouseXrelativeToMap / zoom;
            let unzoomedMouseY = mouseYrelativeToMap / zoom;

            if(zoom - event.deltaY / Math.abs(event.deltaY) * .1 < 0.2)
                return;

            zoom -= event.deltaY / Math.abs(event.deltaY) * .1;

            let rezoomedMouseX = mouseXrelativeToMap / zoom;
            let rezoomedMouseY = mouseYrelativeToMap / zoom;

            translateX += (rezoomedMouseX - unzoomedMouseX);
            translateY += (rezoomedMouseY - unzoomedMouseY);

            svg.style.transform = "scale(" + zoom + ") translate(" + translateX + "px, " + translateY + "px)";
        }
    })

    // TODO: investigate loading (ready is sometimes fired before page is loaded)
    for (let clickable of worldMap.contentDocument.querySelectorAll(".land, .island")) {
        clickable.addEventListener('click', () => {
            if (isDragging)
                return false;

            if(!clickable.parentNode.classList.contains("selectable"))
                return false;

            if (currentlySelected !== undefined)
                currentlySelected.classList.remove("selected");

            if (clickable !== currentlySelected) {
                statsViewer.dropdown.select(clickable.id.toUpperCase());

                currentlySelected = clickable;
                currentlySelected.classList.add("selected");
            } else {
                statsViewer.dropdown.select('International');
                currentlySelected = undefined;
            }
        })
    }

    statsViewer.dropdown.addEventListener(selected => {
        if(currentlySelected !== undefined && currentlySelected.id.toUpperCase() === selected)
            return;

        if (currentlySelected !== undefined)
            currentlySelected.classList.remove("selected");

        if(selected === 'International') {
            currentlySelected = undefined;
        } else {
            currentlySelected = worldMap.contentDocument.getElementById(selected.toLowerCase());
            currentlySelected.classList.add("selected");
        }
    })
});

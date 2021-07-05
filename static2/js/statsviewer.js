import {populateSubdivisionDropdown, StatsViewer} from "./modules/demonlist.mjs";
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

    let subdivisionDropdown = new Dropdown(document.getElementById("subdivision-dropdown"));

    subdivisionDropdown.addEventListener(selected => {
        if(selected === 'None') {
            statsViewer.dropdown.select(statsViewer.queryData['nation']);
        } else {
            let countryCode = statsViewer.queryData['nation'];
            let targetElement = worldMap.contentDocument.getElementById(countryCode.toUpperCase() + "-" + selected.toUpperCase());

            if (targetElement !== currentlySelected)
                selectSubdivision(targetElement);
        }
    });

    function selectSubdivision(subdivision) {
        let subdivisionCode = subdivision.id.substring(3);
        let countryCode = subdivision.id.substring(0, 2);

        if(isDragging)
            return false;

        // bruh
        if(!subdivision.parentNode.parentNode.parentNode.classList.contains("selectable"))
            return false;

        if (currentlySelected !== undefined)
            currentlySelected.classList.remove("selected");

        if (subdivision !== currentlySelected) {
            if(currentlySelected === undefined || currentlySelected.id.substring(0, 2) !== countryCode) {
                statsViewer.dropdown.selectSilently(countryCode);

                populateSubdivisionDropdown(subdivisionDropdown, countryCode)
                    .then(() => subdivisionDropdown.select(subdivisionCode));
            } else {
                subdivisionDropdown.selectSilently(subdivisionCode);
            }

            statsViewer.updateQueryData2({nation: countryCode, subdivision: subdivisionCode});

            currentlySelected = subdivision;
            currentlySelected.classList.add("selected");
        } else {
            statsViewer.dropdown.selectSilently('International');
            statsViewer.updateQueryData2({nation: undefined, subdivision: undefined});

            subdivisionDropdown.reset();

            currentlySelected = undefined;
        }
    }

    for (let subdivision of worldMap.contentDocument.querySelectorAll(".land-with-states .state")) {
        subdivision.addEventListener('click', event => {
            // states are overlaid over the .land-with-states. We need to stop propagation as otherwise the
            // event handler on the .land-with-states is also run and it would select the entire country.
            event.stopPropagation();

            selectSubdivision(subdivision);
        });
    }

    // TODO: investigate loading (ready is sometimes fired before page is loaded)
    for (let clickable of worldMap.contentDocument.querySelectorAll(".land, .island, .land-with-states")) {
        clickable.addEventListener('click', () => {
            if (isDragging)
                return false;

            if(!clickable.parentNode.classList.contains("selectable"))
                return false;

            if (clickable !== currentlySelected) {
                statsViewer.dropdown.select(clickable.id.toUpperCase());
            } else {
                statsViewer.dropdown.select('International');
            }

            statsViewer.updateQueryData('subdivision', undefined);
        })
    }

    statsViewer.dropdown.addEventListener(selected => {
        // Selection unchanged
        if(currentlySelected === undefined && selected === 'International' || currentlySelected !== undefined && currentlySelected.id.toUpperCase() === selected)
            return;

        if (currentlySelected !== undefined)
            currentlySelected.classList.remove("selected");

        if(selected === 'International') {
            currentlySelected = undefined;
        } else {
            currentlySelected = worldMap.contentDocument.getElementById(selected.toLowerCase());
            currentlySelected.classList.add("selected");
        }

        // if 'countryCode == International' we send a nonsense request which results in a 404 and causes the dropdown to clear. That's exactly what we want, though.
        populateSubdivisionDropdown(subdivisionDropdown, selected);

        statsViewer.updateQueryData('subdivision', undefined);
    })
});

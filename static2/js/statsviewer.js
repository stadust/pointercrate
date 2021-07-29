import {populateSubdivisionDropdown, StatsViewer} from "./modules/demonlistv2.mjs";
import {Dropdown, findParentWithClass} from "./modules/formv2.mjs";

class InteractiveWorldMap {
    constructor() {
        this.wrapper = document.getElementById("world-map-wrapper");
        this.map = document.getElementById("world-map");
        this.svg = this.map.contentDocument.children[0];

        this.selectionListeners = [];
        this.deselectionListeners = [];

        this.zoom = 1;
        this.translate = {x: 0, y: 0};

        this.isDragging = false;
        this.dragDistance = 0; // approximate line integral of mouse movement

        this.relativeMousePosition = {x: 0, y: 0};
        this.lastTouchPosition = {x: 0, y: 0};

        this.setupTouchHandlers();
        this.setupMouseHandlers();

        this.currentlySelected = undefined;

        for (let subdivision of this.map.contentDocument.querySelectorAll(".land-with-states .state")) {
            subdivision.addEventListener('click', event => {
                // states are overlaid over the .land-with-states. We need to stop propagation as otherwise the
                // event handler on the .land-with-states is also run and it would select the entire country.
                event.stopPropagation();

                if(!findParentWithClass(subdivision, "continent").classList.contains("selectable"))
                    return;

                if (this.currentlySelected === subdivision) {
                    this._deselect();
                } else {
                    this._select(subdivision);
                }
            });
        }

        for (let clickable of this.map.contentDocument.querySelectorAll(".land, .island, .land-with-states")) {
            clickable.addEventListener('click', () => {
                if(!findParentWithClass(clickable, "continent").classList.contains("selectable"))
                    return;

                if(this.currentlySelected === clickable) {
                    this._deselect();
                }  else {
                    this._select(clickable);
                }
            })
        }
    }

    /**
     * Adds a selection listener to be called when a country/subdivision is selected by clicking
     *
     * @param listener callback (object, object?) -> void taking a nation and optionally a subdivision (both as objects with 'name' and 'code' fields)
     */
    addSelectionListener(listener) {
        this.selectionListeners.push(listener);
    }

    addDeselectionListener(listener) {
        this.deselectionListeners.push(listener);
    }

    highlightContinent(continentName) {
        if(continentName === undefined) {
            for(let continent of this.svg.getElementsByClassName("continent")) {
                continent.classList.add("selectable");
            }
        } else {
            for(let continent of this.svg.getElementsByClassName("continent")) {
                if(continent.id !== continentName.toLowerCase().replaceAll(' ', "-")) {
                    continent.classList.remove("selectable");
                } else {
                    continent.classList.add("selectable");
                }
            }
        }
    }

    resetContinentHighlight() {
        this.highlightContinent(undefined);
    }

    select(nation, subdivision) {
        let elementId = nation.toUpperCase();

        if(subdivision !== undefined)
            elementId += "-" + subdivision.toUpperCase();

        let element = this.svg.getElementById(elementId) || this.svg.getElementById(elementId.toLowerCase());

        if(element !== undefined)
            this._select(element, false);
    }

    deselectSubdivision() {
        if (this.currentlySelected === undefined || !this.currentlySelected.id.contains("-"))
            return;

        this.select(this.currentlySelected.id.substring(0, 2));
    }

    deselect() {
        this._deselect(false);
    }

    // private

    _select(clicked, fireEvents = true) {
        if(this.isDragging)
            return;

        if (this.currentlySelected !== undefined)
            this.currentlySelected.classList.remove("selected");

        this.currentlySelected = clicked;
        this.currentlySelected.classList.add("selected");

        let subdivisionCode = clicked.id.substring(3);
        let countryCode = clicked.id.substring(0, 2);

        if(fireEvents)
            for(let listener of this.selectionListeners)
                listener(countryCode.toUpperCase(), subdivisionCode === "" ? undefined : subdivisionCode.toUpperCase());
    }

    _deselect(fireEvents = true) {
        if(this.isDragging)
            return

        if(this.currentlySelected === undefined)
            return;

        this.currentlySelected.classList.remove("selected");
        this.currentlySelected = undefined;

        if(fireEvents)
            for(let listener of this.deselectionListeners)
                listener();
    }

    setLastPosFromTouchEvent(event) {
        this.lastTouchPosition.x = event.touches[0].pageX;
        this.lastTouchPosition.y = event.touches[0].pageY;
    }

    doDrag(deltaX, deltaY) {
        if(deltaX === undefined || deltaY === undefined)
            return;

        this.translate.x += deltaX / this.zoom;
        this.translate.y += deltaY / this.zoom;

        // TODO(patrick): press sure this is nonsense?
        this.dragDistance += Math.sqrt(this.translate.x * this.translate.x + this.translate.y * this.translate.y);

        this.svg.style.transform = "scale(" + this.zoom + ") translate(" + this.translate.x + "px, " + this.translate.y + "px)";
    }

    setupTouchHandlers() {
        this.svg.addEventListener("touchstart", event => {
            this.isDragging = event.touches.length === 1;

            if(this.isDragging) {
                this.setLastPosFromTouchEvent(event);

                event.preventDefault();
            }
        });

        this.svg.addEventListener("touchend", event => {
            this.isDragging = event.touches.length !== 1;

            if(this.isDragging) {
                this.setLastPosFromTouchEvent(event);

                event.preventDefault();
            }
        });

        this.svg.addEventListener("touchmove", event => {
            if(this.isDragging) {
                this.doDrag(event.touches[0].pageX - this.lastTouchPosition.x, event.touches[0].pageY - this.lastTouchPosition.y);

                this.setLastPosFromTouchEvent(event);

                event.preventDefault();
            }
        });
    }

    setupMouseHandlers() {
        document.addEventListener('scroll', () => {
            let scrollRatio = window.scrollY / this.wrapper.clientHeight;

            this.wrapper.style.filter = "blur(" + (scrollRatio * .25) + "rem)";
        });

        this.svg.addEventListener("mousedown", event => {
            this.isDragging = true;
        });

        this.svg.addEventListener("mousemove", event => {
            if (this.isDragging)
                this.doDrag(event.movementX, event.movementY);

            this.relativeMousePosition.x = event.clientX - this.svg.getBoundingClientRect().left + this.translate.x * this.zoom;
            this.relativeMousePosition.y = event.clientY - this.svg.getBoundingClientRect().top + this.translate.y * this.zoom;
        });

        this.svg.addEventListener("mouseleave", event => {
            this.isDragging = false;
        });

        this.svg.addEventListener("mouseup", event => {
            this.isDragging = false;

            if (this.dragDistance >= 5) {
                let captureClick = event => {
                    event.stopPropagation();
                    this.svg.removeEventListener('click', captureClick, true);
                };

                this.svg.addEventListener(
                    'click',
                    captureClick,
                    true
                );
            }

            this.dragDistance = 0;
        });

        this.svg.addEventListener('wheel', event => {
            if (event.shiftKey) {
                let unzoomedMouseX = this.relativeMousePosition.x / this.zoom;
                let unzoomedMouseY = this.relativeMousePosition.y / this.zoom;

                if(this.zoom - event.deltaY / Math.abs(event.deltaY) * .1 < 0.2)
                    return;

                this.zoom -= event.deltaY / Math.abs(event.deltaY) * .1;

                let rezoomedMouseX = this.relativeMousePosition.x / this.zoom;
                let rezoomedMouseY = this.relativeMousePosition.y / this.zoom;

                this.translate.x += (rezoomedMouseX - unzoomedMouseX);
                this.translate.y += (rezoomedMouseY - unzoomedMouseY);

                this.svg.style.transform = "scale(" + this.zoom + ") translate(" + this.translate.x + "px, " + this.translate.y + "px)";
            }
        })
    }
}

$(window).on("load", function () {
    let map = new InteractiveWorldMap();

    window.statsViewer = new StatsViewer(document.getElementById("statsviewer"));
    window.statsViewer.initialize();

    new Dropdown(
        document
            .getElementById("continent-dropdown")
    ).addEventListener(selected => {
        if(selected === "All") {
            window.statsViewer.updateQueryData("continent", undefined);
            map.resetContinentHighlight();
        } else {
            window.statsViewer.updateQueryData("continent", selected);
            map.highlightContinent(selected);
        }
    });

    let subdivisionDropdown = new Dropdown(document.getElementById("subdivision-dropdown"));

    subdivisionDropdown.addEventListener(selected => {
        if(selected === 'None') {
            map.deselectSubdivision();
            statsViewer.updateQueryData('subdivision', undefined);
        } else {
            let countryCode = statsViewer.queryData['nation'];

            map.select(countryCode, selected);
            statsViewer.updateQueryData2({nation: countryCode, subdivision: selected});
        }
    });

    statsViewer.dropdown.addEventListener(selected => {
        if(selected === 'International') {
            map.deselect();
        } else {
            map.select(selected);
        }

        // if 'countryCode == International' we send a nonsense request which results in a 404 and causes the dropdown to clear. That's exactly what we want, though.
        populateSubdivisionDropdown(subdivisionDropdown, selected);

        statsViewer.updateQueryData('subdivision', undefined);
    });

    map.addSelectionListener((countryCode, subdivisionCode) => {
        populateSubdivisionDropdown(subdivisionDropdown, countryCode).then(() => subdivisionDropdown.selectSilently(subdivisionCode));

        statsViewer.dropdown.selectSilently(countryCode);

        statsViewer.updateQueryData2({nation: countryCode, subdivision: subdivisionCode});
    });

    map.addDeselectionListener(() => {
        statsViewer.dropdown.selectSilently("International");
        subdivisionDropdown.clearOptions();
        statsViewer.updateQueryData2({nation: undefined, subdivision: undefined});
    });
});

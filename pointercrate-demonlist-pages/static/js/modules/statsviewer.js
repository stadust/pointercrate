import {
  getCountryFlag,
  getSubdivisionFlag,
  populateSubdivisionDropdown,
} from "/static/demonlist/js/modules/demonlist.js?v=4";
import {
  Dropdown,
  FilteredPaginator,
  findParentWithClass,
  get,
  Viewer,
} from "/static/core/js/modules/form.js?v=4";

export class StatsViewer extends FilteredPaginator {
  /**
   * Constructs a new StatsViewer
   *
   * @param {HTMLElement} html The container element of this stats viewer instance
   * @param statsviewerdata additional settings for this stats viewer
   */
  constructor(html, statsviewerdata) {
    super(
      "stats-viewer-pagination",
      statsviewerdata.entryGenerator,
      "name_contains"
    );

    this.endpoint = statsviewerdata.rankingEndpoint;
    // different from pagination endpoint here!
    this.retrievalEndpoint = statsviewerdata.retrievalEndpoint;
    this.currentLink = this.endpoint + "?" + $.param(this.queryData);

    this.html = html;
    this.output = new Viewer(
      html.getElementsByClassName("viewer-content")[0],
      this
    );

    this._name = document.getElementById("player-name");
    this._created = document.getElementById("created");
    this._beaten = document.getElementById("beaten");
    this._verified = document.getElementById("verified");
    this._published = document.getElementById("published");
    this._hardest = document.getElementById("hardest");
    this._score = document.getElementById("score");
    this._rank = document.getElementById("rank");
    this._amountBeaten = document.getElementById("stats");
    this._welcome = html.getElementsByClassName("viewer-welcome")[0];
    this._progress = document.getElementById("progress");
    this._content = html.getElementsByClassName("viewer-content")[0];

    let dropdownElement = html.getElementsByClassName("dropdown-menu")[0];

    if (dropdownElement !== undefined) {
      this.dropdown = new Dropdown(dropdownElement);
      this.dropdown.addEventListener((selected) => {
        if (selected === "International") {
          this.updateQueryData("nation", undefined);
        } else {
          this.updateQueryData("nation", selected);
        }
      });
    }
  }

  initialize() {
    return get("/api/v1/list_information/").then((data) => {
      this.list_size = data.data["list_size"];
      this.extended_list_size = data.data["extended_list_size"];

      super.initialize();
    });
  }

  setName(name, nationality) {
    if (nationality === null) {
      this._name.textContent = name;
    } else {
      while (this._name.lastChild) {
        this._name.removeChild(this._name.lastChild);
      }

      let nameSpan = document.createElement("span");
      nameSpan.style.padding = "0 8px";
      nameSpan.innerText = name;

      this._name.appendChild(
        getCountryFlag(nationality.nation, nationality.country_code)
      );
      this._name.appendChild(nameSpan);

      if (nationality.subdivision !== null) {
        this._name.appendChild(
          getSubdivisionFlag(
            nationality.subdivision.name,
            nationality.country_code,
            nationality.subdivision.iso_code
          )
        );
      } else {
        // needed for layout
        this._name.appendChild(document.createElement("span"));
      }
    }
  }

  setHardest(hardest) {
    if (this._hardest.lastChild)
      this._hardest.removeChild(this._hardest.lastChild);
    this._hardest.appendChild(
      hardest === undefined
        ? document.createTextNode("None")
        : this.formatDemon(hardest, "/demonlist/permalink/" + hardest.id + "/")
    );
  }

  setCompletionNumber(main, extended, legacy) {
    this._amountBeaten.textContent =
      main + " Main, " + extended + " Extended, " + legacy + " Legacy ";
  }

  onReceive(response) {
    super.onReceive(response);

    // Using currentlySelected is O.K. here, as selection via clicking li-elements is the only possibility (well, not for the nation based one, but oh well)!
    this._rank.innerText = this.currentlySelected.dataset.rank;
    this._score.innerHTML = this.currentlySelected.getElementsByTagName(
      "i"
    )[0].innerHTML;
  }

  formatDemon(demon, link) {
    var element;

    if (demon.position <= this.list_size) {
      element = document.createElement("b");
    } else if (demon.position <= this.extended_list_size) {
      element = document.createElement("span");
    } else {
      element = document.createElement("i");
      element.style.opacity = ".5";
    }

    if (link) {
      let a = document.createElement("a");
      a.href = link;
      a.textContent = demon.name;

      element.appendChild(a);
    } else {
      element.textContent = demon.name;
    }

    return element;
  }
}

export function formatInto(parent, childs) {
  while (parent.lastChild) {
    parent.removeChild(parent.lastChild);
  }

  if (childs.length) {
    for (let child of childs) {
      parent.appendChild(child);
      parent.appendChild(document.createTextNode(" - "));
    }

    // remove trailing dash
    parent.removeChild(parent.lastChild);
  } else {
    parent.appendChild(document.createTextNode("None"));
  }
}

export class InteractiveWorldMap {
  constructor() {
    this.wrapper = document.getElementById("world-map-wrapper");
    this.map = document.getElementById("world-map");
    this.svg = this.map.contentDocument.children[0];

    this.selectionListeners = [];
    this.deselectionListeners = [];

    this.zoom = 1;
    this.translate = { x: 0, y: 0 };

    this.isDragging = false;
    this.dragDistance = 0; // approximate line integral of mouse movement

    this.relativeMousePosition = { x: 0, y: 0 };
    this.lastTouchPosition = { x: 0, y: 0 };

    this.setupTouchHandlers();
    this.setupMouseHandlers();

    this.currentlySelected = undefined;

    for (let subdivision of this.map.contentDocument.querySelectorAll(
      ".land-with-states .state"
    )) {
      subdivision.addEventListener("click", (event) => {
        // states are overlaid over the .land-with-states. We need to stop propagation as otherwise the
        // event handler on the .land-with-states is also run and it would select the entire country.
        event.stopPropagation();

        if (
          !findParentWithClass(subdivision, "continent").classList.contains(
            "selectable"
          )
        )
          return;

        if (this.currentlySelected === subdivision) {
          this._deselect();
        } else {
          this._select(subdivision);
        }
      });
    }

    for (let clickable of this.map.contentDocument.querySelectorAll(
      ".land, .island, .land-with-states"
    )) {
      clickable.addEventListener("click", () => {
        if (
          !findParentWithClass(clickable, "continent").classList.contains(
            "selectable"
          )
        )
          return;

        if (this.currentlySelected === clickable) {
          this._deselect();
        } else {
          this._select(clickable);
        }
      });
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
    if (continentName === undefined) {
      for (let continent of this.svg.getElementsByClassName("continent")) {
        continent.classList.add("selectable");
      }
    } else {
      for (let continent of this.svg.getElementsByClassName("continent")) {
        if (continent.id !== continentName.toLowerCase().replaceAll(" ", "-")) {
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

    if (subdivision !== undefined) elementId += "-" + subdivision.toUpperCase();

    let element =
      this.svg.getElementById(elementId) ||
      this.svg.getElementById(elementId.toLowerCase());

    if (element !== undefined) this._select(element, false);
  }

  deselectSubdivision() {
    if (
      this.currentlySelected === undefined ||
      !this.currentlySelected.id.includes("-")
    )
      return;

    this.select(this.currentlySelected.id.substring(0, 2));
  }

  deselect() {
    this._deselect(false);
  }

  showSubdivisions() {
    for (let divided of this.map.contentDocument.querySelectorAll(
      ".land-with-states"
    )) {
      divided.classList.add("subdivided");
    }
  }

  hideSubdivisions() {
    for (let divided of this.map.contentDocument.querySelectorAll(
      ".land-with-states.subdivided"
    )) {
      divided.classList.remove("subdivided");
    }
  }

  // private

  _select(clicked, fireEvents = true) {
    if (this.isDragging) return;

    if (this.currentlySelected !== undefined)
      this.currentlySelected.classList.remove("selected");

    this.currentlySelected = clicked;
    this.currentlySelected.classList.add("selected");

    let subdivisionCode = clicked.id.substring(3);
    let countryCode = clicked.id.substring(0, 2);

    if (fireEvents)
      for (let listener of this.selectionListeners)
        listener(
          countryCode.toUpperCase(),
          subdivisionCode === "" ? undefined : subdivisionCode.toUpperCase()
        );
  }

  _deselect(fireEvents = true) {
    if (this.isDragging) return;

    if (this.currentlySelected === undefined) return;

    this.currentlySelected.classList.remove("selected");
    this.currentlySelected = undefined;

    if (fireEvents) for (let listener of this.deselectionListeners) listener();
  }

  setLastPosFromTouchEvent(event) {
    this.lastTouchPosition.x = event.touches[0].pageX;
    this.lastTouchPosition.y = event.touches[0].pageY;
  }

  doDrag(deltaX, deltaY) {
    if (deltaX === undefined || deltaY === undefined) return;

    this.translate.x += deltaX / this.zoom;
    this.translate.y += deltaY / this.zoom;

    // TODO(patrick): pretty sure this is nonsense?
    this.dragDistance += Math.sqrt(
      this.translate.x * this.translate.x + this.translate.y * this.translate.y
    );

    this.svg.style.transform =
      "scale(" +
      this.zoom +
      ") translate(" +
      this.translate.x +
      "px, " +
      this.translate.y +
      "px)";
  }

  setupTouchHandlers() {
    this.svg.addEventListener("touchstart", (event) => {
      this.isDragging = event.touches.length === 1;

      if (this.isDragging) {
        this.setLastPosFromTouchEvent(event);

        event.preventDefault();
      }
    });

    this.svg.addEventListener("touchend", (event) => {
      this.isDragging = event.touches.length !== 1;

      if (this.isDragging) {
        this.setLastPosFromTouchEvent(event);

        event.preventDefault();
      }
    });

    this.svg.addEventListener("touchmove", (event) => {
      if (this.isDragging) {
        this.doDrag(
          event.touches[0].pageX - this.lastTouchPosition.x,
          event.touches[0].pageY - this.lastTouchPosition.y
        );

        this.setLastPosFromTouchEvent(event);

        event.preventDefault();
      }
    });
  }

  setupMouseHandlers() {
    document.addEventListener("scroll", () => {
      let scrollRatio = window.scrollY / this.wrapper.clientHeight;

      this.wrapper.style.filter = "blur(" + scrollRatio * 0.25 + "rem)";
    });

    this.svg.addEventListener("mousedown", (event) => {
      this.isDragging = true;
    });

    this.svg.addEventListener("mousemove", (event) => {
      if (this.isDragging) this.doDrag(event.movementX, event.movementY);

      this.relativeMousePosition.x =
        event.clientX -
        this.svg.getBoundingClientRect().left +
        this.translate.x * this.zoom;
      this.relativeMousePosition.y =
        event.clientY -
        this.svg.getBoundingClientRect().top +
        this.translate.y * this.zoom;
    });

    this.svg.addEventListener("mouseleave", (event) => {
      this.isDragging = false;
    });

    this.svg.addEventListener("mouseup", (event) => {
      this.isDragging = false;

      if (this.dragDistance >= 5) {
        let captureClick = (event) => {
          event.stopPropagation();
          this.svg.removeEventListener("click", captureClick, true);
        };

        this.svg.addEventListener("click", captureClick, true);
      }

      this.dragDistance = 0;
    });

    this.svg.addEventListener("wheel", (event) => {
      if (event.shiftKey) {
        let unzoomedMouseX = this.relativeMousePosition.x / this.zoom;
        let unzoomedMouseY = this.relativeMousePosition.y / this.zoom;

        if (this.zoom - (event.deltaY / Math.abs(event.deltaY)) * 0.1 < 0.2)
          return;

        this.zoom -= (event.deltaY / Math.abs(event.deltaY)) * 0.1;

        let rezoomedMouseX = this.relativeMousePosition.x / this.zoom;
        let rezoomedMouseY = this.relativeMousePosition.y / this.zoom;

        this.translate.x += rezoomedMouseX - unzoomedMouseX;
        this.translate.y += rezoomedMouseY - unzoomedMouseY;

        this.svg.style.transform =
          "scale(" +
          this.zoom +
          ") translate(" +
          this.translate.x +
          "px, " +
          this.translate.y +
          "px)";
      }
    });
  }
}

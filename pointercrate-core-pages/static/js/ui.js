class DropDown {
  constructor(dropdown) {
    this.dropdown = dropdown;
    this.shown = false;
  }

  show(complete) {
    this.shown = true;
    stopAnimation(this.dropdown);
    slideDown(this.dropdown, 200, complete);

    DropDown.currentlyShown = this.dropdown.id;
  }

  hide(complete) {
    this.shown = false;
    stopAnimation(this.dropdown);
    slideUp(this.dropdown, 200, complete);

    DropDown.currentlyShown = undefined;
  }

  static showDropDown(id, complete) {
    var toShow = DropDown.getDropDown(id);

    if (DropDown.currentlyShown !== undefined) {
      DropDown.hideDropDown(DropDown.currentlyShown, function () {
        toShow.show(complete);
      });
    } else {
      toShow.show(complete);
    }
  }

  static hideDropDown(id, complete) {
    DropDown.getDropDown(id).hide(complete);
  }

  static toggleDropDown(id, complete) {
    if (DropDown.getDropDown(id).shown) {
      DropDown.hideDropDown(id, complete);
    } else {
      DropDown.showDropDown(id, complete);
    }
  }

  static getDropDown(id) {
    return DropDown.allDropDowns[id];
  }
}

DropDown.allDropDowns = {};

class Search {
  constructor(search) {
    this.search = search;
    this.input = search.getElementsByTagName("input")[0];
    this.searchDepth = search.dataset.searchDepth;

    if (typeof this.searchDepth === "undefined") {
      this.container = this.search.parentElement;
    } else {
      var src = this.search;

      for (var i = 0; i < this.searchDepth; ++i) {
        src = src.parentElement;
      }

      this.container = src;
    }

    this.registerHandlers();

    if (this.input.value) {
      this.updateResults(this.input.value.toLowerCase());
    }
  }

  updateResults(searchString) {
    var queries = searchString.split(";");
    for (var ul of this.container.getElementsByTagName("ul")) {
      ul.style.display = "none";
    }

    for (var li of this.container.getElementsByTagName("li")) {
      var content = li.innerText.toLowerCase();
      if (queries.some(function (q) { return content.includes(q); })) {
        li.style.display = "";
      } else {
        li.style.display = "none";
      }
    }

    for (var ul of this.container.getElementsByTagName("ul")) {
      ul.style.display = "";
    }
  }

  registerHandlers() {
    var update = function () {
      this.updateResults(this.input.value.toLowerCase());
    }.bind(this);

    this.input.addEventListener("keydown", update);
    this.input.addEventListener("change", update);
    this.input.addEventListener("input", update);
    this.input.addEventListener("paste", update);

    this.search.addEventListener("click", function (event) {
      if (event.target === this.search) {
        var xOff = event.pageX - this.search.getBoundingClientRect().left;

        if (xOff > this.input.offsetWidth) {
          this.input.value = "";
          this.input.dispatchEvent(new Event("change"));
        }
      }
    }.bind(this));
  }
}

Search.allSearchBars = [];

document.addEventListener("DOMContentLoaded", function () {
  // register dropdowns

  for (var elem of document.querySelectorAll(".dropdown")) {
    DropDown.allDropDowns[elem.id] = new DropDown(elem);
  }

  // register search elements

  for (var element of document.querySelectorAll(".js-search")) {
    Search.allSearchBars.push(new Search(element));
  }

  // close all dropdowns if clicked outside of dropdown

  document.addEventListener("click", function (event) {
    if (!event.target.closest("#lists")) {
      if (DropDown.currentlyShown) {
        // don't try to hide undefined
        DropDown.hideDropDown(DropDown.currentlyShown);
        // remove active class to remove highlight
        for (var elem of document.querySelectorAll(".js-toggle.active")) {
          elem.classList.remove("active");
        }
      }
    }
  });

  // toggle button event handling

  var toggleGroups = {};

  for (var elem of document.querySelectorAll(".js-toggle")) {
    var group = elem.dataset.toggleGroup;

    if (group !== undefined) {
      if (toggleGroups[group] === undefined) {
        toggleGroups[group] = [elem];
      } else {
        toggleGroups[group].push(elem);
      }
    }

    elem.addEventListener("click", function () {
      if (this.classList.contains("active")) {
        this.classList.remove("active");
      } else {
        for (var other of toggleGroups[group]) other.classList.remove("active");

        this.classList.add("active");
      }
    }.bind(elem));
  }
});

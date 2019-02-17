class DropDown {
  constructor(dropdown) {
    this.dropdown = dropdown;
    this.shown = false;
  }

  show(complete) {
    this.shown = true;
    this.dropdown.stop().slideDown({
      duration: 200,
      easing: "easeInOutQuad",
      complete: complete
    });

    DropDown.currentlyShown = this.dropdown[0].id;
  }

  hide(complete) {
    this.shown = false;
    this.dropdown.stop().slideUp({
      duration: 200,
      easing: "easeInOutQuad",
      complete: complete
    });

    DropDown.currentlyShown = undefined;
  }

  static showDropDown(id, complete) {
    var toShow = DropDown.getDropDown(id);

    if (DropDown.currentlyShown !== undefined) {
      DropDown.hideDropDown(DropDown.currentlyShown, () =>
        toShow.show(complete)
      );
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
    this.search = $(search);
    this.input = this.search.children("input");
    this.searchDepth = this.search.data("search-depth");

    if (typeof this.searchDepth === "undefined") {
      this.container = this.search.parent();
    } else {
      var src = this.search;

      for (var i = 0; i < this.searchDepth; ++i) {
        src = src.parent();
      }

      this.container = src;
    }

    this.target = this.container.find("li");
    this.registerHandlers();
  }

  updateResults(searchString) {
    // TODO: wait a bit here
    var queries = searchString.split(";");
    this.container.find("ul").each((i, l) => $(l).hide());

    this.target.each((index, element) => {
      element = $(element);
      var content = element.text().toLowerCase();
      if (queries.some(q => content.includes(q))) {
        element.show();
      } else {
        element.hide();
      }
    });

    this.container.find("ul").each((i, l) => $(l).show());
  }

  registerHandlers() {
    this.input.on("keydown change input paste", () => {
      this.updateResults(this.input.val().toLowerCase());
    });

    this.search.click(event => {
      if ($(event.target).is(this.search)) {
        let xOff = event.pageX - this.search.offset().left;

        if (xOff > this.input.width()) {
          this.input.val("");
          this.input.change();
        }
      }
    });
  }
}

Search.allSearchBars = [];

$(document).ready(function() {
  // register dropdowns

  $(".dropdown").each((i, elem) => {
    DropDown.allDropDowns[elem.id] = new DropDown($(elem));
  });

  // register dialogs

  $(".overlay").each((index, element) => {
    Dialog.allDialogs[element.id] = new Dialog(element);
  });

  // register search elements

  $(".search").each((index, element) => {
    Search.allSearchBars.push(new Search(element));
  });

  // toggle button event handling

  var toggleGroups = {};

  $(".js-toggle").each((i, elem) => {
    var obj = $(elem);
    var group = obj.data("toggle-group");

    if (group !== undefined) {
      if (toggleGroups[group] === undefined) {
        toggleGroups[group] = [obj];
      } else {
        toggleGroups[group].push(obj);
      }
    }

    obj.click(() => {
      if (obj.hasClass("active")) {
        obj.removeClass("active");
      } else {
        for (var other of toggleGroups[group]) other.removeClass("active");

        obj.addClass("active");
      }
    });
  });
});

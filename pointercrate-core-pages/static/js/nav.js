"use strict";

const TEMPLATES = {
  DROP_DOWN: "<div class='nav-drop-down'></div>",
  BUTTON:
    '<div class ="nav-item collapse-button nav-nohide"><div class="hamburger hover"><input type="checkbox" /><span></span><span></span><span></span></div></div>'
};

const NOHIDE_CLASSES = [".collapse-button", ".nav-icon"];

class NavigationBar {
  constructor(navigation) {
    this.extended = false;
    this.nav = $(navigation);

    let dropDown = document.createElement("div");
    dropDown.classList.add("nav-drop-down");

    for(let navGroup of navigation.getElementsByClassName("nav-group")) {
      for(let navItem of navGroup.childNodes) {
        if(!navItem.classList.contains("nav-nohide")) {
          let clone = navItem.cloneNode(true);

          for (let hovered of clone.getElementsByClassName("nav-hover-dropdown"))
            hovered.classList.remove("nav-hover-dropdown");

          dropDown.appendChild(clone);
        }
      }
    }
    navigation.appendChild(dropDown);

    console.log(dropDown);

    this.dropDown = $(dropDown);
    this.button = this.ensureButton();
    this.registerHandlers();
  }

  toggleDisplay(instant) {
    this.extended = !this.extended;

    if (this.extended) {
      this.dropDown.stop().slideDown({
        duration: instant ? 0 : 400,
        easing: "easeInOutQuad"
      });
    } else {
      this.dropDown.stop().slideUp({
        duration: instant ? 0 : 400,
        easing: "easeInOutQuad"
      });
    }
  }

  ensureButton() {
    var button = this.nav.find(".collapse-button");

    if (button.length === 0) {
      button = $(TEMPLATES.BUTTON);
      this.nav.append(button);
    }

    return button;
  }

  registerHandlers() {
    $(window).resize(() => {
      if (this.extended) {
        if ($(window).width() >= 1024) {
          this.dropDown.css("display", "none");
        } else if (this.dropDown.css("display") === "none") {
          this.dropDown.css("display", "flex");
        }
      }
    });

    this.button.click(() => this.toggleDisplay());
  }
}

NavigationBar.allNavigationBars = [];

$(document).ready(function() {
  for (let i = 0; i < NOHIDE_CLASSES.length; ++i) {
    $(NOHIDE_CLASSES[i]).each((index, element) =>
      $(element).addClass("nav-nohide")
    );
  }

  $("header nav.collapse").each((index, element) => {
    NavigationBar.allNavigationBars.push(new NavigationBar(element));
  });
});

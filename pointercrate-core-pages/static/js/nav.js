"use strict";

class NavigationBar {
  constructor(navigation) {
    let dropDown = navigation.getElementsByClassName("nav-drop-down")[0];

    navigation
      .getElementsByClassName("collapse-button")[0]
      .addEventListener("click", (e) => dropDown.classList.toggle("extended"));
  }
}

$(document).ready(function () {
  for (let navbar of document.querySelectorAll("header nav.collapse"))
    new NavigationBar(navbar);
});

class TabbedPane {
  constructor(htmlElement) {
    var tabs = htmlElement.querySelectorAll(".tab-selection .tab");

    this.tabs = {};
    this.panes = {};
    this.listeners = {};

    for (var tab of tabs) {
      let id = tab.dataset.tabId;
      let pane = htmlElement.querySelector(
        ".tab-display .tab-content[data-tab-id='" + id + "']"
      );

      this.panes[id] = pane;
      this.tabs[id] = tab;

      tab.addEventListener("click", () => this.selectPane(id));
    }
  }

  selectPane(id) {
    if (this.listeners[id] !== undefined) {
      for (let listener of this.listeners[id]) {
        listener();
      }
    }
    for (let paneId in this.panes) {
      if (paneId !== id) {
        this.panes[paneId].classList.remove("tab-content-active");
        this.tabs[paneId].classList.remove("tab-active");
      }
    }
    this.panes[id].classList.add("tab-content-active");
    this.tabs[id].classList.add("tab-active");
  }

  addSwitchListener(id, listener) {
    if (this.listeners[id] === undefined) {
      this.listeners[id] = [listener];
    } else {
      this.listeners[id].push(listener);
    }
  }
}

TABBED_PANES = {};

$(document).ready(function() {
  for (var tabbedPane of document.getElementsByClassName("tabbed")) {
    TABBED_PANES[tabbedPane.id] = new TabbedPane(tabbedPane);
  }
});

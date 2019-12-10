class TabbedPane {
  /**
   * Creates an instance of TabbedPane.
   *
   * @param {HTMLElement} htmlElement
   *
   * @memberof TabbedPane
   */
  constructor(htmlElement) {
    // TODO: figure out what the .tab-display class is used for and if we can remove it
    this.tabs = {};
    this.panes = {};
    this.listeners = {};

    for (var tab of htmlElement.querySelectorAll(".tab-selection .tab")) {
      // We first need to check if this tab really is for this TabbedPane, or for another tabbed pane nested within this one:
      let parent = tab.parentNode;
      while (!parent.classList.contains("tabbed")) parent = parent.parentNode;
      if (parent !== htmlElement) continue;

      let id = tab.dataset.tabId;

      this.tabs[id] = tab;

      tab.addEventListener("click", () => this.selectPane(id));
    }
    for (var pane of htmlElement.querySelectorAll(
      ".tab-display .tab-content"
    )) {
      let parent = pane.parentNode;
      while (!parent.classList.contains("tabbed")) parent = parent.parentNode;
      if (parent !== htmlElement) continue;

      this.panes[pane.dataset.tabId] = pane;
    }

    console.log(this.tabs);
    console.log(this.panes);
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

var TABBED_PANES = {};

$(document).ready(function() {
  for (var tabbedPane of document.getElementsByClassName("tabbed")) {
    TABBED_PANES[tabbedPane.id] = new TabbedPane(tabbedPane);
  }
});

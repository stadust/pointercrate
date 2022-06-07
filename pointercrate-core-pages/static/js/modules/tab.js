export class TabbedPane {
  /**
   * Creates an instance of TabbedPane.
   *
   * These work the following way: Inside a `.tab-display` element we have a `.tab-selection` and an arbitrary amount of `.tab-content`s.
   * At any point in time, exactly one of those contents is shown.
   * The selection contains one `.tab` for each content, and these tabs can be clicked to selected them.
   * Only the selected tab is ever shown.
   *
   * The currently selected tab is stored in local storage under the given storage key (if provided) and will be restored whenever the page is (re)loaded
   *
   * @param {HTMLElement} htmlElement
   *
   * @memberof TabbedPane
   */
  constructor(htmlElement, storageKey = null) {
    this.content = {};
    this.listeners = {};
    this.storageKey = storageKey;
    this.selectedId = null;

    for (var tab of htmlElement.querySelectorAll(".tab-selection .tab")) {
      // We first need to check if this tab really is for this TabbedPane, or for another tabbed pane nested within this one:
      if (containingTabbedPane(tab) !== htmlElement) continue;

      let id = tab.dataset.tabId;

      // we then try to find the associated pane
      let candiatePanes = htmlElement.querySelectorAll(
        ".tab-display .tab-content[data-tab-id='" + id + "']"
      );
      for (var pane of candiatePanes) {
        if (containingTabbedPane(pane) === htmlElement) {
          // found it!
          this.content[id] = [tab, pane];
          tab.addEventListener("click", () => this.selectPane(id));
          break;
        }
      }

      if (tab.classList.contains("tab-active")) this.selectedId = id;
    }

    if (storageKey !== null) {
      let stored = window.localStorage.getItem(storageKey);

      if (stored !== null) {
        this.selectPane(stored);
      }
    }
  }

  selectPane(id) {
    if (id in this.content && this.selectedId != id) {
      let [tab, pane] = this.content[id];
      let [oldTab, oldPane] = this.content[this.selectedId];

      oldTab.classList.remove("tab-active");
      oldPane.classList.remove("tab-content-active");

      tab.classList.add("tab-active");
      pane.classList.add("tab-content-active");

      if (this.listeners[id] !== undefined) {
        for (let listener of this.listeners[id]) {
          listener();
        }
      }

      this.selectedId = id;
      if (this.storageKey != null) {
        window.localStorage.setItem(this.storageKey, id);
      }
    }
  }

  addSwitchListener(id, listener) {
    if (this.listeners[id] === undefined) {
      this.listeners[id] = [listener];
    } else {
      this.listeners[id].push(listener);
    }
    // Directly fire if we're currently on the page. This is needed to fix the case where the selected tab is reloaded from local storage, which always happens before any listener is registered
    if (this.selectedId == id) {
      listener();
    }
  }
}

function containingTabbedPane(tabElement) {
  let parent = tabElement.parentNode;
  while (!parent.classList.contains("tab-display")) parent = parent.parentNode;
  return parent;
}

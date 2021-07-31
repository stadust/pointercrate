import { TabbedPane } from "./modules/tab.js";

$(document).ready(function() {
  new TabbedPane(
    document.getElementById("information-tabs"),
    "information-tabber-selection"
  );
  new TabbedPane(
    document.getElementById("changelog-tabs"),
    "changelog-tabber-selection"
  );
});

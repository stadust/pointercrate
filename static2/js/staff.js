import { initialize as initProfile } from "./account/profile.js";
import { initialize as initUsers } from "./account/users.js";
import { initialize as initRecords } from "./account/records.js";
import { TabbedPane } from "./modules/tab.mjs";

let usersInitialized = false;
let recordsInitialized = false;

$(document).ready(function() {
  var csrfTokenSpan = document.getElementById("chicken-salad-red-fish");
  var csrfToken = csrfTokenSpan.innerHTML;

  csrfTokenSpan.remove();

  let accountTabber = new TabbedPane(
    document.getElementById("account-tabber"),
    "account-tab-selection"
  );

  initProfile();

  accountTabber.addSwitchListener("2", () => {
    if (!usersInitialized) {
      initUsers(csrfToken);

      usersInitialized = true;
    }
  });

  accountTabber.addSwitchListener("3", () => {
    if (!recordsInitialized) {
      initRecords(csrfToken);

      recordsInitialized = true;
    }
  });
});

import { initialize as initProfile } from "./account/profile.js";
import { initialize as initUsers } from "./account/users.js";
import { initialize as initRecords, recordManager } from "./account/records.js";
import { initialize as initPlayers } from "./account/player.js";
import {
  initialize as initSubmitters,
  submitterManager,
} from "./account/submitter.js";
import { TabbedPane } from "./modules/tab.mjs";
import { initialize as initDemons, demonManager } from "./account/demon.js";

let usersInitialized = false;
let playersInitialized = false;

$(document).ready(function () {
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
    if (recordManager == null) {
      initRecords(csrfToken);
    }
  });

  accountTabber.addSwitchListener("4", () => {
    if (!playersInitialized) {
      initPlayers(csrfToken, accountTabber);
    }
    playersInitialized = true;
  });

  accountTabber.addSwitchListener("5", () => {
    if (!demonManager) {
      initDemons(csrfToken, accountTabber);
    }
  });

  accountTabber.addSwitchListener("6", () => {
    if (!submitterManager) {
      initSubmitters(csrfToken, accountTabber);
    }
  });
});

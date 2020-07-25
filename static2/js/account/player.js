import { generatePlayer } from "../modules/demonlist.mjs";
import {
  displayError,
  Form,
  valueMissing,
  FilteredPaginator,
  setupFormDialogEditor,
  PaginatorEditorBackend,
  setupDropdownEditor,
  Viewer,
} from "../modules/form.mjs";
import { recordManager, initialize as initRecords } from "./records.js";

export let playerManager;

class PlayerManager extends FilteredPaginator {
  constructor(csrfToken) {
    super("player-pagination", generatePlayer, "name_contains");

    this.output = new Viewer(
      this.html.parentNode.getElementsByClassName("viewer-content")[0],
      this
    );

    this._id = document.getElementById("player-player-id");
    this._name = document.getElementById("player-player-name");

    this._banned = setupDropdownEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "edit-player-banned",
      "banned",
      this.output,
      { true: true, false: false }
    );

    this._nationality = setupDropdownEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "edit-player-nationality",
      "nationality",
      this.output,
      { None: null }
    );

    this.initNameDialog(csrfToken);
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    this._id.innerText = this.currentObject.id;
    this._name.innerText = this.currentObject.name;

    this._banned.selectSilently(this.currentObject.banned.toString());

    if (this.currentObject.nationality) {
      this._nationality.selectSilently(
        this.currentObject.nationality.country_code
      );
    } else {
      this._nationality.selectSilently("None");
    }
  }

  initNameDialog(csrfToken) {
    let form = setupFormDialogEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "player-name-dialog",
      "player-name-pen",
      this.output
    );

    form.addValidators({
      "player-name-edit": {
        "Please provide a name for the player": valueMissing,
      },
    });
  }
}

function setupPlayerSearchPlayerIdForm() {
  var playerSearchByIdForm = new Form(
    document.getElementById("player-search-by-player-id-form")
  );
  var playerId = playerSearchByIdForm.input("search-player-id");

  playerId.addValidator(valueMissing, "Player ID required");
  playerSearchByIdForm.onSubmit(function (event) {
    playerManager
      .selectArbitrary(parseInt(playerId.value))
      .catch(displayError(playerSearchByIdForm));
  });
}

export function initialize(csrfToken, tabber) {
  setupPlayerSearchPlayerIdForm();

  playerManager = new PlayerManager(csrfToken);
  playerManager.initialize();

  document
    .getElementById("player-list-records")
    .addEventListener("click", () => {
      if (recordManager == null) {
        // Prevent race conditions between initialization request and the request caused by 'updateQueryData'
        initRecords(csrfToken).then(() => {
          recordManager.updateQueryData(
            "player",
            playerManager.currentObject.id
          );
          tabber.selectPane("3");
        });
      } else {
        recordManager.updateQueryData("player", playerManager.currentObject.id);
        tabber.selectPane("3"); // definitely initializes the record manager
      }
    });
}

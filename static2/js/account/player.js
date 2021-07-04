import {generatePlayer, getSubdivisionFlag} from "../modules/demonlist.mjs";
import {
  displayError,
  Form,
  valueMissing,
  FilteredPaginator,
  setupFormDialogEditor,
  PaginatorEditorBackend,
  setupDropdownEditor,
  Viewer, get,
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

    this._subdivision = setupDropdownEditor(
        new PaginatorEditorBackend(this, csrfToken, true),
        "edit-player-subdivision",
        "subdivision",
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

    let subdivisionList = this._subdivision.html.getElementsByTagName("ul")[0];

    // Kill all but the default entry
    while(subdivisionList.childNodes.length > 1)
      subdivisionList.removeChild(subdivisionList.lastChild);

    if (this.currentObject.nationality) {
      this._nationality.selectSilently(
        this.currentObject.nationality.country_code
      );

      get("/api/v1/nationalities/" + this.currentObject.nationality.country_code + "/subdivisions/").then(result => {
        this._subdivision.reset();

        for(let subdivision of result.data) {
          let flag = getSubdivisionFlag(subdivision.name, this.currentObject.nationality.country_code, subdivision.iso_code);

          flag.style.marginLeft = "-10px";
          flag.style.paddingRight = "10px";

          let li = document.createElement("li");

          li.className = "white hover";
          li.dataset.value = subdivision.iso_code;
          li.dataset.display = subdivision.name;
          li.appendChild(flag);
          li.appendChild(document.createTextNode(subdivision.name));

          this._subdivision.addLI(li);
        }

        if(!this.currentObject.nationality.subdivision) {
          this._subdivision.selectSilently("None");
        } else {
          this._subdivision.selectSilently(this.currentObject.nationality.subdivision.iso_code);
        }
      });
    } else {
      this._nationality.selectSilently("None");
      this._subdivision.selectSilently("None");
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
  playerSearchByIdForm.onSubmit(function () {
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

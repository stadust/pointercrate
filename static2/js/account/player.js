import { generatePlayer } from "../modules/demonlist.mjs";
import {
  patch,
  displayError,
  Form,
  Dropdown,
  FilteredViewer,
  valueMissing,
} from "../modules/form.mjs";

export let playerManager;

class PlayerManager extends FilteredViewer {
  constructor(csrfToken) {
    super("player-pagination", generatePlayer, "name_contains");

    this.currentPlayer = null;
    this.currentPlayerEtag = null;

    this._id = document.getElementById("player-player-id");
    this._name = document.getElementById("player-player-name");

    this._banned = new Dropdown(document.getElementById("edit-player-banned"));
    this._banned.addEventListener((selected) => {
      let banned = selected == "true";

      if (banned == this.currentPlayer.banned) return;

      patch(
        "/api/v1/players/" + this.currentPlayer.id + "/",
        {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": this.currentPlayerEtag,
        },
        { banned: banned }
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            this.refresh();
            this.onReceive(response);

            this.setSuccess("Player successfully edited!");
          }
        })
        .catch(displayError(this.errorOutput));
    });

    this._nationality = new Dropdown(
      document.getElementById("edit-player-nationality")
    );
    this._nationality.addEventListener((selected) => {
      if (selected == "None") selected = null;
      if (
        (selected == null && this.currentPlayer.nationality == null) ||
        (this.currentPlayer.nationality != null &&
          selected == this.currentPlayer.nationality.country_code)
      )
        return;
      patch(
        "/api/v1/players/" + this.currentPlayer.id + "/",
        {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": this.currentPlayerEtag,
        },
        { nationality: selected }
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            this.refresh();
            this.onReceive(response);

            this.setSuccess("Player nationality successfully edited!");
          }
        })
        .catch(displayError(this.errorOutput));
    });

    this.initNameDialog(csrfToken);
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    this.currentPlayer = response.data.data;
    this.currentPlayerEtag = response.headers["etag"];

    this._id.innerText = this.currentPlayer.id;
    this._name.innerText = this.currentPlayer.name;
    this._banned.select(this.currentPlayer.banned.toString());
    if (this.currentPlayer.nationality) {
      this._nationality.select(this.currentPlayer.nationality.country_code);
    } else {
      this._nationality.select("None");
    }
  }

  initNameDialog(csrfToken) {
    var editNameDialog = document.getElementById("player-name-dialog");
    var editNameForm = new Form(editNameDialog.getElementsByTagName("form")[0]);
    document.getElementById("player-name-pen").addEventListener("click", () => {
      $(editNameDialog.parentElement).show();
    });

    let name = editNameForm.input("player-name-edit");

    name.addValidator(valueMissing, "Please provide a name for the player");

    editNameForm.onSubmit(() => {
      patch(
        "/api/v1/players/" + this.currentPlayer.id + "/",
        {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": this.currentPlayerEtag,
        },
        editNameForm.serialize()
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            // directly refresh the record manager :pog:
            this.refresh();
            this.onReceive(response);
            this.setSuccess("Player name successfully edited!");
          }
          $(editNameDialog.parentElement).hide();
        })
        .catch(displayError(editNameForm.errorOutput));
    });
  }
}

function setupPlayerSearchPlayerIdForm() {
  var playerSearchByIdForm = new Form(
    document.getElementById("player-search-by-player-id-form")
  );
  var playerId = playerSearchByIdForm.input("player-player-id");

  playerId.addValidator(valueMissing, "Player ID required");
  playerSearchByIdForm.onSubmit(function (event) {
    playerManager
      .selectArbitrary(parseInt(playerId.value))
      .catch(displayError(playerSearchByIdForm.errorOutput));
  });
}

export function initialize(csrfToken) {
  setupPlayerSearchPlayerIdForm();

  playerManager = new PlayerManager(csrfToken);
  playerManager.initialize();
}

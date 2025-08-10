import {
  del,
  displayError,
  FilteredPaginator,
  Output,
  patch,
  post,
  put,
  get,
} from "/static/core/js/modules/form.js";
import {
  embedVideo,
  generatePlayer,
} from "/static/demonlist/js/modules/demonlist.js";
import { Paginator } from "/static/core/js/modules/form.js";
import { generateRecord } from "/static/demonlist/js/modules/demonlist.js";
import { loadResource, tr, trp } from "/static/core/js/modules/localization.js";

export let claimManager;

class ClaimManager extends FilteredPaginator {
  constructor() {
    super(
      "claim-pagination",
      (claim) => generateClaim(claim),
      "any_name_contains"
    );
  }

  onSelect(selected) {
    get(
      "/api/v1/records/?limit=1&status=APPROVED&player=" +
        selected.dataset.playerId,
      {}
    ).then((response) => {
      if (response.data.length === 0) {
        this.setError(
          trp("demonlist", "player", "claim-manager.claim-no-records", {
            ["player-id"]: selected.dataset.playerId,
          })
        );
        document.getElementById("claim-video").removeAttribute("src");
      } else {
        document.getElementById("claim-video").src = embedVideo(
          response.data[0].video
        );
        this.setError(null);
      }
    });
  }
}

function generateClaim(claim) {
  let li = document.createElement("li");

  li.classList.add("flex", "no-stretch");
  li.style.justifyContent = "space-between";

  let leftDiv = document.createElement("div");
  let userSpan = document.createElement("span");
  let playerSpan = document.createElement("span");

  let uname = document.createElement("b");
  uname.innerText = tr("demonlist", "player", "claim-listed-user") + " ";

  userSpan.appendChild(uname);
  userSpan.appendChild(
    document.createTextNode(claim.user.name + " (" + claim.user.id + ")")
  );

  let pname = document.createElement("b");
  pname.innerText = tr("demonlist", "player", "claim-listed-player") + " ";

  playerSpan.appendChild(pname);
  playerSpan.appendChild(
    document.createTextNode(claim.player.name + " (" + claim.player.id + ")")
  );

  leftDiv.appendChild(userSpan);
  leftDiv.appendChild(document.createElement("br"));
  leftDiv.appendChild(playerSpan);

  li.appendChild(leftDiv);

  let rightDiv = document.createElement("div");

  rightDiv.classList.add("flex");

  if (claim.verified) {
    li.classList.add("ok");
  } else {
    li.classList.add("consider");
    let button = makeButton("check");
    button.style.marginRight = "5px";

    button.addEventListener("click", (event) => {
      event.stopPropagation();
      patch(
        "/api/v1/players/" + claim.player.id + "/claims/" + claim.user.id,
        {},
        { verified: true }
      ).then(() => claimManager.refresh());
    });

    rightDiv.appendChild(button);
  }

  let deleteButton = makeButton("trash-alt");

  deleteButton.addEventListener("click", (event) => {
    event.stopPropagation();
    del(
      "/api/v1/players/" + claim.player.id + "/claims/" + claim.user.id,
      {}
    ).then(() => claimManager.refresh());
  });

  rightDiv.appendChild(deleteButton);
  li.appendChild(rightDiv);

  li.dataset.playerId = claim.player.id;

  return li;
}

function makeButton(faClass) {
  let a = document.createElement("a");
  a.classList.add("button", "blue", "hover");

  let i = document.createElement("i");
  i.classList.add("fas", "fa-" + faClass);

  a.appendChild(i);

  return a;
}

class ClaimPlayerPaginator extends FilteredPaginator {
  constructor() {
    super("claims-initiate-claim-pagination", generatePlayer, "name_contains");
  }

  onSelect(selected) {
    // No need to actually retrieve the player object!
    this.onReceive({
      status: 200,
      data: {
        data: {
          id: selected.dataset.id,
          name: selected.dataset.name,
        },
      },
      headers: {},
    });
  }
}

class ClaimedPlayerRecordPaginator extends Paginator {
  constructor(playerId) {
    super("claims-record-pagination", { player: playerId }, generateRecord);
  }

  onSelect(selected) {
    let recordId = selected.dataset.id;

    get("/api/v1/records/" + recordId + "/notes/")
      .then((response) => {
        if (Array.isArray(response.data) && response.data.length > 0) {
          this.setSuccess(null);

          while (this.successOutput.lastChild) {
            this.successOutput.removeChild(this.successOutput.lastChild);
          }

          let title = document.createElement("b");
          title.innerText = trp(
            "demonlist",
            "player",
            "claim-records.record-notes",
            {
              ["record-id"]: recordId,
            }
          );

          this.successOutput.appendChild(title);
          this.successOutput.appendChild(document.createElement("br"));

          for (let note of response.data) {
            let noteAuthor = document.createElement("i");
            noteAuthor.innerText = "(" + note.author + ") ";

            this.successOutput.appendChild(noteAuthor);
            this.successOutput.appendChild(
              document.createTextNode(note.content)
            );
          }

          this.successOutput.style.display = "block";
        } else {
          this.setSuccess(
            tr("demonlist", "player", "claim-records.record-notes-none")
          );
        }
      })
      .catch(displayError(this));
  }
}

export function initialize() {
  if (document.getElementById("claim-pagination")) {
    claimManager = new ClaimManager();
    claimManager.initialize();
  }

  let claimedPlayer = document.getElementById("claimed-player");

  let playerPaginator = new ClaimPlayerPaginator();
  playerPaginator.initialize();
  playerPaginator.addSelectionListener((selected) => {
    put("/api/v1/players/" + selected.id + "/claims/")
      .then(() => {
        window.location.reload();
      })
      .catch(displayError(playerPaginator));
  });

  document.getElementById("player-claim-pen").addEventListener("click", () => {
    playerPaginator.html.parentElement.style.display = "block";
  });

  let claimPanel = document.getElementById("claims-claim-panel");

  if (claimPanel) {
    let geolocationButton = document.getElementById(
      "claims-geolocate-nationality"
    );
    let output = new Output(claimPanel);

    let playerId = claimedPlayer.dataset.id;

    if (geolocationButton) {
      geolocationButton.addEventListener("click", () => {
        post("/api/v1/players/me/geolocate/")
          .then((response) => {
            let nationality = response.data;
            if (nationality.subdivision) {
              output.setSuccess(
                trp(
                  "demonlist",
                  "player",
                  "claim-geolocate.edit-success-subdivision",
                  {
                    ["nationality"]: nationality.nation,
                    ["subdivision"]: nationality.subdivision.name,
                  }
                )
              );
            } else {
              output.setSuccess(
                trp("demonlist", "player", "claim-geolocate.edit-success", {
                  ["nationality"]: nationality.nation,
                })
              );
            }
          })
          .catch(displayError(output));
      });
    }

    let lockSubmissionsCheckbox = document.getElementById(
      "lock-submissions-checkbox"
    );
    lockSubmissionsCheckbox.addEventListener("change", () => {
      patch(
        "/api/v1/players/" + playerId + "/claims/" + window.userId + "/",
        {},
        { lock_submissions: lockSubmissionsCheckbox.checked }
      )
        .then((_) => {
          output.setSuccess(
            tr("demonlist", "player", "claim-lock-submissions.edit-success")
          );
        })
        .catch(displayError(output));
    });

    let recordPaginator = new ClaimedPlayerRecordPaginator(playerId);
    recordPaginator.initialize();
  }
}

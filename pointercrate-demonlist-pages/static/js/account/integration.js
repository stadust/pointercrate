import {del, displayError, FilteredPaginator, Output, patch, post, put, get} from "/static/core/js/modules/form.js?v=4";
import {embedVideo, generatePlayer} from "/static/demonlist/js/modules/demonlist.js?v=4";
import {Paginator} from "/static/core/js/modules/form.js?v=4";;
import {generateRecord} from "/static/demonlist/js/modules/demonlist.js?v=4";

export let claimManager;

class ClaimManager extends FilteredPaginator {
    constructor() {
        super("claim-pagination", claim => generateClaim(claim), "any_name_contains");
    }

    onSelect(selected) {
        get("/api/v1/records/?limit=1&status=APPROVED&player=" + selected.dataset.playerId, {})
            .then(response => {
                if (response.data.length === 0)
                    this.setError("The claimed player does not have an approved record on the list")
                else
                    document.getElementById("claim-video").src = embedVideo(response.data[0].video);
            })
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
    uname.innerText = "Claim by user: ";

    userSpan.appendChild(uname);
    userSpan.appendChild(document.createTextNode(claim.user.name + " (" + claim.user.id + ")"));

    let pname = document.createElement("b");
    pname.innerText = "Claim on player: ";

    playerSpan.appendChild(pname);
    playerSpan.appendChild(document.createTextNode(claim.player.name + " (" + claim.player.id + ")"));

    leftDiv.appendChild(userSpan);
    leftDiv.appendChild(document.createElement("br"));
    leftDiv.appendChild(playerSpan);

    li.appendChild(leftDiv);

    let rightDiv = document.createElement("div");

    rightDiv.classList.add("flex");

    if (claim.verified) {
        li.style.backgroundColor = "rgba( 198, 255, 161, .3)";
    } else {
        li.style.backgroundColor = "rgba(142, 230, 230, .3)";
        let button = makeButton("check");
        button.style.marginRight = "5px";

        button.addEventListener("click", event => {
            event.stopPropagation();
            patch("/api/v1/players/" + claim.player.id + "/claims/" + claim.user.id, {}, {"verified": true}).then(() => claimManager.refresh());
        })

        rightDiv.appendChild(button);
    }

    let deleteButton = makeButton("trash-alt");

    deleteButton.addEventListener("click", event => {
        event.stopPropagation();
        del("/api/v1/players/" + claim.player.id + "/claims/" + claim.user.id, {}).then(() => claimManager.refresh());
    })

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
                    name: selected.dataset.name
                }
            },
            headers: {}
        })
    }
}

class ClaimedPlayerRecordPaginator extends Paginator {
    constructor(playerId) {
        super("claims-record-pagination", {player: playerId}, generateRecord);
    }

    onSelect(selected) {
        let recordId = selected.dataset.id;

        get("/api/v1/records/" + recordId + "/notes/")
            .then(response => {
                if(Array.isArray(response.data) && response.data.length > 0) {
                    this.setSuccess(null);

                    while(this.successOutput.lastChild) {
                        this.successOutput.removeChild(this.successOutput.lastChild);
                    }

                    let title = document.createElement("b");
                    title.innerText = "Notes for record " + recordId + ":";

                    this.successOutput.appendChild(title);
                    this.successOutput.appendChild(document.createElement("br"));

                    for (let note of response.data) {
                        let noteAuthor = document.createElement("i");
                        noteAuthor.innerText = "(" + note.author + ") ";

                        this.successOutput.appendChild(noteAuthor);
                        this.successOutput.appendChild(document.createTextNode(note.content));
                    }

                    this.successOutput.style.display = "block";
                } else {
                    this.setSuccess("No public notes on this record!");
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
    playerPaginator.addSelectionListener(selected => {
        put("/api/v1/players/" + selected.id + "/claims/")
            .then(() => {
                window.location.reload();
            }).catch(displayError(playerPaginator));
    })

    document.getElementById("player-claim-pen").addEventListener("click", () => {
        playerPaginator.html.parentElement.style.display = "block";
    });

    let claimPanel = document.getElementById("claims-claim-panel");

    if (claimPanel) {
        let geolocationButton = document.getElementById("claims-geolocate-nationality");
        let output = new Output(claimPanel);

        let playerId = claimedPlayer.dataset.id;

        geolocationButton.addEventListener("click", () => {
            post("/api/v1/players/" + playerId + "/geolocate")
                .then(response => {
                    let nationality = response.data;
                    if (nationality.subdivision) {
                        output.setSuccess("Set nationality to " + nationality.nation + "/" + nationality.subdivision.name);
                    } else {
                        output.setSuccess("Set nationality to " + nationality.nation);
                    }
                }).catch(displayError(output))
        })

        let lockSubmissionsCheckbox = document.getElementById("lock-submissions-checkbox");
        lockSubmissionsCheckbox.addEventListener("change", () => {
            patch("/api/v1/players/" + playerId + "/claims/" + window.userId + "/", {}, {"lock_submissions": lockSubmissionsCheckbox.checked}).then(_ => {
                output.setSuccess("Successfully applied changed")
            }).catch(displayError(output))
        });

        let recordPaginator = new ClaimedPlayerRecordPaginator(playerId);
        recordPaginator.initialize();
    }
}
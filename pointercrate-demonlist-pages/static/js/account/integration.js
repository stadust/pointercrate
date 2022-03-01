import {del, displayError, FilteredPaginator, get, Output, patch, post, put} from "/static/core/js/modules/form.js";
import {embedVideo, generatePlayer} from "/static/demonlist/js/modules/demonlist.js";

export let claimManager;

class ClaimManager extends FilteredPaginator {
    constructor(token) {
        super("claim-pagination", claim => generate_claim(token, claim), "any_name_contains");
    }

    onSelect(selected) {
        get("/api/v1/records/?limit=1&status=APPROVED&player=" + selected.dataset.playerId, {})
            .then(response => {
                if (response.data.length === 0)
                    this.setError("The claimed player does not have an approved records on the list")
                else
                    document.getElementById("claim-video").src = embedVideo(response.data[0].video);
            })
    }
}

function generate_claim(csrfToken, claim) {
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
            patch("/api/v1/players/" + claim.player.id + "/claims/" + claim.user.id, {"X-CSRF-TOKEN": csrfToken}, {"verified": true}).then(() => claimManager.refresh());
        })

        rightDiv.appendChild(button);
    }

    let deleteButton = makeButton("trash-alt");

    deleteButton.addEventListener("click", event => {
        event.stopPropagation();
        del("/api/v1/players/" + claim.player.id + "/claims/" + claim.user.id, {"X-CSRF-TOKEN": csrfToken}).then(() => claimManager.refresh());
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

export function initialize(csrfToken) {
    if (document.getElementById("claim-pagination")) {
        claimManager = new ClaimManager(csrfToken);
        claimManager.initialize();
    }

    let claimedPlayer = document.getElementById("claimed-player");

    let playerPaginator = new ClaimPlayerPaginator();
    playerPaginator.initialize();
    playerPaginator.addSelectionListener(selected => {
        put("/api/v1/players/" + selected.id + "/claims/", {"X-CSRF-TOKEN": csrfToken})
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

        geolocationButton.addEventListener("click", () => {
            let playerId = claimedPlayer.dataset.id;

            post("/api/v1/players/" + playerId + "/geolocate", {'X-CSRF-TOKEN': csrfToken})
                .then(response => {
                    let nationality = response.data;
                    if (nationality.subdivision) {
                        output.setSuccess("Set nationality to " + nationality.nation + "/" + nationality.subdivision.name);
                    } else {
                        output.setSuccess("Set nationality to " + nationality.nation);
                    }
                }).catch(displayError(output))
        })
    }
}
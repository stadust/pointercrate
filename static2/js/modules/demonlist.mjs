import {
  Dropdown,
  get,
  Form,
  post,
  valueMissing,
  typeMismatch,
  badInput,
  stepMismatch,
  rangeUnderflow,
  rangeOverflow,
  tooLong,
  findParentWithClass,
  FilteredPaginator,
} from "./form.mjs";
import { setupFormDialogEditor } from "./form.mjs";
import { Viewer } from "./form.mjs";

export function embedVideo(video) {
  if (!video) return;
  // welcome to incredibly fragile string parsing with stadust
  // see pointercrate::video::embed for a proper implementation of this

  if (video.startsWith("https://www.youtube")) {
    return "https://www.youtube.com/embed/" + video.substring(32);
  }

  if (video.startsWith("https://www.twitch")) {
    return (
      "https://player.twitch.tv/?autoplay=false&parent=pointercrate.com&video=" +
      video.substring(29)
    );
  }
}

export function initializeRecordSubmitter(csrf = null, submitApproved = false) {
  var submissionForm = new Form(document.getElementById("submission-form"));

  var demon = submissionForm.input("id_demon");
  var player = submissionForm.input("id_player");
  var progress = submissionForm.input("id_progress");
  var video = submissionForm.input("id_video");

  demon.addValidator(valueMissing, "Please specify a demon");

  player.addValidator(valueMissing, "Please specify a record holder");
  player.addValidator(
    tooLong,
    "Due to Geometry Dash's limitations I know that no player has such a long name"
  );

  progress.addValidator(valueMissing, "Please specify the record's progress");
  progress.addValidator(rangeUnderflow, "Record progress cannot be negative");
  progress.addValidator(
    rangeOverflow,
    "Record progress cannot be larger than 100%"
  );
  progress.addValidator(badInput, "Record progress must be a valid integer");
  progress.addValidator(stepMismatch, "Record progress mustn't be a decimal");

  video.addValidator(
    valueMissing,
    "Please specify a video so we can check the records validity"
  );
  video.addValidator(typeMismatch, "Please enter a valid URL");

  submissionForm.onSubmit(function (event) {
    let data = submissionForm.serialize();
    let headers = {};

    if (submitApproved) {
      data.status = "approved";
      headers["X-CSRF-TOKEN"] = csrf;
    }
    post("/api/v1/records/", headers, data)
      .then((response) => {
        submissionForm.setSuccess("Record successfully submitted");
        submissionForm.clear();
      })
      .catch((response) => submissionForm.setError(response.data.message)); // TODO: maybe specially handle some error codes
  });
}

export class StatsViewer extends FilteredPaginator {
  /**
   * Constructs a new StatsViewer
   *
   * @param {HtmlElement} html The container element of this stats viewer instance
   */
  constructor(html) {
    super(
      "stats-viewer-pagination",
      generateStatsViewerPlayer,
      "name_contains"
    );

    // different from pagination endpoint here!
    this.retrievalEndpoint = "/api/v1/players/";

    this.html = html;
    this.output = new Viewer(
      html.getElementsByClassName("viewer-content")[0],
      this
    );

    this._name = document.getElementById("player-name");
    this._created = document.getElementById("created");
    this._beaten = document.getElementById("beaten");
    this._verified = document.getElementById("verified");
    this._published = document.getElementById("published");
    this._hardest = document.getElementById("hardest");
    this._score = document.getElementById("score");
    this._rank = document.getElementById("rank");
    this._amountBeaten = document.getElementById("amount-beaten");
    this._amountLegacy = document.getElementById("amount-legacy");
    this._welcome = html.getElementsByClassName("viewer-welcome")[0];
    this._progress = document.getElementById("progress");
    this._content = html.getElementsByClassName("viewer-content")[0];

    this.dropdown = new Dropdown(
      html.getElementsByClassName("dropdown-menu")[0]
    );
    this.dropdown.addEventListener((selected) => {
      if (selected == "International") {
        this.updateQueryData("nation", undefined);
      } else {
        this.updateQueryData("nation", selected);
      }
    });
  }

  onReceive(response) {
    super.onReceive(response);

    this._rank.innerHTML = this.currentlySelected.dataset.rank;
    this._score.innerHTML = this.currentlySelected.getElementsByTagName(
      "i"
    )[0].innerHTML;

    var playerData = response.data.data;

    if (playerData.nationality == null) {
      this._name.textContent = playerData.name;
    } else {
      let flagClass =
        "flag-icon-" + playerData.nationality.country_code.toLowerCase();

      let span = document.createElement("span");
      span.classList.add("flag-icon", flagClass);
      span.title = playerData.nationality.nation;

      while (this._name.lastChild) {
        this._name.removeChild(this._name.lastChild);
      }

      this._name.textContent = playerData.name + " ";
      this._name.appendChild(span);
    }

    formatDemonsInto(this._created, playerData.created);
    formatDemonsInto(this._published, playerData.published);
    formatDemonsInto(this._verified, playerData.verified);

    let beaten = playerData.records.filter((record) => record.progress == 100);

    beaten.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));

    let legacy = beaten.filter(
      (record) => record.demon.position > window.extended_list_length
    ).length;
    let extended = beaten.filter(
      (record) =>
        record.demon.position > window.list_length &&
        record.demon.position <= window.extended_list_length
    ).length;

    formatRecordsInto(this._beaten, beaten);

    this._amountBeaten.textContent =
      beaten.length - legacy - extended + " ( + " + extended + " )";
    this._amountLegacy.textContent = legacy;

    var hardest = playerData.verified
      .concat(beaten.map((record) => record.demon))
      .reduce((acc, next) => (acc.position > next.position ? next : acc), {
        position: 34832834,
        name: "None",
      });

    this._hardest.textContent = hardest.name || "None";

    var non100Records = playerData.records
      .filter((record) => record.progress != 100)
      .sort((r1, r2) => r1.progress - r2.progress);

    formatRecordsInto(this._progress, non100Records);
  }
}

export function setupPlayerSelectionEditor(
  backend,
  paginatorId,
  buttonId,
  output
) {
  let paginator = new FilteredPaginator(
    paginatorId,
    generatePlayer,
    "name_contains"
  );

  let form = setupFormDialogEditor(
    backend,
    findParentWithClass(paginator.html, "dialog").id,
    buttonId,
    output
  );

  let playerName = form.inputs[0];

  playerName.addValidator(valueMissing, "Please provide a player name");

  paginator.initialize();
  paginator.addSelectionListener((selected) => {
    playerName.value = selected.name;
    form.html.requestSubmit();
  });
}

export function generatePlayer(player) {
  var li = document.createElement("li");
  var b = document.createElement("b");
  var b2 = document.createElement("b");

  li.className = "white";

  if (player.banned) {
    li.style.backgroundColor = "rgba(255, 161, 174, .3)";
  } else {
    li.style.backgroundColor = "rgba( 198, 255, 161, .3)";
  }

  li.dataset.name = player.name;
  li.dataset.id = player.id;

  b2.appendChild(document.createTextNode(player.id));

  if (player.nationality) {
    var span = document.createElement("span");

    span.className =
      "flag-icon flag-icon-" + player.nationality.country_code.toLowerCase();

    li.appendChild(span);
    li.appendChild(document.createTextNode(" "));
  }

  li.appendChild(b);
  li.appendChild(document.createTextNode(player.name + " - "));
  li.appendChild(b2);

  return li;
}

export function generateDemon(demon) {
  let li = document.createElement("li");
  let b = document.createElement("b");

  li.dataset.id = demon.id;

  b.innerText = "#" + demon.position + " - ";

  li.appendChild(b);
  li.appendChild(
    document.createTextNode(demon.name + " (ID: " + demon.id + ")")
  );
  li.appendChild(document.createElement("br"));
  li.appendChild(document.createTextNode("by " + demon.publisher.name));

  return li;
}

export function generateRecord(record) {
  var li = document.createElement("li");
  var recordId = document.createElement("b");

  li.className = "white";
  li.dataset.id = record.id;

  switch (record.status) {
    case "approved":
      li.style.backgroundColor = "rgba( 198, 255, 161, .3)";
      break;
    case "rejected":
      li.style.backgroundColor = "rgba(255, 161, 174, .3)";
      break;
    case "submitted":
      li.style.backgroundColor = "rgba(255, 255, 161, .3)";
      break;
    case "under consideration":
      li.style.backgroundColor = "rgba(142, 230, 230, .3)";
      break;
    default:
      break;
  }

  recordId.appendChild(document.createTextNode("Record #" + record.id));

  li.appendChild(recordId);
  li.appendChild(document.createElement("br"));
  li.appendChild(
    document.createTextNode(record.player.name + " (" + record.player.id + ")")
  );
  li.appendChild(document.createElement("br"));
  li.appendChild(
    document.createTextNode(record.progress + "% on " + record.demon.name)
  );
  li.appendChild(document.createElement("br"));

  return li;
}

function generateStatsViewerPlayer(player) {
  var li = document.createElement("li");
  var b = document.createElement("b");
  var i = document.createElement("i");

  li.className = "white hover";
  li.dataset.id = player.id;
  li.dataset.rank = player.rank;

  b.appendChild(document.createTextNode("#" + player.rank + " "));
  i.appendChild(document.createTextNode(player.score.toFixed(2)));

  if (player.nationality) {
    var span = document.createElement("span");

    span.className =
      "flag-icon flag-icon-" + player.nationality.country_code.toLowerCase();

    li.appendChild(span);
    li.appendChild(document.createTextNode(" "));
  }

  li.appendChild(b);
  li.appendChild(document.createTextNode(player.name));
  li.appendChild(i);

  return li;
}

function formatDemon(demon, link) {
  var element;

  if (demon.position <= window.list_length) {
    element = document.createElement("b");
  } else if (demon.position <= window.extended_list_length) {
    element = document.createElement("span");
  } else {
    element = document.createElement("i");
  }

  if (link) {
    let a = document.createElement("a");
    a.href = link;
    a.textContent = demon.name;

    element.appendChild(a);
  } else {
    element.textContent = demon.name;
  }

  return element;
}

function formatDemonsInto(element, demons) {
  while (element.lastChild) {
    element.removeChild(element.lastChild);
  }

  if (demons.length) {
    for (var demon of demons) {
      element.appendChild(
        formatDemon(demon, "/demonlist/" + demon.position + "/")
      );
      element.appendChild(document.createTextNode(" - "));
    }
    element.removeChild(element.lastChild);
  } else {
    element.appendChild(document.createTextNode("None"));
  }
}

function formatRecordsInto(element, records) {
  while (element.lastChild) {
    element.removeChild(element.lastChild);
  }

  console.log("record thingy");

  if (records.length) {
    for (var record of records) {
      let demon = formatDemon(record.demon, record.video);
      if (record.progress != 100) {
        demon.appendChild(
          document.createTextNode(" (" + record.progress + "%)")
        );
      }
      element.appendChild(demon);
      element.appendChild(document.createTextNode(" - "));
    }
    element.removeChild(element.lastChild);
  } else {
    element.appendChild(document.createTextNode("None"));
  }
}

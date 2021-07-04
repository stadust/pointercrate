import {
  Dropdown,
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
  Viewer,
  setupFormDialogEditor, FormDialog, setupEditorDialog, get,
} from "./form.mjs";

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

export function initializeTimeMachine() {
  let formHtml = document.getElementById("time-machine-form");

  if(formHtml === null)
    return;

  var timeMachineForm = new Form(formHtml);

  var inputs = ['year', 'month', 'day', 'hour', 'minute', 'second'].map(name => timeMachineForm.input("time-machine-" + name));

  for(let input of inputs) {
    input.addValidator(input => input.dropdown.selected !== undefined, "Please specify a value");
  }

  var offset = new Date().getTimezoneOffset();
  var offsetHours = Math.abs(offset) / 60;
  var offsetMinutes = Math.abs(offset) % 60;

  const MONTHS  = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];

  timeMachineForm.onSubmit(() => {
    let when = inputs[0].value + "-"
        + ("" + (MONTHS.indexOf(inputs[1].value) + 1)).padStart(2, '0') + "-"
        + ("" + inputs[2].value).padStart(2, '0') + "T"
        + ("" + inputs[3].value).padStart(2, '0') + ":"
        + ("" + inputs[4].value).padStart(2, '0') + ":"
        + ("" + inputs[5].value).padStart(2, '0') + (offsetHours < 0 ? "%2B" : "-") + (offsetHours + "").padStart(2, "0") + ":" + (offsetMinutes + "").padStart(2, "0");

    document.cookie = "when=" + when;
    gtag('event', 'time-machine-usage', {'event-category': 'demonlist', 'label': when});

    window.location = "/demonlist/";
  })
}

export function initializeRecordSubmitter(csrf = null, submitApproved = false) {
  var submissionForm = new Form(document.getElementById("submission-form"));

  var demon = submissionForm.input("id_demon");
  var player = submissionForm.input("id_player");
  var progress = submissionForm.input("id_progress");
  var video = submissionForm.input("id_video");

  demon.addValidator(input => input.dropdown.selected !== undefined, "Please specify a demon");

  let holderSelector = new PlayerSelectionDialog("submission-holder-dialog");
  document.getElementById("record-submitter-holder-pen").addEventListener('click', () => holderSelector.open().then(data => player.value = data.player));

  player.addValidator(input => input.value !== undefined, "Please specify a record holder");
  player.addValidator(
    input => input.value === undefined || input.value.length <= 50,
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

  submissionForm.onInvalid(() => gtag('event', 'record-submit-failure-frontend', {'event-category': 'demonlist'}));
  submissionForm.onSubmit(function () {
    let data = submissionForm.serialize();
    let headers = {};

    if (submitApproved) {
      data.status = "approved";
      headers["X-CSRF-TOKEN"] = csrf;
    }
    post("/api/v1/records/", headers, data)
      .then(() => {
        submissionForm.setSuccess("Record successfully submitted");
        submissionForm.clear();
        gtag('event', 'record-submit-success', {'event-category': 'demonlist'});
      })
      .catch((response) =>  {
        switch(response.data.code) {
          case 40401:
            demon.errorText = response.data.message;
            break;
          case 42218:
            player.errorText = response.data.message;
            break;
          case 42215:
          case 42220:
            progress.errorText = response.data.message;
            break;
          case 42222:
          case 42223:
          case 42224:
          case 42225:
            video.errorText = response.data.message;
            break;
          default:
            submissionForm.setError(response.data.message)
        }
        gtag('event', 'record-submit-failure-backend', {'event-category': 'demonlist'});
      }); // TODO: maybe specially handle some error codes
  });
}

export class StatsViewer extends FilteredPaginator {
  /**
   * Constructs a new StatsViewer
   *
   * @param {HTMLElement} html The container element of this stats viewer instance
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
      if (selected === "International") {
        this.updateQueryData("nation", undefined);
      } else {
        this.updateQueryData("nation", selected);
      }
    });
  }

  initialize() {
    return get("/api/v1/list_information/").then(data => {
      this.list_size = data.data['list_size'];
      this.extended_list_size = data.data['extended_list_size'];

      super.initialize()
    });
  }

  onReceive(response) {
    super.onReceive(response);

    // Using currentlySelected is O.K. here, as selection via clicking li-elements is the only possibility!
    this._rank.innerHTML = this.currentlySelected.dataset.rank;
    this._score.innerHTML = this.currentlySelected.getElementsByTagName(
      "i"
    )[0].innerHTML;

    var playerData = response.data.data;

    if (playerData.nationality == null) {
      this._name.textContent = playerData.name;
    } else {
      while (this._name.lastChild) {
        this._name.removeChild(this._name.lastChild);
      }

      let nameSpan = document.createElement("span");
      nameSpan.style.padding = "0 8px";
      nameSpan.innerText = playerData.name;

      this._name.appendChild(getCountryFlag(playerData.nationality.nation, playerData.nationality.country_code));
      this._name.appendChild(nameSpan);

      if (playerData.nationality.subdivision !== null) {
        this._name.appendChild(getSubdivisionFlag(playerData.nationality.subdivision.name, playerData.nationality.country_code, playerData.nationality.subdivision.iso_code));
      } else {
        this._name.appendChild(document.createElement("span"));
      }
    }

    this.formatDemonsInto(this._created, playerData.created);
    this.formatDemonsInto(this._published, playerData.published);
    this.formatDemonsInto(this._verified, playerData.verified);

    let beaten = playerData.records.filter((record) => record.progress == 100);

    beaten.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));

    let legacy = beaten.filter(
      (record) => record.demon.position > this.extended_list_size
    ).length;
    let extended = beaten.filter(
      (record) =>
        record.demon.position > this.list_size &&
        record.demon.position <= this.extended_list_size
    ).length;

    let verifiedExtended = playerData.verified.filter(demon => demon.position <= this.extended_list_size && demon.position > this.list_size).length;
    let verifiedLegacy = playerData.verified.filter(demon => demon.position > this.extended_list_size).length;

    this.formatRecordsInto(this._beaten, beaten);

    this._amountBeaten.textContent =
      (beaten.length - legacy - extended + playerData.verified.length - verifiedExtended - verifiedLegacy) + " ( + " + (extended + verifiedExtended) + " )";
    this._amountLegacy.textContent = legacy + verifiedLegacy;

    let hardest = playerData.verified
      .concat(beaten.map((record) => record.demon))
      .reduce((acc, next) => (acc.position > next.position ? next : acc), {name: "None", position: 321321321321});

    if(this._hardest.lastChild)
      this._hardest.removeChild(this._hardest.lastChild);
    this._hardest.appendChild(hardest.name === "None" ? document.createTextNode("None") : this.formatDemon(hardest, "/demonlist/permalink/" + hardest.id + "/"));

    let non100Records = playerData.records
      .filter((record) => record.progress != 100)
      .sort((r1, r2) => r1.progress - r2.progress);

    this.formatRecordsInto(this._progress, non100Records);
  }

  formatDemon(demon, link) {
    var element;

    if (demon.position <= this.list_size) {
      element = document.createElement("b");
    } else if (demon.position <= this.extended_list_size) {
      element = document.createElement("span");
    } else {
      element = document.createElement("i");
      element.style.opacity = ".5";
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

  formatDemonsInto(element, demons) {
    while (element.lastChild) {
      element.removeChild(element.lastChild);
    }

    if (demons.length) {
      for (var demon of demons) {
        element.appendChild(
            this.formatDemon(demon, "/demonlist/permalink/" + demon.id + "/")
        );
        element.appendChild(document.createTextNode(" - "));
      }
      element.removeChild(element.lastChild);
    } else {
      element.appendChild(document.createTextNode("None"));
    }
  }

  formatRecordsInto(element, records) {
    while (element.lastChild) {
      element.removeChild(element.lastChild);
    }

    if (records.length) {
      for (var record of records) {
        let demon = this.formatDemon(record.demon, "/demonlist/permalink/" + record.demon.id + "/");
        if (record.progress !== 100) {
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
}

export function getCountryFlag(title, countryCode) {
  let countrySpan = document.createElement("span");
  countrySpan.classList.add("flag-icon");
  countrySpan.title = title;
  countrySpan.style.backgroundImage = "url(/static2/images/flags/" + countryCode.toLowerCase() + ".svg";
  return countrySpan;
}

export function getSubdivisionFlag(title, countryCode, subdivisionCode) {
  let stateSpan = document.createElement("span");
  stateSpan.classList.add("flag-icon");
  stateSpan.title = title;
  stateSpan.style.backgroundImage = "url(/static2/images/flags/" + countryCode.toLowerCase() + "/" + subdivisionCode.toLowerCase() + ".svg";
  return stateSpan;
}

export class PlayerSelectionDialog extends FormDialog {
  constructor(dialogId) {
    super(dialogId);

    let paginator = new FilteredPaginator(
        dialogId + "-pagination",
        generatePlayer,
        "name_contains"
    );

    let playerName = this.form.inputs[0];

    playerName.addValidator(valueMissing, "Please provide a player name");

    paginator.initialize();
    paginator.addSelectionListener((selected) => {
      playerName.value = selected.name;
      this.form.html.requestSubmit();
    });
  }
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
    li.appendChild(getCountryFlag(player.nationality.nation, player.nationality.country_code));
    li.appendChild(document.createTextNode(" "));
  }

  li.appendChild(b);
  li.appendChild(document.createTextNode(player.name));
  li.appendChild(i);

  return li;
}

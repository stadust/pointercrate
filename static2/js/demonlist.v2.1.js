function generatePlayer(player) {
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

class StatsViewer extends FilteredPaginator {
  constructor() {
    super("stats-viewer-pagination", generatePlayer, "name_contains");

    this.html = $("#statsviewer");
    this._name = this.html.find("#player-name");
    this._created = this.html.find("#created");
    this._beaten = this.html.find("#beaten");
    this._verified = this.html.find("#verified");
    this._published = this.html.find("#published");
    this._hardest = this.html.find("#hardest");
    this._score = this.html.find("#score");
    this._rank = this.html.find("#rank");
    this._amountBeaten = this.html.find("#amount-beaten");
    this._amountLegacy = this.html.find("#amount-legacy");
    this._current = this.html.find("#name");
    this._welcome = this.html.find(".viewer-welcome");
    this._progress = this.html.find("#progress");
    this._content = this.html.find(".viewer-content");

    document
      .getElementById("show-stats-viewer")
      .addEventListener("click", () => this.initialize());

    this.dropdown = new Dropdown(
      this.html[0].getElementsByClassName("dropdown-menu")[0]
    );
    this.dropdown.addEventListener(li =>
      this.updateQueryData("nation", li.dataset.code)
    );
  }

  onSelect(selected) {
    makeRequest(
      "GET",
      "/api/v1/players/" + selected.dataset.id + "/",
      this.errorOutput,
      jsonData => {
        this.onReceive(jsonData);

        this._rank.text(selected.dataset.rank);
        this._score.text(selected.getElementsByTagName("i")[0].innerHTML);
      }
    );
  }

  onReceive(response) {
    var playerData = response.responseJSON.data;

    if (playerData.nationality == null) {
      this._name.text(playerData.name);
    } else {
      this._name.html(
        playerData.name +
          "&nbsp;<span class = 'flag-icon flag-icon-" +
          playerData.nationality.country_code.toLowerCase() +
          "' title = '" +
          playerData.nationality.nation +
          "'></span>"
      );
    }

    this._current.text(playerData.name);

    this.setFields(
      playerData.created,
      playerData.published,
      playerData.verified,
      playerData.records
    );

    this._welcome.hide(100);
    this._content.show(100);
  }

  setFields(created, published, verified, records) {
    this._created.html(formatDemons(created) || "None");
    this._published.html(formatDemons(published) || "None");
    this._verified.html(formatDemons(verified) || "None");

    let beaten = records.filter(record => record.progress == 100);

    let legacy = beaten.filter(
      record => record.demon.position > window.extended_list_length
    ).length;
    let extended = beaten.filter(
      record =>
        record.demon.position > window.list_length &&
        record.demon.position <= window.extended_list_length
    ).length;

    this._beaten.html(formatRecords(beaten) || "None");
    this._amountBeaten.text(
      beaten.length - legacy - extended + " ( + " + extended + " )"
    );
    this._amountLegacy.text(legacy);

    var hardest = verified
      .concat(beaten.map(record => record.demon))
      .reduce((acc, next) => (acc.position > next.position ? next : acc), {
        position: 34832834,
        name: "None"
      });

    this._hardest.text(hardest.name || "None");

    var non100Records = records
      .filter(record => record.progress != 100)
      .sort((r1, r2) => r1.progress - r2.progress)
      .map(record => formatRecord(record) + " (" + record.progress + "%)")
      .join(", ");

    this._progress.html(non100Records || "None");
  }
}

$(document).ready(function() {
  window.statsViewer = new StatsViewer();
  var submissionForm = new Form(document.getElementById("submission-form"));

  var demon = submissionForm.input("id_demon");
  var player = submissionForm.input("id_player");
  var progress = submissionForm.input("id_progress");
  var video = submissionForm.input("id_video");
  var note = submissionForm.input("submit-note");

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

  submissionForm.onSubmit(function(event) {
    $.ajax({
      method: "POST",
      url: "/api/v1/records/",
      contentType: "application/json",
      dataType: "json",
      data: JSON.stringify({
        demon: demon.value,
        player: player.value,
        video: video.value,
        progress: parseInt(progress.value),
        note: note.value
      }),
      error: data => submissionForm.setError(data.responseJSON.message),
      success: () => {
        submissionForm.setSuccess("Record successfully submitted");

        player.value = "";
        progress.value = "";
        video.value = "";
        demon.value = "";
        note.value = "";
      }
    });
  });
});

function formatRecords(records) {
  return records.map(formatRecord).join(", ");
}

function formatRecord(record) {
  let link =
    '<a target=blank href = "' +
    record.video +
    '">' +
    record.demon.name +
    "</a>";
  let demon = record.demon;

  if (demon.position <= window.list_length) {
    return "<b>" + link + "</b>";
  } else if (demon.position <= window.extended_list_length) {
    return link;
  } else {
    return "<i>" + link + "</i>";
  }
}

function formatDemon(demon) {
  if (demon.position <= window.list_length) {
    return "<b>" + demon.name + "</b>";
  } else if (demon.position <= window.extended_list_length) {
    return demon.name;
  } else {
    return "<i>" + demon.name + "</i>";
  }
}

function formatDemons(demons) {
  return demons.map(formatDemon).join(", ");
}

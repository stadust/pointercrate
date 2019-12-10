function generateRecord(record) {
  var li = document.createElement("li");
  var recordId = document.createElement("b");
  var submitter = document.createElement("i");

  li.className = "white hover";
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
    default:
      break;
  }

  recordId.appendChild(document.createTextNode("Record #" + record.id));

  submitter.appendChild(
    document.createTextNode("Submitter ID: " + record.submitter)
  );

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
  li.appendChild(submitter);

  return li;
}

function embedVideo(video) {
  if (video === undefined) return;
  // welcome to incredibly fragile string parsing with stadust
  // see pointercrate::video::embed for a proper implementation of this

  if (video.startsWith("https://www.youtube")) {
    return "https://www.youtube.com/embed/" + video.substring(32);
  }

  if (video.statsWith("https://www.twitch")) {
    return (
      "https://player.twitch.tv/?autoplay=false&video=" + video.substring(29)
    );
  }
}

class RecordManager extends Paginator {
  constructor() {
    super("record-pagination", {}, generateRecord);

    var manager = document.getElementById("record-manager");

    this._welcome = manager.getElementsByClassName("viewer-welcome")[0];
    this._content = manager.getElementsByClassName("viewer-content")[0];

    this._video = document.getElementById("record-video");
    this._video_link = document.getElementById("record-video-link");
    this._id = document.getElementById("record-id");
    this._demon = document.getElementById("record-demon");
    this._holder = document.getElementById("record-holder");
    this._status = document.getElementById("record-status");
    this._progress = document.getElementById("record-progress");
    this._submitter = document.getElementById("record-submitter");
    this._notes = document.getElementById("record-notes");

    this.dropdown = new Dropdown(
      document
        .getElementById("status-filter-panel")
        .getElementsByClassName("dropdown-menu")[0]
    );
    this.dropdown.addEventListener(li => {
      if (li.innerHTML === "All") this.updateQueryData("status", undefined);
      else this.updateQueryData("status", li.innerHTML);
    });
  }

  onReceive(response) {
    var recordData = response.responseJSON.data;

    var embeddedVideo = embedVideo(recordData.video);

    if (embeddedVideo !== undefined) {
      this._video.style.display = "block";
      this._video.src = embedVideo(recordData.video);
    } else {
      this._video.style.display = "none";
    }

    this._video_link.href = recordData.video;
    this._video_link.innerHTML = recordData.video;
    this._id.innerHTML = recordData.id;
    this._demon.innerHTML = recordData.demon.name;
    this._holder.innerHTML = recordData.player.name;
    this._status.innerHTML = recordData.status;
    this._progress.innerHTML = recordData.progress;
    this._submitter.innerHTML = recordData.submitter.id;
    this._notes.innerHTML = recordData.notes;

    $(this._welcome).hide(100);
    $(this._content).show(100);
  }
}

$(document).ready(function() {
  TABBED_PANES["account-tabber"].addSwitchListener("3", () => {
    if (window.recordManager === undefined) {
      window.recordManager = new RecordManager();
      window.recordManager.initialize();

      setupRecordFilterPlayerIdForm();
      setupRecordFilterPlayerNameForm();
    }
  });
});

function setupRecordFilterPlayerIdForm() {
  var recordFilterPlayerIdForm = new Form(
    document.getElementById("record-filter-by-player-id-form")
  );
  var playerId = recordFilterPlayerIdForm.input("record-player-id");

  playerId.addValidator(valueMissing, "Player ID required");
  recordFilterPlayerIdForm.onSubmit(function(event) {
    window.recordManager.updateQueryData("player", playerId.value);
  });
}

function setupRecordFilterPlayerNameForm() {
  var recordFilterPlayerNameForm = new Form(
    document.getElementById("record-filter-by-player-name-form")
  );
  var playerName = recordFilterPlayerNameForm.input("record-player-name");

  playerName.addValidators({
    "Player name required": valueMissing
  });

  recordFilterPlayerNameForm.onSubmit(function(event) {
    makeRequest(
      "GET",
      "/players/?name=" + playerName.value,
      recordFilterPlayerNameForm.errorOutput,
      data => {
        let json = data.responseJSON;

        if (!json || json.length == 0) {
          playerName.setError("No player with that name found!");
        } else {
          window.recordManager.updateQueryData("player", json[0].id);
        }
      }
    );
  });
}

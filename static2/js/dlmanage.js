function generateRecord(record) {
  var li = document.createElement("li");
  var recordId = document.createElement("b");

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
  constructor(tok) {
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
    this._tok = tok; // FIXME: bad

    this.dropdown = new Dropdown(
      document
        .getElementById("status-filter-panel")
        .getElementsByClassName("dropdown-menu")[0]
    );

    new Dropdown(
      manager.getElementsByClassName("dropdown-menu")[0]
    ).addEventListener(li => {
      if (li.innerHTML === "All Demons")
        this.updateQueryData("demon_id", undefined);
      else this.updateQueryData("demon_id", li.dataset.value);
    });
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
    this._demon.innerHTML =
      recordData.demon.name + " (" + recordData.demon.id + ")";
    this._holder.innerHTML =
      recordData.player.name + " (" + recordData.player.id + ")";
    this._status.innerHTML = recordData.status;
    this._progress.innerHTML = recordData.progress;
    this._submitter.innerHTML = recordData.submitter.id;

    if (!recordData.notes.length)
      // TODO: maybe via CSS transform?
      $(this._notes).hide(100);

    // clear notes
    while (this._notes.firstChild) {
      this._notes.removeChild(this._notes.firstChild);
    }

    for (let note of recordData.notes) {
      this._notes.appendChild(createNoteHtml(recordData.id, note, this._tok));
    }

    if (recordData.notes.length) {
      $(this._notes).show(100); // TODO: maybe via CSS transform?
    }

    $(this._welcome).hide(100);
    $(this._content).show(100);
  }
}

function createNoteHtml(recordId, note, csrfToken) {
  let noteDiv = document.createElement("div");

  noteDiv.classList.add("white");
  noteDiv.classList.add("hover");

  // only add option to delete notes if you're list admin (and yes, server sided validation is also in place. I am just too lazy to write permission error handling)
  let isAdmin =
    (window.permissions & 0x8) == 0x8 || window.username == note.author;

  if (isAdmin) {
    var closeX = document.createElement("span");
    closeX.classList.add("hover");
    closeX.classList.add("plus");
    closeX.classList.add("cross");

    closeX.style.transform = "scale(0.75)";

    closeX.addEventListener("click", () => {
      confirm("This action will irrevocably delete this note. Proceed?");

      makeRequest(
        "DELETE",
        "/api/v1/records/" + recordId + "/notes/" + note.id + "/",
        null,
        () => {
          // node suicide
          noteDiv.parentElement.removeChild(noteDiv);
        },
        {},
        { "X-CSRF-TOKEN": csrfToken }
      );
    });
  }

  let b = document.createElement("b");
  b.innerHTML = "Record Note #" + note.id;

  let i = document.createElement("i");
  i.innerHTML = note.content;

  let furtherInfo = document.createElement("i");
  furtherInfo.style.fontSize = "80%";
  furtherInfo.style.textAlign = "right";

  if (note.author === null) {
    furtherInfo.innerHTML =
      "This note was left as a comment by the submitter. ";
  } else {
    furtherInfo.innerHTML = "This note was left by " + note.author + ". ";
  }

  if (note.editors.length) {
    furtherInfo.innerHTML +=
      "This note was subsequently modified by: " +
      note.editors.join(", ") +
      ". ";
  }

  if (note.transferred) {
    furtherInfo.innerHTML += "This not was not originally left on this record.";
  }

  if (isAdmin) noteDiv.appendChild(closeX);
  noteDiv.appendChild(b);
  noteDiv.appendChild(i);
  noteDiv.appendChild(furtherInfo);

  return noteDiv;
}

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
      "/api/v1/players/?name=" + playerName.value,
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

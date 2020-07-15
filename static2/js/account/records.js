"use strict";

import {
  Paginator,
  post,
  patch,
  del,
  get,
  displayError,
  valueMissing,
  Form,
  Dropdown,
  badInput,
  rangeUnderflow,
  rangeOverflow,
  stepMismatch,
  typeMismatch,
} from "../modules/form.mjs";

let recordManager;
let recordEditor;

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

  if (video.startsWith("https://www.twitch")) {
    return (
      "https://player.twitch.tv/?autoplay=false&parent=pointercrate.com&video=" +
      video.substring(29)
    );
  }
}

class RecordManager extends Paginator {
  constructor(tok) {
    super("record-pagination", {}, generateRecord);

    var manager = document.getElementById("record-manager");

    this.currentRecord = null;
    this.currentRecordEtag = null;

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
    ).addEventListener((selected) => {
      if (selected === "All") this.updateQueryData("demon_id", undefined);
      else this.updateQueryData("demon_id", selected);
    });
    this.dropdown.addEventListener((selected) => {
      if (selected === "All") this.updateQueryData("status", undefined);
      else this.updateQueryData("status", selected);
    });
  }

  onReceive(response) {
    if (response.status == 204) {
      return;
    }

    var recordData = (this.currentRecord = response.data.data);
    this.currentRecordEtag = response.headers["etag"];

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

    // clear notes
    while (this._notes.firstChild) {
      this._notes.removeChild(this._notes.firstChild);
    }

    for (let note of recordData.notes) {
      this._notes.appendChild(createNoteHtml(note, this._tok));
    }

    recordEditor.selectRecord(recordData);

    $(this._notes.parentElement).show(100); // TODO: maybe via CSS transform?

    $(this._welcome).hide(100);
    $(this._content).show(100);
  }
}

function createNoteHtml(note, csrfToken) {
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
      if (confirm("This action will irrevocably delete this note. Proceed?")) {
        del(
          "/api/v1/records/" +
            recordManager.currentRecord.id +
            "/notes/" +
            note.id +
            "/",
          { "X-CSRF-TOKEN": csrfToken }
        ).then(() => noteDiv.parentElement.removeChild(noteDiv));
      }
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

function setupAddNote(csrfToken) {
  let adder = document.getElementById("add-record-note");
  let output = adder.getElementsByClassName("output")[0];
  let textArea = adder.getElementsByTagName("textarea")[0];
  let add = adder.getElementsByClassName("button")[0];

  add.addEventListener("click", () => {
    post(
      "/api/v1/records/" + recordManager.currentRecord.id + "/notes/",
      { "X-CSRF-TOKEN": csrfToken },
      { content: textArea.value }
    )
      .then((noteResponse) => {
        let newNote = createNoteHtml(noteResponse.data.data, csrfToken);
        recordManager._notes.appendChild(newNote);

        $(adder).hide(100);
        textArea.value = "";
      })
      .catch(displayError(output));
  });

  document
    .getElementById("add-record-note-open")
    .addEventListener("click", () => {
      $(adder).show(100);
    });
}

function setupRecordFilterPlayerIdForm() {
  var recordFilterPlayerIdForm = new Form(
    document.getElementById("record-filter-by-player-id-form")
  );
  var playerId = recordFilterPlayerIdForm.input("record-player-id");

  playerId.addValidator(valueMissing, "Player ID required");
  recordFilterPlayerIdForm.onSubmit(function (event) {
    window.recordManager.updateQueryData("player", playerId.value);
  });
}

function setupRecordFilterPlayerNameForm() {
  var recordFilterPlayerNameForm = new Form(
    document.getElementById("record-filter-by-player-name-form")
  );
  var playerName = recordFilterPlayerNameForm.input("record-player-name");

  playerName.addValidators({
    "Player name required": valueMissing,
  });

  recordFilterPlayerNameForm.onSubmit(function (event) {
    get("/api/v1/players/?name=" + playerName.value)
      .then((response) => {
        let json = response.data;

        if (!json || json.length == 0) {
          playerName.setError("No player with that name found!");
        } else {
          recordManager.updateQueryData("player", json[0].id);
        }
      })
      .catch(displayError(recordFilterPlayerNameForm.errorOutput));
  });
}

class RecordEditor extends Form {
  constructor(csrfToken) {
    super(document.getElementById("edit-record-form"));

    this.recordId = document.getElementById("edit-record-id");

    this.statusDropdown = new Dropdown(
      document.getElementById("edit-record-status")
    );

    var progress = this.input("edit-record-progress");
    var video = this.input("edit-record-video");

    progress.addValidator(rangeUnderflow, "Record progress cannot be negative");
    progress.addValidator(
      rangeOverflow,
      "Record progress cannot be larger than 100%"
    );
    progress.addValidator(badInput, "Record progress must be a valid integer");
    progress.addValidator(stepMismatch, "Record progress mustn't be a decimal");

    video.addValidator(typeMismatch, "Please enter a valid URL");

    this.setClearOnSubmit(true);
    this.onSubmit(function (event) {
      let data = this.serialize();

      if (this.statusDropdown.selected != recordManager.currentRecord.status) {
        data["status"] = this.statusDropdown.selected;
      }

      patch(
        "/api/v1/records/" + recordManager.currentRecord.id + "/",
        {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": recordManager.currentRecordEtag,
        },
        data
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            // directly refresh the record manager :pog:
            recordManager.refresh();
            recordManager.onReceive(response);

            this.setSuccess(
              "Record successfully edited! You may now close this panel"
            );
          }
        })
        .catch(displayError(this.errorOutput));
    });
  }

  selectRecord(record) {
    this.recordId.innerText = record.id;
    this.statusDropdown.select(record.status);
  }
}

function setupEditRecordForm(csrfToken) {
  recordEditor = new RecordEditor(csrfToken);

  document.getElementById("record-delete").addEventListener("click", () => {
    if (
      confirm(
        "Are you sure? This will irrevocably delete this record and all notes made on it!"
      )
    ) {
      del("/api/v1/records/" + recordManager.currentRecord.id + "/", {
        "X-CSRF-TOKEN": csrfToken,
        "If-Match": window.recordManager.currentRecordEtag,
      }).then(() => {
        $(recordManager._content).hide(100);
        $(recordManager._notes.parentElement).hide(100);
        $(recordManager._welcome).show(100);
        recordManager.refresh();
      });
    }
  });
}

export function initialize(csrfToken) {
  setupRecordFilterPlayerIdForm();
  setupRecordFilterPlayerNameForm();
  setupAddNote(csrfToken);
  setupEditRecordForm(csrfToken);

  recordManager = new RecordManager(csrfToken);
  recordManager.initialize();
}

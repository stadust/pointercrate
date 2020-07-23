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
import {
  initializeRecordSubmitter,
  generatePlayer,
} from "../modules/demonlist.mjs";
import { FilteredPaginator } from "../modules/form.mjs";

export let recordManager;

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
    this._progress = document.getElementById("record-progress");
    this._submitter = document.getElementById("record-submitter");
    this._notes = document.getElementById("record-notes");
    this._tok = tok; // FIXME: bad

    // Gotta start counting at '1', since '0' is the error output of the paginator
    this.errorOutput = manager.getElementsByClassName("output")[1];
    this.successOutput = manager.getElementsByClassName("output")[2];

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

    this._status = new Dropdown(document.getElementById("edit-record-status"));
    this._status.addEventListener((selected) => {
      if (selected != this.currentRecord.status) {
        patch(
          "/api/v1/records/" + this.currentRecord.id + "/",
          {
            "X-CSRF-TOKEN": this._tok,
            "If-Match": this.currentRecordEtag,
          },
          { status: selected }
        )
          .then((response) => {
            if (response.status == 304) {
              this.setSuccess("Nothing changed!");
            } else {
              // directly refresh the record manager :pog:
              this.refresh();
              this.onReceive(response);

              this.setSuccess("Record status successfully edited!");
            }
          })
          .catch(displayError(this.errorOutput));
      }
    });

    this.initProgressDialog();
    this.initVideoDialog();
    new HolderDialog();
    this.initDemonDialog();
  }

  initProgressDialog() {
    var editProgressDialog = document.getElementById("record-progress-dialog");
    var editProgressForm = new Form(
      editProgressDialog.getElementsByTagName("form")[0]
    );
    document
      .getElementById("record-progress-pen")
      .addEventListener("click", () => {
        $(editProgressDialog.parentElement).show();
      });

    let progress = editProgressForm.input("record-progress-edit");

    progress.addValidator(rangeUnderflow, "Record progress cannot be negative");
    progress.addValidator(
      rangeOverflow,
      "Record progress cannot be larger than 100%"
    );
    progress.addValidator(badInput, "Record progress must be a valid integer");
    progress.addValidator(stepMismatch, "Record progress mustn't be a decimal");

    editProgressForm.onSubmit(() => {
      patch(
        "/api/v1/records/" + this.currentRecord.id + "/",
        {
          "X-CSRF-TOKEN": this._tok,
          "If-Match": this.currentRecordEtag,
        },
        editProgressForm.serialize()
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            // directly refresh the record manager :pog:
            this.refresh();
            this.onReceive(response);
            this.setSuccess("Record progress successfully edited!");
          }
          $(editProgressDialog.parentElement).hide();
        })
        .catch(
          displayError(editProgressForm.errorOutput, {
            42215: (response) => progress.setError(response.message),
          })
        );
    });
  }

  initVideoDialog() {
    var editVideoDialog = document.getElementById("record-video-dialog");
    var editVideoForm = new Form(
      editVideoDialog.getElementsByTagName("form")[0]
    );
    document
      .getElementById("record-video-pen")
      .addEventListener("click", () => {
        $(editVideoDialog.parentElement).show();
      });

    let video = editVideoForm.input("record-video-edit");

    video.addValidator(typeMismatch, "Please enter a valid URL");

    editVideoForm.onSubmit(() => {
      patch(
        "/api/v1/records/" + this.currentRecord.id + "/",
        {
          "X-CSRF-TOKEN": this._tok,
          "If-Match": this.currentRecordEtag,
        },
        editVideoForm.serialize()
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            this.onReceive(response);
            this.setSuccess("Record video successfully edited!");
          }
          $(editVideoDialog.parentElement).hide();
        })
        .catch(displayError(video.errorOutput));
    });
  }

  initDemonDialog() {
    var editDemonDialog = document.getElementById("record-demon-dialog");

    document
      .getElementById("record-demon-pen")
      .addEventListener("click", () => {
        $(editDemonDialog.parentElement).show();
      });

    new Dropdown(document.getElementById("edit-demon-record")).addEventListener(
      (demonId) => {
        patch(
          "/api/v1/records/" + this.currentRecord.id + "/",
          {
            "X-CSRF-TOKEN": this._tok,
            "If-Match": this.currentRecordEtag,
          },
          { demon_id: parseInt(demonId) }
        )
          .then((response) => {
            if (response.status == 304) {
              this.setSuccess("Nothing changed!");
            } else {
              this.refresh();
              this.onReceive(response);
              this.setSuccess("Record demon successfully edited!");
            }
            $(editDemonDialog.parentElement).hide();
          })
          .catch((response) => {
            displayError(this.errorOutput)(response);
            $(editDemonDialog.parentElement).hide();
          });
      }
    );
  }

  setError(message) {
    if (this.successOutput) this.successOutput.style.display = "none";

    if (this.errorOutput) {
      if (message === null || message === undefined) {
        this.errorOutput.style.display = "none";
      } else {
        this.errorOutput.innerHTML = message;
        this.errorOutput.style.display = "block";
      }
    }
  }

  setSuccess(message) {
    if (this.errorOutput) this.errorOutput.style.display = "none";

    if (this.successOutput) {
      if (message === null || message === undefined) {
        this.successOutput.style.display = "none";
      } else {
        this.successOutput.innerHTML = message;
        this.successOutput.style.display = "block";
      }
    }
  }

  onReceive(response) {
    this.setError(null);
    this.setSuccess(null);

    if (response.status == 204) {
      return;
    }

    var recordData = (this.currentRecord = response.data.data);
    this.currentRecordEtag = response.headers["etag"];

    var embeddedVideo = embedVideo(recordData.video);

    if (embeddedVideo !== undefined) {
      this._video.style.display = "block";
      this._video_link.style.display = "initial";
      this._video.src = embedVideo(recordData.video);
      this._video_link.href = recordData.video;
      this._video_link.innerHTML = recordData.video;
    } else {
      this._video.style.display = "none";
      this._video_link.style.display = "none";
    }

    this._id.innerHTML = recordData.id;
    this._demon.innerHTML =
      recordData.demon.name + " (" + recordData.demon.id + ")";
    this._holder.innerHTML =
      recordData.player.name + " (" + recordData.player.id + ")";
    this._status.select(recordData.status);
    this._progress.innerHTML = recordData.progress + "%";
    this._submitter.innerHTML = recordData.submitter.id;

    // clear notes
    while (this._notes.firstChild) {
      this._notes.removeChild(this._notes.firstChild);
    }

    for (let note of recordData.notes) {
      this._notes.appendChild(createNoteHtml(note, this._tok));
    }

    $(this._notes.parentElement).show(100); // TODO: maybe via CSS transform?

    $(this._welcome).hide(100);
    $(this._content).show(100);
  }
}

class HolderDialog extends FilteredPaginator {
  constructor() {
    super("record-holder-dialog-pagination", generatePlayer, "name_contains");

    this.editHolderDialog = document.getElementById("record-holder-dialog");
    this.editHolderForm = new Form(
      this.editHolderDialog.getElementsByTagName("form")[0]
    );
    document
      .getElementById("record-holder-pen")
      .addEventListener("click", () => {
        this.initialize();
        $(this.editHolderDialog.parentElement).show();
      });

    this.editHolderForm.onSubmit(() =>
      this.changeHolder(
        this.editHolderForm.input("record-holder-name-edit").value
      )
    );
  }

  changeHolder(newHolder) {
    patch(
      "/api/v1/records/" + recordManager.currentRecord.id + "/",
      {
        "X-CSRF-TOKEN": recordManager._tok,
        "If-Match": recordManager.currentRecordEtag,
      },
      { player: newHolder }
    )
      .then((response) => {
        if (response.status == 304) {
          recordManager.setSuccess("Nothing changed!");
        } else {
          recordManager.onReceive(response);
          recordManager.refresh();
          recordManager.setSuccess("Record holder successfully edited!");
        }
        $(this.editHolderDialog.parentElement).hide();
      })
      .catch(displayError(this.editHolderForm.errorOutput));
  }

  onSelect(selected) {
    this.changeHolder(selected.dataset.name);
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

function setupRecordSearchRecordIdForm() {
  var recordFilterPlayerIdForm = new Form(
    document.getElementById("record-filter-by-player-id-form")
  );
  var playerId = recordFilterPlayerIdForm.input("record-player-id");

  playerId.addValidator(valueMissing, "Player ID required");
  recordFilterPlayerIdForm.onSubmit(function (event) {
    window.recordManager.updateQueryData("player", playerId.value);
  });
}

function setupRecordFilterPlayerIdForm() {
  var recordFilterPlayerIdForm = new Form(
    document.getElementById("record-search-by-record-id-form")
  );
  var recordId = recordFilterPlayerIdForm.input("record-record-id");

  recordId.addValidator(valueMissing, "Record ID required");
  recordFilterPlayerIdForm.onSubmit(function (event) {
    recordManager
      .selectArbitrary(parseInt(recordId.value))
      .catch(displayError(recordFilterPlayerIdForm.errorOutput));
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

function setupEditRecordForm(csrfToken) {
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

  initializeRecordSubmitter();

  recordManager = new RecordManager(csrfToken);
  recordManager.initialize();
}

"use strict";

import {
  post,
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
  Paginator,
  typeMismatch,
  Viewer,
  setupFormDialogEditor,
  Output,
  setupDropdownEditor,
  setupDialogEditor,
  PaginatorEditorBackend,
} from "../modules/form.mjs";
import {
  initializeRecordSubmitter,
  generateRecord,
  embedVideo,
} from "../modules/demonlist.mjs";
import { setupPlayerSelectionEditor } from "../modules/demonlist.mjs";

export let recordManager;

class RecordManager extends Paginator {
  constructor(tok) {
    super("record-pagination", {}, generateRecord);

    var manager = document.getElementById("record-manager");

    this.output = new Viewer(
      manager.getElementsByClassName("viewer-content")[0],
      this
    );

    this._video = document.getElementById("record-video");
    this._video_link = document.getElementById("record-video-link");
    this._id = document.getElementById("record-id");
    this._demon = document.getElementById("record-demon");
    this._holder = document.getElementById("record-holder");
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

    this._status = setupDropdownEditor(
      new PaginatorEditorBackend(this, this._tok, true),
      "edit-record-status",
      "status",
      this.output
    );

    this.initProgressDialog();
    this.initVideoDialog();
    setupPlayerSelectionEditor(
      new PaginatorEditorBackend(this, this._tok, true),
      "record-holder-dialog-pagination",
      "record-holder-pen",
      this.output
    );
    this.initDemonDialog();
  }

  initProgressDialog() {
    let form = setupFormDialogEditor(
      new PaginatorEditorBackend(this, this._tok, true),
      "record-progress-dialog",
      "record-progress-pen",
      this.output
    );

    form.addValidators({
      "record-progress-edit": {
        "Record progress cannot be negative": rangeUnderflow,
        "Record progress cannot be larger than 100%": rangeOverflow,
        "Record progress must be a valid integer": badInput,
        "Record progress mustn't be a decimal": stepMismatch,
        "Please enter a progress value": valueMissing,
      },
    });

    form.addErrorOverride(42215, "record-progress-edit");
  }

  initVideoDialog() {
    let form = setupFormDialogEditor(
      new PaginatorEditorBackend(this, this._tok, false),
      "record-video-dialog",
      "record-video-pen",
      this.output
    );

    form.addValidators({
      "record-video-edit": {
        "Please enter a valid URL": typeMismatch,
      },
    });

    for (let errorCode of [42222, 42223, 42224, 42225]) {
      form.addErrorOverride(errorCode, "record-video-edit");
    }
  }

  initDemonDialog() {
    var editor = setupDialogEditor(
      new PaginatorEditorBackend(this, this._tok, true),
      "record-demon-dialog",
      "record-demon-pen",
      this.output
    );

    new Dropdown(
      document.getElementById("edit-demon-record")
    ).addEventListener((demonId) => editor({ demon_id: parseInt(demonId) }));
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    var embeddedVideo = embedVideo(this.currentObject.video);

    if (embeddedVideo !== undefined) {
      this._video.style.display = "block";
      this._video_link.style.display = "initial";
      this._video.src = embeddedVideo;
      this._video_link.href = this.currentObject.video;
      this._video_link.innerHTML = this.currentObject.video;
    } else {
      this._video.style.display = "none";
      this._video_link.style.display = "none";
    }

    this._id.innerHTML = this.currentObject.id;
    this._demon.innerHTML =
      this.currentObject.demon.name + " (" + this.currentObject.demon.id + ")";
    this._holder.innerHTML =
      this.currentObject.player.name +
      " (" +
      this.currentObject.player.id +
      ")";
    this._status.selectSilently(this.currentObject.status);
    this._progress.innerHTML = this.currentObject.progress + "%";
    this._submitter.innerHTML = this.currentObject.submitter.id;

    // clear notes
    while (this._notes.firstChild) {
      this._notes.removeChild(this._notes.firstChild);
    }

    for (let note of this.currentObject.notes) {
      this._notes.appendChild(createNoteHtml(note, this._tok));
    }

    $(this._notes.parentElement).show(300); // TODO: maybe via CSS transform?
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
            recordManager.currentObject.id +
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
  let output = new Output(adder);
  let textArea = adder.getElementsByTagName("textarea")[0];
  let add = adder.getElementsByClassName("button")[0];

  add.addEventListener("click", () => {
    post(
      "/api/v1/records/" + recordManager.currentObject.id + "/notes/",
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
      $(adder).show(300);
    });
}

function setupRecordFilterPlayerIdForm() {
  var recordFilterPlayerIdForm = new Form(
    document.getElementById("record-filter-by-player-id-form")
  );
  var playerId = recordFilterPlayerIdForm.input("record-player-id");

  playerId.addValidator(valueMissing, "Player ID required");
  recordFilterPlayerIdForm.onSubmit(function (event) {
    recordManager.updateQueryData("player", playerId.value);
  });
}

function setupRecordSearchRecordIdForm() {
  var recordSearchByIdForm = new Form(
    document.getElementById("record-search-by-record-id-form")
  );
  var recordId = recordSearchByIdForm.input("record-record-id");

  recordId.addValidator(valueMissing, "Record ID required");
  recordSearchByIdForm.onSubmit(function (event) {
    recordManager
      .selectArbitrary(parseInt(recordId.value))
      .catch(displayError(recordSearchByIdForm));
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
      .catch(displayError(recordFilterPlayerNameForm));
  });
}

function setupEditRecordForm(csrfToken) {
  document.getElementById("record-delete").addEventListener("click", () => {
    if (
      confirm(
        "Are you sure? This will irrevocably delete this record and all notes made on it!"
      )
    ) {
      del("/api/v1/records/" + recordManager.currentObject.id + "/", {
        "X-CSRF-TOKEN": csrfToken,
        "If-Match": window.recordManager.currentEtag,
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
  setupRecordSearchRecordIdForm();

  initializeRecordSubmitter(csrfToken, true);

  recordManager = new RecordManager(csrfToken);
  return recordManager.initialize();
}

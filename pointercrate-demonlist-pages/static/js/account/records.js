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
  PaginatorEditorBackend,
  setupEditorDialog,
  DropdownDialog,
  FormDialog,
} from "/static/core/js/modules/form.js";
import {
  initializeRecordSubmitter,
  generateRecord,
  embedVideo,
} from "/static/demonlist/js/modules/demonlist.js";
import { tr, trp } from "/static/core/js/modules/localization.js";

export let recordManager;

class RecordManager extends Paginator {
  constructor() {
    super("record-pagination", {}, generateRecord);

    var manager = document.getElementById("record-manager");

    this.output = new Viewer(
      manager.getElementsByClassName("viewer-content")[0],
      this
    );

    this._video = document.getElementById("record-video");
    this._video_link = document.getElementById("record-video-link");
    this._raw_footage_link = document.getElementById("record-raw-footage-link");
    this._id = document.getElementById("record-id");
    this._demon = document.getElementById("record-demon");
    this._holder = document.getElementById("record-holder");
    this._progress = document.getElementById("record-progress");
    this._submitter = document.getElementById("record-submitter");
    this._notes = document.getElementById("record-notes");

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
      new PaginatorEditorBackend(this, true),
      "edit-record-status",
      "status",
      this.output
    );

    this.initProgressDialog();
    this.initVideoDialog();
    setupEditorDialog(
      new FormDialog("record-holder-dialog"),
      "record-holder-pen",
      new PaginatorEditorBackend(this, true),
      this.output
    );
    this.initDemonDialog();

    document
      .getElementById("record-copy-info")
      .addEventListener("click", () => {
        navigator.clipboard
          .writeText(
            this.currentObject.id +
              ", " +
              this._holder.innerText +
              ", " +
              this.currentObject.video
          )
          .then(() =>
            this.output.setSuccess(
              tr("demonlist", "record", "record-viewer.copy-data-success")
            )
          )
          .catch(() =>
            this.output.setError(
              tr("demonlist", "record", "record-viewer.copy-data-error")
            )
          );
      });
  }

  initProgressDialog() {
    let form = setupFormDialogEditor(
      new PaginatorEditorBackend(this, true),
      "record-progress-dialog",
      "record-progress-pen",
      this.output
    );

    form.addValidators({
      "record-progress-edit": {
        [tr(
          "demonlist",
          "record",
          "record-progress-dialog.progress-validator-rangeunderflow"
        )]: rangeUnderflow,
        [tr(
          "demonlist",
          "record",
          "record-progress-dialog.progress-validator-rangeoverflow"
        )]: rangeOverflow,
        [tr(
          "demonlist",
          "record",
          "record-progress-dialog.progress-validator-badinput"
        )]: badInput,
        [tr(
          "demonlist",
          "record",
          "record-progress-dialog.progress-validator-stepmismatch"
        )]: stepMismatch,
        [tr(
          "demonlist",
          "record",
          "record-progress-dialog.progress-validator-valuemissing"
        )]: valueMissing,
      },
    });

    form.addErrorOverride(42215, "record-progress-edit");
  }

  initVideoDialog() {
    let form = setupFormDialogEditor(
      new PaginatorEditorBackend(this, false),
      "record-video-dialog",
      "record-video-pen",
      this.output
    );

    form.addValidators({
      "record-video-edit": {
        [tr(
          "demonlist",
          "record",
          "record-videolink-dialog.videolink-validator-typemismatch"
        )]: typeMismatch,
      },
    });

    for (let errorCode of [42222, 42223, 42224, 42225]) {
      form.addErrorOverride(errorCode, "record-video-edit");
    }
  }

  initDemonDialog() {
    setupEditorDialog(
      new DropdownDialog("record-demon-dialog", "edit-demon-record"),
      "record-demon-pen",
      new PaginatorEditorBackend(this, true),
      this.output,
      (demonId) => ({ demon_id: parseInt(demonId) })
    );
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    var embeddedVideo = embedVideo(this.currentObject.video);

    if (embeddedVideo !== undefined) {
      this._video.style.display = "block";
      this._video.src = embeddedVideo;
    } else {
      this._video.style.display = "none";
    }

    if (this.currentObject.video !== undefined) {
      this._video_link.href = this.currentObject.video;
      this._video_link.innerText = this.currentObject.video;
      this._video_link.style.display = "initial";
    } else {
      this._video_link.style.display = "none";
    }

    if (this.currentObject.raw_footage !== undefined) {
      this._raw_footage_link.href = this.currentObject.raw_footage;
      this._raw_footage_link.innerText = this.currentObject.raw_footage;
      this._raw_footage_link.style.display = "initial";
    } else {
      this._raw_footage_link.style.display = "none";
    }

    this._id.innerText = this.currentObject.id;
    this._demon.innerText =
      this.currentObject.demon.name + " (" + this.currentObject.demon.id + ")";
    this._holder.innerText =
      this.currentObject.player.name +
      " (" +
      this.currentObject.player.id +
      ")";
    this._status.selectSilently(this.currentObject.status);
    this._progress.innerText = this.currentObject.progress + "%";
    this._submitter.innerText = this.currentObject.submitter.id;

    // this is introducing race conditions. Oh well.
    return get("/api/v1/records/" + this.currentObject.id + "/notes").then(
      (response) => {
        // clear notes
        while (this._notes.firstChild) {
          this._notes.removeChild(this._notes.firstChild);
        }

        for (let note of response.data) {
          this._notes.appendChild(createNoteHtml(note));
        }

        $(this._notes.parentElement).show(300); // TODO: maybe via CSS transform?
      }
    );
  }
}

function createNoteHtml(note) {
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
      if (
        confirm(tr("demonlist", "record", "record-note-listed.confirm-delete"))
      ) {
        del(
          "/api/v1/records/" +
            recordManager.currentObject.id +
            "/notes/" +
            note.id +
            "/"
        ).then(() => noteDiv.parentElement.removeChild(noteDiv));
      }
    });
  }

  let b = document.createElement("b");
  b.innerText = trp("demonlist", "record", "record-note-listed", {
    ["note-id"]: note.id,
  });

  let i = document.createElement("i");
  i.innerText = note.content;

  let furtherInfo = document.createElement("i");
  furtherInfo.style.fontSize = "80%";
  furtherInfo.style.textAlign = "right";

  if (note.author === null) {
    furtherInfo.innerText = tr(
      "demonlist",
      "record",
      "record-note-listed.author-submitter"
    );
  } else {
    furtherInfo.innerText = trp(
      "demonlist",
      "record",
      "record-note-listed.author",
      {
        ["author"]: note.author,
      }
    );
  }
  furtherInfo.innerText += " ";

  if (note.editors.length) {
    furtherInfo.innerText +=
      trp("demonlist", "record", "record-note-listed.editors", {
        ["editors"]: note.editors.join(", "),
      }) + " ";
  }

  if (note.transferred) {
    furtherInfo.innerText +=
      tr("demonlist", "record", "record-note-listed.transferred") + " ";
  }

  if (note.is_public) {
    furtherInfo.innerText +=
      tr("demonlist", "record", "record-note-listed.public") + " ";
  }

  if (isAdmin) noteDiv.appendChild(closeX);
  noteDiv.appendChild(b);
  noteDiv.appendChild(i);
  noteDiv.appendChild(furtherInfo);

  return noteDiv;
}

function setupAddNote() {
  let adder = document.getElementById("add-record-note");
  let output = new Output(adder);
  let textArea = adder.getElementsByTagName("textarea")[0];
  let add = adder.getElementsByClassName("button")[0];
  let isPublic = document.getElementById("add-note-is-public-checkbox");

  add.addEventListener("click", () => {
    post(
      "/api/v1/records/" + recordManager.currentObject.id + "/notes/",
      {},
      { content: textArea.value, is_public: isPublic.checked }
    )
      .then((noteResponse) => {
        let newNote = createNoteHtml(noteResponse.data.data);
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

  recordFilterPlayerIdForm.onSubmit(function () {
    // Reset search filter if player ID field is empty
    recordManager.updateQueryData(
      "player",
      valueMissing(playerId) ? playerId.value : undefined
    );
  });
}

function setupRecordSearchRecordIdForm() {
  var recordSearchByIdForm = new Form(
    document.getElementById("record-search-by-record-id-form")
  );
  var recordId = recordSearchByIdForm.input("record-record-id");

  recordId.addValidator(
    valueMissing,
    tr("demonlist", "record", "record-idsearch-panel.id-validator-valuemissing")
  );
  recordSearchByIdForm.onSubmit(function () {
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

  recordFilterPlayerNameForm.onSubmit(function () {
    if (!valueMissing(playerName)) {
      // Player name field is empty, so reset search filter
      recordManager.updateQueryData("player", undefined);
    } else {
      get("/api/v1/players/?name=" + playerName.value)
        .then((response) => {
          let json = response.data;

          if (!json || json.length == 0) {
            playerName.errorText = trp(
              "core",
              "error",
              "error-demonlist-playernotfoundname",
              {
                ["player-name"]: playerName.value,
              }
            );
          } else {
            recordManager.updateQueryData("player", json[0].id);
          }
        })
        .catch(displayError(recordFilterPlayerNameForm));
    }
  });
}

function setupEditRecordForm() {
  document.getElementById("record-delete").addEventListener("click", () => {
    if (confirm(tr("demonlist", "record", "record-viewer.confirm-delete"))) {
      del("/api/v1/records/" + recordManager.currentObject.id + "/", {
        "If-Match": recordManager.currentEtag,
      }).then(() => {
        recordManager.output.hideContent();
        recordManager.refresh();
      });
    }
  });
}

export function initialize() {
  setupRecordFilterPlayerIdForm();
  setupRecordFilterPlayerNameForm();
  setupAddNote();
  setupEditRecordForm();
  setupRecordSearchRecordIdForm();

  initializeRecordSubmitter(true);

  recordManager = new RecordManager();
  return recordManager.initialize();
}

import {
  generateDemon,
  embedVideo,
  setupPlayerSelectionEditor,
} from "../modules/demonlist.mjs";
import {
  FilteredPaginator,
  Viewer,
  setupFormDialogEditor,
  PaginatorEditorBackend,
  rangeOverflow,
  rangeUnderflow,
  badInput,
  stepMismatch,
  valueMissing,
  typeMismatch,
} from "../modules/form.mjs";

export let demonManager;

export class DemonManager extends FilteredPaginator {
  constructor(csrfToken) {
    super("demon-pagination", generateDemon, "name_contains");

    this.output = new Viewer(
      this.html.parentNode.getElementsByClassName("viewer-content")[0],
      this
    );

    this.retrievalEndpoint = "/api/v2/demons/";

    this._id = document.getElementById("demon-demon-id");
    this._name = document.getElementById("demon-demon-name");

    this._video = document.getElementById("demon-video");
    this._video_link = document.getElementById("demon-video-link");

    this._position = document.getElementById("demon-position");
    this._requirement = document.getElementById("demon-requirement");

    this._verifier = document.getElementById("demon-verifier");
    this._publisher = document.getElementById("demon-publisher");

    this._creators = document.getElementById("demon-creators");

    let videoForm = setupFormDialogEditor(
      new PaginatorEditorBackend(this, csrfToken, false),
      "demon-video-dialog",
      "demon-video-pen",
      this.output
    );

    videoForm.addValidators({
      "demon-video-edit": {
        "Please enter a valid URL": typeMismatch,
      },
    });

    for (let errorCode of [42222, 42223, 42224, 42225]) {
      videoForm.addErrorOverride(errorCode, "demon-video-edit");
    }

    let requirementForm = setupFormDialogEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "demon-requirement-dialog",
      "demon-requirement-pen",
      this.output
    );

    requirementForm.addValidators({
      "demon-requirement-edit": {
        "Record requirement cannot be negative": rangeUnderflow,
        "Record requirement cannot be larger than 100%": rangeOverflow,
        "Record requirement must be a valid integer": badInput,
        "Record requirement mustn't be a decimal": stepMismatch,
        "Please enter a requirement value": valueMissing,
      },
    });

    requirementForm.addErrorOverride(42212, "demon-requirement-edit");

    let positionForm = setupFormDialogEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "demon-position-dialog",
      "demon-position-pen",
      this.output
    );

    positionForm.addValidators({
      "demon-position-edit": {
        "Demon position must be at least 1": rangeUnderflow,
        "Demon position must be a valid integer": badInput,
        "Demon position mustn't be a decimal": stepMismatch,
        "Please enter a position": valueMissing,
      },
    });

    positionForm.addErrorOverride(42213, "demon-position-edit");

    let nameForm = setupFormDialogEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "demon-name-dialog",
      "demon-name-pen",
      this.output
    );

    nameForm.addValidators({
      "demon-name-edit": {
        "Please provide a name for the demon": valueMissing,
      },
    });

    setupPlayerSelectionEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "demon-verifier-dialog-pagination",
      "demon-verifier-pen",
      this.output
    );
    setupPlayerSelectionEditor(
      new PaginatorEditorBackend(this, csrfToken, true),
      "demon-publisher-dialog-pagination",
      "demon-publisher-pen",
      this.output
    );
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    this._id.innerText = this.currentObject.id;
    this._name.innerText = this.currentObject.name;
    this._position.innerText = this.currentObject.position;
    this._requirement.innerText = this.currentObject.requirement;

    var embeddedVideo = embedVideo(this.currentObject.video);

    if (embeddedVideo !== undefined) {
      this._video.style.display = "block";
      this._video_link.style.display = "initial";
      this._video.src = embeddedVideo;
    } else {
      this._video.style.display = "none";
    }

    if (this.currentObject.video) {
      this._video_link.href = this.currentObject.video;
      this._video_link.innerHTML = this.currentObject.video;
    } else {
      this._video_link.style.display = "none";
    }

    this._publisher.innerHTML =
      this.currentObject.publisher.name +
      " (" +
      this.currentObject.publisher.id +
      ")";
    this._verifier.innerHTML =
      this.currentObject.verifier.name +
      " (" +
      this.currentObject.verifier.id +
      ")";
  }
}

export function initialize(csrfToken) {
  demonManager = new DemonManager(csrfToken);
  demonManager.initialize();
}

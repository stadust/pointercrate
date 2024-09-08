import {
  displayError,
  Form,
  Viewer,
  valueMissing,
  Paginator,
  setupDropdownEditor,
  PaginatorEditorBackend,
} from "/static/core/js/modules/form.js?v=4";
import { recordManager, initialize as initRecords } from "./records.js?v=4";

export let submitterManager;

function generateSubmitter(submitter) {
  var li = document.createElement("li");
  var b = document.createElement("b");

  li.className = "white";

  li.dataset.id = submitter.id;

  if (submitter.banned) {
    li.style.backgroundColor = "rgba(255, 161, 174, .3)";
  } else {
    li.style.backgroundColor = "rgba( 198, 255, 161, .3)";
  }

  b.innerText = "Submitter #" + submitter.id;

  li.appendChild(b);
  return li;
}

class SubmitterManager extends Paginator {
  constructor() {
    super("submitter-pagination", {}, generateSubmitter);

    this.output = new Viewer(
      this.html.parentNode.getElementsByClassName("viewer-content")[0],
      this
    );

    this._id = document.getElementById("submitter-submitter-id");
    this._banned = setupDropdownEditor(
      new PaginatorEditorBackend(this, true),
      "edit-submitter-banned",
      "banned",
      this.output,
      { true: true, false: false }
    );
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    this._id.innerText = this.currentObject.id;
    this._banned.selectSilently(this.currentObject.banned.toString());
  }
}

function setupSubmitterSearchSubmitterIdForm() {
  var submitterSearchByIdForm = new Form(
    document.getElementById("submitter-search-by-id-form")
  );
  var submitterId = submitterSearchByIdForm.input("search-submitter-id");

  submitterId.addValidator(valueMissing, "Submitter ID required");
  submitterSearchByIdForm.onSubmit(function () {
    submitterManager
      .selectArbitrary(parseInt(submitterId.value))
      .catch(displayError(submitterSearchByIdForm));
  });
}

export function initialize(tabber) {
  setupSubmitterSearchSubmitterIdForm();

  submitterManager = new SubmitterManager();
  submitterManager.initialize();

  document
    .getElementById("submitter-list-records")
    .addEventListener("click", () => {
      if (recordManager == null) {
        // Prevent race conditions between initialization request and the request caused by 'updateQueryData'
        initRecords().then(() => {
          recordManager.updateQueryData(
            "submitter",
            submitterManager.currentObject.id
          );
          tabber.selectPane("3");
        });
      } else {
        recordManager.updateQueryData(
          "submitter",
          submitterManager.currentObject.id
        );
        tabber.selectPane("3");
      }
    });
}

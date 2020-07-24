import {
  patch,
  displayError,
  Form,
  Dropdown,
  Viewer,
  valueMissing,
} from "../modules/form.mjs";
import { recordManager, initialize as initRecords } from "./records.js";

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

class SubmitterManager extends Viewer {
  constructor(csrfToken) {
    super("submitter-pagination", {}, generateSubmitter);

    this.currentSubmitter = null;
    this.currentSubmitterEtag = null;

    this._id = document.getElementById("submitter-submitter-id");

    this._banned = new Dropdown(
      document.getElementById("edit-submitter-banned")
    );
    this._banned.addEventListener((selected) => {
      let banned = selected == "true";

      if (banned == this.currentSubmitter.banned) return;

      patch(
        "/api/v1/submitters/" + this.currentSubmitter.id + "/",
        {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": this.currentSubmitterEtag,
        },
        { banned: banned }
      )
        .then((response) => {
          if (response.status == 304) {
            this.setSuccess("Nothing changed!");
          } else {
            this.refresh();
            this.onReceive(response);

            this.setSuccess("Submitter successfully edited!");
          }
        })
        .catch(displayError(this.errorOutput));
    });
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    this.currentSubmitter = response.data.data;
    this.currentSubmitterEtag = response.headers["etag"];

    this._id.innerText = this.currentSubmitter.id;
    this._banned.select(this.currentSubmitter.banned.toString());
  }
}

function setupSubmitterSearchSubmitterIdForm() {
  var submitterSearchByIdForm = new Form(
    document.getElementById("submitter-search-by-id-form")
  );
  var submitterId = submitterSearchByIdForm.input("search-submitter-id");

  submitterId.addValidator(valueMissing, "Submitter ID required");
  submitterSearchByIdForm.onSubmit(function (event) {
    submitterManager
      .selectArbitrary(parseInt(submitterId.value))
      .catch(displayError(submitterSearchByIdForm.errorOutput));
  });
}

export function initialize(csrfToken, tabber) {
  setupSubmitterSearchSubmitterIdForm();

  submitterManager = new SubmitterManager(csrfToken);
  submitterManager.initialize();

  document
    .getElementById("submitter-list-records")
    .addEventListener("click", () => {
      if (recordManager == null) {
        // Prevent race conditions between initialization request and the request caused by 'updateQueryData'
        initRecords(csrfToken).then(() => {
          recordManager.updateQueryData(
            "submitter",
            submitterManager.currentSubmitter.id
          );
          tabber.selectPane("3");
        });
      } else {
        recordManager.updateQueryData(
          "submitter",
          submitterManager.currentSubmitter.id
        );
        tabber.selectPane("3");
      }
    });
}

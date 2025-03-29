"use strict";

import {
  del,
  displayError,
  EditorBackend,
  Form,
  Output,
  post,
  setupEditorDialog,
  setupFormDialogEditor,
  tooShort,
  typeMismatch,
  valueMissing,
} from "/static/core/js/modules/form.js";

function setupGetAccessToken() {
  var getTokenForm = new Form(document.getElementById("get-token-form"));
  var accessTokenArea = document.getElementById("token-area");
  var accessToken = document.getElementById("access-token");

  getTokenForm.onSubmit(function () {
    post("/api/v1/auth/", {})
      .then((response) => {
        accessToken.innerText = response.data.token;
        accessTokenArea.style.display = "block";
      })
      .catch(displayError(getTokenForm));
  });
}

class ProfileEditorBackend extends EditorBackend {
  constructor(passwordInput) {
    super();

    this._pw = passwordInput;
    this._displayName = document.getElementById("profile-display-name");
    this._youtube = document.getElementById("profile-youtube-channel");
  }

  url() {
    return "/api/v1/auth/me/";
  }

  headers() {
    let headers = { "If-Match": window.etag };
    if (this._pw)
      headers["Authorization"] =
        "Basic " + btoa(window.username + ":" + this._pw.value);
    return headers;
  }

  onSuccess(response) {
    if (response.status == 204) {
      window.location.reload();
    } else {
      window.etag = response.headers["etag"];
      window.username = response.data.data.name;

      this._displayName.innerText = response.data.data.display_name || "None";
      this._youtube.removeChild(this._youtube.lastChild); // only ever has one child
      if (response.data.data.youtube_channel) {
        let a = document.createElement("a");
        a.href = response.data.data.youtube_channel;
        a.classList.add("link");
        this._youtube.appendChild(a);
      } else {
        this._youtube.innerText = "-";
      }
    }
  }
}

function setupEditAccount() {
  let output = new Output(document.getElementById("things"));

  setupFormDialogEditor(
    new ProfileEditorBackend(null),
    "edit-dn-dialog",
    "display-name-pen",
    output
  );

  let editYoutubeForm = setupFormDialogEditor(
    new ProfileEditorBackend(null),
    "edit-yt-dialog",
    "youtube-pen",
    output
  );

  editYoutubeForm.addValidators({
    "edit-yt": {
      "Please enter a valid URL": typeMismatch,
    },
  });

  editYoutubeForm.addErrorOverride(42225, "edit-yt");
  editYoutubeForm.addErrorOverride(42226, "edit-yt");

  if (document.getElementById("change-password")) {
    let changePasswordForm = setupFormDialogEditor(
      new ProfileEditorBackend(document.querySelector("#auth-pw input")), // not pretty, but oh well
      "edit-pw-dialog",
      "change-password",
      output
    );

    let editPw = changePasswordForm.input("edit-pw");

    changePasswordForm.addValidators({
      "auth-pw": {
        "Password required": valueMissing,
        "Password too short. It needs to be at least 10 characters long.":
          tooShort,
      },
      "edit-pw": {
        "Password too short. It needs to be at least 10 characters long.":
          tooShort,
      },
      "edit-pw-repeat": {
        "Password too short. It needs to be at least 10 characters long.":
          tooShort,
        "Passwords don't match": (rpp) => rpp.value == editPw.value,
      },
    });

    changePasswordForm.addErrorOverride(40100, "auth-pw");
  }

  var deleteAccountDialog = document.getElementById("delete-acc-dialog");
  var deleteAccountForm = new Form(
    deleteAccountDialog.getElementsByTagName("form")[0]
  );
  document.getElementById("delete-account").addEventListener("click", () => {
    $(deleteAccountDialog.parentElement).show();
  });

  deleteAccountForm.onSubmit(() => {
    del("/api/v1/auth/me/", {
      "If-Match": window.etag,
    })
      .then(() => window.location.reload())
      .catch(displayError(deleteAccountForm));
  });
}

function setupInvalidateToken() {
  var htmlInvalidateForm = document.getElementById("invalidate-form");
  var invalidateForm = new Form(htmlInvalidateForm);

  invalidateForm.onSubmit(function () {
    post("/api/v1/auth/invalidate/")
      .then(() => window.location.reload())
      .catch(displayError(invalidateForm));
  });
}

function googleOauthCallback(response) {
  let error = document.getElementById("g-signin-error");

  post("/api/v1/auth/oauth/google", {}, response)
    .then(() => window.location.reload())
    .catch((response) => {
      error.innerText = response.data.message;
      error.style.display = "block";
    });
}

window.googleOauthCallback = googleOauthCallback;

export function initialize() {
  setupGetAccessToken();
  setupEditAccount();
  setupInvalidateToken();
}

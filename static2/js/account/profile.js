"use strict";

import {
  Form,
  valueMissing,
  tooShort,
  post,
  Output,
  typeMismatch,
  del,
  displayError,
} from "../modules/form.mjs";
import { EditorBackend } from "../modules/form.mjs";
import { setupFormDialogEditor } from "../modules/form.mjs";
import { Input } from "../modules/form.mjs";

function setupGetAccessToken() {
  var accessTokenArea = document.getElementById("token-area");
  var accessToken = document.getElementById("access-token");
  var getTokenButton = document.getElementById("get-token");

  var htmlLoginForm = document.getElementById("login-form");
  var loginForm = new Form(htmlLoginForm);

  getTokenButton.addEventListener(
    "click",
    () => {
      getTokenButton.style.display = "none";
      accessTokenArea.style.display = "none";
      htmlLoginForm.style.display = "block";
    },
    false
  );

  var loginPassword = loginForm.input("login-password");

  loginPassword.setClearOnInvalid(true);
  loginPassword.addValidators({
    "Password required": valueMissing,
    "Password too short. It needs to be at least 10 characters long.": tooShort,
  });

  loginForm.onSubmit(function (event) {
    post("/api/v1/auth/", {
      Authorization:
        "Basic " + btoa(window.username + ":" + loginPassword.value),
    })
      .then((response) => {
        loginPassword.value = "";
        accessToken.innerHTML = response.data.token;
        htmlLoginForm.style.display = "none";
        accessTokenArea.style.display = "block";
      })
      .catch((response) => {
        if (response.data.code == 40100) {
          loginPassword.setError("Invalid credentials");
        } else {
          loginForm.setError(response.data.message);
        }
      });
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
    return {
      "If-Match": window.etag,
      Authorization: "Basic " + btoa(window.username + ":" + this._pw.value),
    };
  }

  onSuccess(response) {
    if (response.status == 204) {
      window.location.reload();
    } else {
      window.etag = response.headers["etag"];
      window.username = response.data.data.name;

      this._displayName.innerText = response.data.data.display_name || "-";
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
  let editDisplayNameForm = setupFormDialogEditor(
    new ProfileEditorBackend(new Input(document.getElementById("auth-dn"))), // not pretty, but oh well
    "edit-dn-dialog",
    "display-name-pen",
    output
  );

  editDisplayNameForm.addValidators({
    "auth-dn": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort,
    },
  });

  editDisplayNameForm.addErrorOverride(40100, "auth-dn");

  let editYoutubeForm = setupFormDialogEditor(
    new ProfileEditorBackend(new Input(document.getElementById("auth-yt"))), // not pretty, but oh well
    "edit-yt-dialog",
    "youtube-pen",
    output
  );

  editYoutubeForm.addValidators({
    "edit-yt": {
      "Please enter a valid URL": typeMismatch,
    },
    "auth-yt": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort,
    },
  });

  editYoutubeForm.addErrorOverride(40100, "auth-yt");
  editYoutubeForm.addErrorOverride(42225, "edit-yt");

  let changePasswordForm = setupFormDialogEditor(
    new ProfileEditorBackend(new Input(document.getElementById("auth-pw"))), // not pretty, but oh well
    "edit-pw-dialog",
    "change-password",
    output
  );

  let editPw = changePasswordForm.input("edit-pw");

  changePasswordForm.addValidators({
    "auth-pw": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort,
    },
    "edit-pw": {
      "Password too short. It needs to be at least 10 characters long.": tooShort,
    },
    "edit-pw-repeat": {
      "Password too short. It needs to be at least 10 characters long.": tooShort,
      "Passwords don't match": (rpp) => rpp.value == editPw.value,
    },
  });

  changePasswordForm.addErrorOverride(40100, "auth-pw");

  var deleteAccountDialog = document.getElementById("delete-acc-dialog");
  var deleteAccountForm = new Form(
    deleteAccountDialog.getElementsByTagName("form")[0]
  );
  document.getElementById("delete-account").addEventListener("click", () => {
    $(deleteAccountDialog.parentElement).show();
  });

  var deleteAuth = deleteAccountForm.input("auth-delete");
  deleteAuth.addValidators({
    "Password required": valueMissing,
    "Password too short. It needs to be at least 10 characters long.": tooShort,
  });

  deleteAccountForm.addErrorOverride(40100, "auth-delete");

  deleteAccountForm.onSubmit(() => {
    del("/api/v1/auth/me/", {
      "If-Match": window.etag,
      Authorization: "Basic " + btoa(window.username + ":" + deleteAuth.value),
    })
      .then(() => window.location.reload())
      .catch(displayError(deleteAccountForm));
  });
}

function setupInvalidateToken() {
  var invalidateButton = document.getElementById("invalidate-token");
  var htmlInvalidateForm = document.getElementById("invalidate-form");
  var invalidateForm = new Form(htmlInvalidateForm);

  invalidateButton.addEventListener(
    "click",
    () => {
      invalidateButton.style.display = "none";
      htmlInvalidateForm.style.display = "block";
    },
    false
  );

  var invalidatePassword = invalidateForm.input("invalidate-auth-password");

  invalidatePassword.setClearOnInvalid(true);
  invalidateForm.addValidators({
    "invalidate-auth-password": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort,
    },
  });

  invalidateForm.onSubmit(function (event) {
    post("/api/v1/auth/invalidate/", {
      Authorization:
        "Basic " + btoa(window.username + ":" + invalidatePassword.value),
    })
      .then((response) => window.location.reload())
      .catch((response) => {
        if (response.data.code == 40100) {
          loginPassword.setError("Invalid credentials");
        } else {
          invalidateForm.setError(response.data.message);
        }
      });
  });
}

export function initialize() {
  setupGetAccessToken();
  setupEditAccount();
  setupInvalidateToken();
}

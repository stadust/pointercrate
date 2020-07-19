"use strict";

import {
  Form,
  valueMissing,
  tooShort,
  post,
  patch,
  typeMismatch,
} from "../modules/form.mjs";
import { displayError } from "../modules/form.mjs";
import { del } from "../modules/form.mjs";

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

function setupEditAccount() {
  var editDisplayNameDialog = document.getElementById("edit-dn-dialog");
  var editDisplayNameForm = new Form(
    editDisplayNameDialog.getElementsByTagName("form")[0]
  );
  document.getElementById("display-name-pen").addEventListener("click", () => {
    $(editDisplayNameDialog.parentElement).show();
  });

  var authPassword = editDisplayNameForm.input("auth-dn");

  authPassword.addValidators({
    "Password required": valueMissing,
    "Password too short. It needs to be at least 10 characters long.": tooShort,
  });

  function editHandler(form, auth, handlers) {
    return () => {
      patch(
        "/api/v1/auth/me/",
        {
          "If-Match": window.etag,
          Authorization: "Basic " + btoa(window.username + ":" + auth.value),
        },
        form.serialize()
      )
        .then((response) => {
          if (response.status == 304) {
            form.setSuccess("Nothing changed!");
          } else {
            window.location.reload();
          }
        })
        .catch(displayError(form.errorOutput, handlers));
    };
  }

  editDisplayNameForm.onSubmit(
    editHandler(editDisplayNameForm, authPassword, {
      40100: () => authPassword.setError("Invalid credentials"),
      41200: () =>
        editDisplayNameForm.setError(
          "Concurrent account access was made. Please reload the page"
        ),
      41800: () =>
        editDisplayNameForm.setError(
          "Concurrent account access was made. Please reload the page"
        ),
    })
  );

  var editYoutubeDialog = document.getElementById("edit-yt-dialog");
  var editYoutubeForm = new Form(
    editYoutubeDialog.getElementsByTagName("form")[0]
  );
  document.getElementById("youtube-pen").addEventListener("click", () => {
    $(editYoutubeDialog.parentElement).show();
  });

  var ytAuth = editYoutubeForm.input("auth-yt");
  var editYt = editYoutubeForm.input("edit-yt");

  editYoutubeForm.addValidators({
    "edit-yt": {
      "Please enter a valid URL": typeMismatch,
    },
    "auth-yt": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort,
    },
  });

  editYoutubeForm.onSubmit(
    editHandler(editYoutubeForm, ytAuth, {
      40100: () => ytAuth.setError("Invalid credentials"),
      41200: () =>
        editYoutubeForm.setError(
          "Concurrent account access was made. Please reload the page"
        ),
      41800: () =>
        editYoutubeForm.setError(
          "Concurrent account access was made. Please reload the page"
        ),
      42225: () => editYt.setError(response.data.message),
    })
  );

  var changePasswordDialog = document.getElementById("edit-pw-dialog");
  var changePasswordForm = new Form(
    changePasswordDialog.getElementsByTagName("form")[0]
  );
  document.getElementById("change-password").addEventListener("click", () => {
    $(changePasswordDialog.parentElement).show();
  });

  var pwAuth = changePasswordForm.input("auth-pw");
  var editPw = changePasswordForm.input("edit-pw");
  var editPwRepeat = changePasswordForm.input("edit-pw-repeat");

  pwAuth.addValidators({
    "Password required": valueMissing,
    "Password too short. It needs to be at least 10 characters long.": tooShort,
  });
  editPw.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );
  editPwRepeat.addValidators({
    "Password too short. It needs to be at least 10 characters long.": tooShort,
    "Passwords don't match": (rpp) => rpp.value == editPw.value,
  });

  changePasswordForm.onSubmit(
    editHandler(changePasswordForm, pwAuth, {
      40100: () => ytAuth.setError("Invalid credentials"),
      41200: () =>
        editYoutubeForm.setError(
          "Concurrent account access was made. Please reload the page"
        ),
      41800: () =>
        editYoutubeForm.setError(
          "Concurrent account access was made. Please reload the page"
        ),
    })
  );

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

  deleteAccountForm.onSubmit(() => {
    del("/api/v1/auth/me/", {
      "If-Match": window.etag,
      Authorization: "Basic " + btoa(window.username + ":" + deleteAuth.value),
    })
      .then(() => window.location.reload())
      .catch(
        displayError(deleteAccountForm.errorOutput, {
          40100: () => deleteAuth.setError("Invalid credentials"),
          41200: () =>
            deleteAccountForm.setError(
              "Concurrent account access was made. Please reload the page"
            ),
          41800: () =>
            deleteAccountForm.setError(
              "Concurrent account access was made. Please reload the page"
            ),
        })
      );
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

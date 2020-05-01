"use strict";

import {
  Form,
  valueMissing,
  tooShort,
  post,
  patch,
  typeMismatch
} from "../modules/form.mjs";

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
    "Password too short. It needs to be at least 10 characters long.": tooShort
  });

  loginForm.onSubmit(function(event) {
    post("/api/v1/auth/", {
      Authorization:
        "Basic " + btoa(window.username + ":" + loginPassword.value)
    })
      .then(response => {
        loginPassword.value = "";
        accessToken.innerHTML = response.data.token;
        htmlLoginForm.style.display = "none";
        accessTokenArea.style.display = "block";
      })
      .catch(response => {
        if (response.data.code == 40100) {
          loginPassword.setError("Invalid credentials");
        } else {
          loginForm.setError(response.data.message);
        }
      });
  });
}

function setupEditAccount() {
  var editForm = new Form(document.getElementById("edit-form"));

  var editYtChannel = editForm.input("edit-yt-channel");
  var editPassword = editForm.input("edit-password");
  var authPassword = editForm.input("auth-password");

  editForm.addValidators({
    "edit-yt-channel": {
      "Please enter a valid URL": typeMismatch
    },
    "edit-password": {
      "Password too short. It needs to be at least 10 characters long.": tooShort
    },
    "edit-password-repeat": {
      "Password too short. It needs to be at least 10 characters long.": tooShort,
      "Passwords don't match": rpp => rpp.value == editPassword.value
    },
    "auth-password": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort
    }
  });

  editForm.onSubmit(function(event) {
    patch(
      "/api/v1/auth/me/",
      {
        "If-Match": window.etag,
        Authorization:
          "Basic " + btoa(window.username + ":" + authPassword.value)
      },
      editForm.serialize()
    )
      .then(response => {
        if (response.status == 304) {
          editForm.setSuccess("Nothing changed!");
        }
        window.location.reload();
      })
      .catch(response => {
        switch (response.data.code) {
          case 40100:
            authPassword.setError("Invalid credentials");
            break;
          case 41200:
            editForm.setError(
              "Concurrent account access was made. Please reload the page"
            );
            break;
          case 41800:
            editForm.setError(
              "Concurrent account access was made. Please reload the page"
            );
            break;
          case 42225:
            editYtChannel.setError(response.data.message);
            break;
          default:
            editForm.setError(response.data.message);
        }
      });
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
      "Password too short. It needs to be at least 10 characters long.": tooShort
    }
  });

  invalidateForm.onSubmit(function(event) {
    post("/api/v1/auth/invalidate/", {
      Authorization:
        "Basic " + btoa(window.username + ":" + invalidatePassword.value)
    })
      .then(response => window.location.reload())
      .catch(response => {
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

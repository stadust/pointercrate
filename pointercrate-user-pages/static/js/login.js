import {
  Form,
  valueMissing,
  tooShort,
  post,
} from "/static/core/js/modules/form.js";

function initializeLoginForm() {
  var loginForm = new Form(document.getElementById("login-form"));

  var loginUsername = loginForm.input("login-username");
  var loginPassword = loginForm.input("login-password");

  loginUsername.addValidator(valueMissing, "Username required");
  loginUsername.addValidator(
    tooShort,
    "Username too short. It needs to be at least 3 characters long."
  );

  loginPassword.clearOnInvalid = true;
  loginPassword.addValidator(valueMissing, "Password required");
  loginPassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );

  loginForm.onSubmit(function (event) {
    post("/login/", {
      Authorization:
        "Basic " + btoa(loginUsername.value + ":" + loginPassword.value),
    })
      .then((response) => {
        window.location = "/account/";
      })
      .catch((response) => {
        console.log(response);
        if (response.status === 401) {
          loginPassword.errorText = "Invalid credentials";
        } else {
          loginForm.setError(response.data.message);
        }
      });
  });
}

function googleOauthCallback(response) {
  let error = document.getElementById("g-signin-error");

  post("/api/v1/auth/oauth/google", {}, response)
    .then(() => (window.location = "/account/"))
    .catch((response) => {
      error.innerText = response.data.message;
      error.style.display = "block";
    });
}

window.googleOauthCallback = googleOauthCallback;

$(document).ready(function () {
  initializeLoginForm();
});

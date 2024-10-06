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
    "Username too short. It needs to be at least 3 characters long.",
  );

  loginPassword.clearOnInvalid = true;
  loginPassword.addValidator(valueMissing, "Password required");
  loginPassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long.",
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

function intializeGoogleForm() {
  var registerForm = new Form(document.getElementById("google-form"));

  registerForm.onSubmit(function (event) {
    window.location = "/api/v1/auth/authorize";
  });
}

$(document).ready(function () {
  initializeLoginForm();
  intializeGoogleForm();
});

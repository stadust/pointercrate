import { Form, valueMissing, tooShort, post } from "./modules/form.mjs";

function initializeLoginForm() {
  var loginForm = new Form(document.getElementById("login-form"));

  var loginUsername = loginForm.input("login-username");
  var loginPassword = loginForm.input("login-password");

  loginUsername.addValidator(valueMissing, "Username required");
  loginUsername.addValidator(
    tooShort,
    "Username too short. It needs to be at least 3 characters long."
  );

  loginPassword.setClearOnInvalid(true);
  loginPassword.addValidator(valueMissing, "Password required");
  loginPassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );

  loginForm.onSubmit(function(event) {
    post("/login/", {
      Authorization:
        "Basic " + btoa(loginUsername.value + ":" + loginPassword.value)
    })
      .then(response => {
        window.location = "/account/";
      })
      .catch(response => {
        console.log(response);
        if (response.status === 401) {
          loginPassword.setError("Invalid credentials");
        } else {
          loginForm.setError(response.data.message);
        }
      });
  });
}

function intializeRegisterForm() {
  var registerForm = new Form(document.getElementById("register-form"));

  var registerUsername = registerForm.input("register-username");
  var registerPassword = registerForm.input("register-password");
  var registerPasswordRepeat = registerForm.input("register-password-repeat");

  registerUsername.addValidator(valueMissing, "Username required");
  registerUsername.addValidator(
    tooShort,
    "Username too short. It needs to be at least 3 characters long."
  );

  registerPassword.addValidator(valueMissing, "Password required");
  registerPassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );

  registerPasswordRepeat.addValidator(valueMissing, "Password required");
  registerPasswordRepeat.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );
  registerPasswordRepeat.addValidator(
    rpp => rpp.value == registerPassword.value,
    "Passwords don't match"
  );

  registerForm.onSubmit(function(event) {
    post("/register/", {}, registerForm.serialize())
      .then(response => {
        window.location = "/account/";
      })
      .catch(response => {
        if (response.status === 409) {
          registerUsername.setError(
            "This username is already taken. Please choose another one"
          );
        } else {
          registerForm.setError(response.data.message);
        }
      });
  });
}

$(document).ready(function() {
  initializeLoginForm();
  intializeRegisterForm();
});

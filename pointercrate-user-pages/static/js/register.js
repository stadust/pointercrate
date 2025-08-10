import {
  displayError,
  Form,
  FormDialog,
  post,
  tooShort,
  valueMissing,
} from "/static/core/js/modules/form.js";
import { tr } from "/static/core/js/modules/localization.js";

function intializeRegisterForm() {
  var registerForm = new Form(document.getElementById("register-form"));

  var registerUsername = registerForm.input("register-username");
  var registerPassword = registerForm.input("register-password");
  var registerPasswordRepeat = registerForm.input("register-password-repeat");

  registerUsername.addValidator(
    valueMissing,
    tr("user", "user", "auth-username.validator-valuemissing")
  );
  registerUsername.addValidator(
    tooShort,
    tr("user", "user", "auth-username.validator-tooshort")
  );

  registerPassword.addValidator(
    valueMissing,
    tr("user", "user", "auth-password.validator-valuemissing")
  );
  registerPassword.addValidator(
    tooShort,
    tr("user", "user", "auth-password.validator-tooshort")
  );

  registerPasswordRepeat.addValidator(
    valueMissing,
    tr("user", "user", "auth-password.validator-valuemissing")
  );
  registerPasswordRepeat.addValidator(
    tooShort,
    tr("user", "user", "auth-password.validator-valuemissing")
  );
  registerPasswordRepeat.addValidator(
    (rpp) => rpp.value == registerPassword.value,
    tr("user", "user", "auth-repeatpassword.validator-notmatching")
  );

  registerForm.onSubmit(function (event) {
    post("/register/", {}, registerForm.serialize())
      .then((response) => {
        window.location = "/account/";
      })
      .catch((response) => {
        if (response.status === 409) {
          registerUsername.errorText = tr(
            "user",
            "user",
            "auth-username.error-alreadytaken"
          );
        } else {
          registerForm.setError(response.data.message);
        }
      });
  });
}

function googleOauthRegisterCallback(response) {
  let dialog = new FormDialog("oauth-registration-pick-username");
  dialog.form.addErrorOverride(40902, "oauth-username");
  dialog.form.onSubmit(() => {
    let formData = dialog.form.serialize();
    formData["credential"] = response["credential"];
    post("/api/v1/auth/oauth/google/register/", {}, formData)
      .then(() => (window.location = "/account/"))
      .catch(displayError(dialog.form));
  });
  dialog.open();
}

window.googleOauthRegisterCallback = googleOauthRegisterCallback;

$(document).ready(function () {
  intializeRegisterForm();
});

$(document).ready(function() {
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
    $.ajax({
      method: "POST",
      url: "/login/",
      dataType: "json",
      headers: {
        Authorization:
          "Basic " + btoa(loginUsername.value + ":" + loginPassword.value)
      },
      error: function(data) {
        loginPassword.setError("Invalid credentials");
      },
      success: function() {
        window.location = "/account";
      }
    });
  });

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
    $.ajax({
      method: "POST",
      url: "/api/v1/auth/register/",
      contentType: "application/json",
      dataType: "json",
      data: JSON.stringify({
        name: registerUsername.value,
        password: registerPassword.value
      }),
      statusCode: {
        409: function() {
          registerUsername.setError(
            "This username is already taken. Please choose another one"
          );
        }
      },
      success: function() {
        $.ajax({
          method: "POST",
          url: "/login/",
          headers: {
            Authorization:
              "Basic " +
              btoa(registerUsername.value + ":" + registerPassword.value)
          },
          success: function() {
            window.location = "/account/";
          }
        });
      }
    });
  });
});

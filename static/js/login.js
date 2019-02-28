$(document).ready(function() {
  var htmlLoginForm = document.getElementById("login-form");
  var loginForm = new Form(htmlLoginForm);

  var loginUsername = loginForm.input("login-username");
  var loginPassword = loginForm.input("login-password");

  var loginError = htmlLoginForm.getElementsByClassName("output")[0];

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
    loginError.style.display = "";

    $.ajax({
      method: "POST",
      url: "/login/",
      dataType: "json",
      headers: {
        Authorization:
          "Basic " + btoa(loginUsername.value + ":" + loginPassword.value)
      },
      error: function(data) {
        if (data.status == 401) {
          loginPassword.setError("Invalid credentials");
        } else {
          loginError.innerHTML = data.responseJSON.message;
          loginError.style.display = "initial";
        }
      },
      success: function() {
        window.location = "/account";
      }
    });
  });

  var htmlRegisterForm = document.getElementById("register-form");
  var registerForm = new Form(htmlRegisterForm);

  var registerUsername = registerForm.input("register-username");
  var registerPassword = registerForm.input("register-password");
  var registerPasswordRepeat = registerForm.input("register-password-repeat");

  var registerError = htmlRegisterForm.getElementsByClassName("output")[0];

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
    registerError.style.display = "";

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
          error: function(data) {
            registerError.innerHTML = data.responseJSON.message;
            registerError.style.display = "initial";
          },
          success: function() {
            window.location = "/account/";
          }
        });
      }
    });
  });
});

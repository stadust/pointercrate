$(document).ready(function() {
  var loginForm = document.getElementById("login-form");
  var loginUsername = document.getElementById("login-username");
  var loginUsernameInput = loginUsername.getElementsByTagName("input")[0];
  var loginUsernameError = loginUsername.getElementsByTagName("p")[0];
  var loginPassword = document.getElementById("login-password");
  var loginPasswordInput = loginPassword.getElementsByTagName("input")[0];
  var loginPasswordError = loginPassword.getElementsByTagName("p")[0];

  var registerForm = document.getElementById("register-form");
  var registerUsername = document.getElementById("register-username");
  var registerUsernameInput = registerUsername.getElementsByTagName("input")[0];
  var registerUsernameError = registerUsername.getElementsByTagName("p")[0];
  var registerPassword = document.getElementById("register-password");
  var registerPasswordInput = registerPassword.getElementsByTagName("input")[0];
  var registerPasswordError = registerPassword.getElementsByTagName("p")[0];
  var registerPasswordRepeat = document.getElementById(
    "register-password-repeat"
  );
  var registerPasswordRepeatInput = registerPasswordRepeat.getElementsByTagName(
    "input"
  )[0];
  var registerPasswordRepeatError = registerPasswordRepeat.getElementsByTagName(
    "p"
  )[0];

  resetErrorOnInput(loginUsernameInput, loginUsernameError);
  resetErrorOnInput(loginPasswordInput, loginPasswordError);
  resetErrorOnInput(registerUsernameInput, registerUsernameError);
  resetErrorOnInput(registerPasswordInput, registerPasswordError);
  resetErrorOnInput(registerPasswordRepeatInput, registerPasswordRepeatError);

  loginForm.addEventListener(
    "submit",
    function(event) {
      // we do custom ajax below
      event.preventDefault();

      // We only want one '&' here since '&&' short-circuits, but the validate functions aren't pure, so we don't want that
      var valid =
        validateUsername(loginUsernameInput, loginUsernameError) &
        validatePassword(loginPasswordInput, loginPasswordError);

      if (!valid) {
        loginPasswordInput.value = "";

        return false;
      }

      $.ajax({
        method: "POST",
        url: "/api/v1/auth/",
        username: loginUsernameInput.value,
        password: loginPasswordInput.value,
        error: function(data) {
          loginPasswordError.innerHTML = "Invalid credentials";
          loginPasswordInput.value = "";
        },
        success: function() {
          window.location = "/account";
        }
      });
    },
    false
  );

  registerForm.addEventListener(
    "submit",
    function(event) {
      event.preventDefault();

      var valid =
        validateUsername(registerUsernameInput, registerUsernameError) &
        validatePassword(registerPasswordInput, registerPasswordError) &
        validatePassword(
          registerPasswordRepeatInput,
          registerPasswordRepeatError
        );

      if (registerPasswordInput.value != registerPasswordRepeatInput.value) {
        registerPasswordRepeatError.innerHTML = "Passwords don't match";

        valid = false;
      }

      if (!valid) {
        registerPasswordRepeatInput.value = "";

        return false;
      }

      $.ajax({
        method: "POST",
        url: "/api/v1/auth/register/",
        contentType: "application/json",
        dataType: "json",
        data: JSON.stringify({
          name: registerUsernameInput.value,
          password: registerPasswordInput.value
        }),
        statusCode: {
          409: function() {
            registerUsernameError.innerHTML =
              "This username is already taken. Please choose another one";
            registerUsernameInput.setCustomValidity("Invalid field.");
          }
        },
        success: function() {
          $.ajax({
            method: "POST",
            url: "/api/v1/auth/",
            username: registerUsernameInput.value,
            password: registerPasswordInput.value,
            success: function() {
              window.location = "/account";
            }
          });
        }
      });
    },
    false
  );
});

function validateUsername(input, error) {
  if (!input.validity.valid) {
    if (input.validity.tooShort) {
      error.innerHTML =
        "Username too short. It needs to be at least 3 characters long.";
    } else if (input.validity.valueMissing) {
      error.innerHTML = "Username required";
    } else {
      error.innerHTML = "Invalid username";
    }

    return false;
  }
  return true;
}

function validatePassword(input, error) {
  if (!input.validity.valid) {
    if (input.validity.tooShort) {
      error.innerHTML =
        "Password too short. It needs to be at least 10 characters long.";
    } else if (input.validity.valueMissing) {
      error.innerHTML = "Password required";
    } else {
      error.innerHTML = "Invalid password";
    }

    return false;
  }
  return true;
}

function resetErrorOnInput(input, error) {
  input.addEventListener(
    "input",
    function(_) {
      if (input.validity.valid || input.validity.customError) {
        error.innerHTML = "";
        input.setCustomValidity("");
      }
    },
    false
  );
}

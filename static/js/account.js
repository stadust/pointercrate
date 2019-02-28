$(document).ready(function() {
  var accessTokenArea = document.getElementById("token-area");
  var accessToken = document.getElementById("access-token");
  var getTokenButton = document.getElementById("get-token");

  getTokenButton.addEventListener(
    "click",
    function(event) {
      accessTokenArea.style.display = "none";
      htmlLoginForm.style.display = "block";
    },
    false
  );

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
      url: "/api/v1/auth/",
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
      success: function(crap, more_crap, data) {
        loginPassword.value = "";
        accessToken.innerHTML = data.responseJSON.token;
        htmlLoginForm.style.display = "none";
        accessTokenArea.style.display = "block";
      }
    });
  });
});

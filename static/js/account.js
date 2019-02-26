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
      url: "/api/v1/auth/",
      dataType: "json",
      headers: {
        Authorization:
          "Basic " + btoa(loginUsername.value + ":" + loginPassword.value)
      },
      error: function(data) {
        loginPassword.setError("Invalid credentials");
      },
      success: function(crap, more_crap, data) {
        $("#access-token").text(data.responseJSON.token);
      }
    });
  });
});

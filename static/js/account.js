$(document).ready(function() {
  var accessTokenArea = document.getElementById("token-area");
  var accessToken = document.getElementById("access-token");
  var getTokenButton = document.getElementById("get-token");

  getTokenButton.addEventListener(
    "click",
    function(event) {
      getTokenButton.style.display = "none";
      accessTokenArea.style.display = "none";
      htmlLoginForm.style.display = "block";
    },
    false
  );

  var htmlLoginForm = document.getElementById("login-form");
  var loginForm = new Form(htmlLoginForm);

  var loginPassword = loginForm.input("login-password");

  var loginError = htmlLoginForm.getElementsByClassName("output")[0];

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
          "Basic " + btoa(window.username + ":" + loginPassword.value)
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

  var htmlEditForm = document.getElementById("edit-form");
  var editForm = new Form(htmlEditForm);

  var editDisplayName = editForm.input("edit-display-name");
  var editYtChannel = editForm.input("edit-yt-channel");
  var editPassword = editForm.input("edit-password");
  var editPasswordRepeat = editForm.input("edit-password-repeat");
  var authPassword = editForm.input("auth-password");

  var editError = htmlEditForm.getElementsByClassName("output")[0];

  editYtChannel.addValidator(typeMismatch, "Please enter a valid URL");

  editPassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );

  editPasswordRepeat.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );
  editPasswordRepeat.addValidator(
    rpp => rpp.value == editPassword.value,
    "Passwords don't match"
  );

  authPassword.addValidator(valueMissing, "Password required");
  authPassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );

  editForm.onSubmit(function(event) {
    editError.style.display = "none";

    data = {};

    if (editDisplayName.value) data["display_name"] = editDisplayName.value;
    if (editYtChannel.value) data["youtube_channel"] = editYtChannel.value;
    if (editPassword.value) data["password"] = editPassword.value;

    $.ajax({
      method: "PATCH",
      url: "/api/v1/auth/me/",
      dataType: "json",
      contentType: "application/json",
      headers: {
        "If-Match": window.etag,
        Authorization:
          "Basic " + btoa(window.username + ":" + authPassword.value)
      },
      data: JSON.stringify(data),
      error: function(data) {
        switch (data.status) {
          case 401:
            authPassword.setError("Invalid credentials");
            break;
          case 412:
          case 418:
            editError.innerHTML =
              "Concurrent account access was made. Please reload the page";
            editError.innerHTML = "initial";
            break;
          default:
            editError.innerHTML = data.responseJSON.message;
            editError.style.display = "initial";
            break;
        }
      },
      success: function() {
        window.location.reload();
      }
    });
  });

  var invalidateButton = document.getElementById("invalidate-token");

  invalidateButton.addEventListener(
    "click",
    function(event) {
      invalidateButton.style.display = "none";
      htmlInvalidateForm.style.display = "block";
    },
    false
  );

  var htmlInvalidateForm = document.getElementById("invalidate-form");
  var invalidateForm = new Form(htmlInvalidateForm);

  var invalidatePassword = invalidateForm.input("invalidate-auth-password");
  var invalidateError = htmlInvalidateForm.getElementsByClassName("output")[0];

  invalidatePassword.setClearOnInvalid(true);
  invalidatePassword.addValidator(valueMissing, "Password required");
  invalidatePassword.addValidator(
    tooShort,
    "Password too short. It needs to be at least 10 characters long."
  );
  invalidateForm.onSubmit(function(event) {
    invalidateError.style.display = "";

    $.ajax({
      method: "POST",
      url: "/api/v1/auth/invalidate/",
      dataType: "json",
      headers: {
        Authorization:
          "Basic " + btoa(window.username + ":" + invalidatePassword.value)
      },
      error: function(data) {
        if (data.status == 401) {
          invalidatePassword.setError("Invalid credentials");
        } else {
          invalidateError.innerHTML = data.responseJSON.message;
          invalidateError.style.display = "initial";
        }
      },
      success: function(crap, more_crap, data) {
        window.location.reload();
      }
    });
  });
});

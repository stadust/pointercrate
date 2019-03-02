$(document).ready(function() {
  var csrfTokenSpan = document.getElementById("chicken-salad-red-fish");
  var csrfToken = csrfTokenSpan.innerHTML;

  csrfTokenSpan.remove();

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
          loginError.style.display = "block";
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
            editError.innerHTML = "block";
            break;
          default:
            editError.innerHTML = data.responseJSON.message;
            editError.style.display = "block";
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
          invalidateError.style.display = "block";
        }
      },
      success: function(crap, more_crap, data) {
        window.location.reload();
      }
    });
  });

  var htmlEditForm2 = document.getElementById("patch-permissions");
  var editForm2 = new Form(htmlEditForm2);
  var edit2Error = htmlEditForm2.getElementsByClassName("output")[0];
  var edit2Success = htmlEditForm2.getElementsByClassName("output")[1];

  var extended = editForm2.input("perm-extended");
  var list_helper = editForm2.input("perm-list-helper");
  var list_mod = editForm2.input("perm-list-mod");
  var list_admin = editForm2.input("perm-list-admin");
  var mod = editForm2.input("perm-mod");
  var admin = editForm2.input("perm-admin");

  var text = document.getElementById("text");

  var htmlUserByIdForm = document.getElementById("find-id-form");
  var userByIdForm = new Form(htmlUserByIdForm);
  var userByIdError = htmlUserByIdForm.getElementsByClassName("output")[0];

  var userId = userByIdForm.input("find-id");

  userId.addValidator(valueMissing, "User ID required");

  userByIdForm.onSubmit(function(event) {
    userByIdError.style.display = "";

    $.ajax({
      method: "GET",
      url: "/api/v1/users/" + userId.value + "/",
      dataType: "json",
      error: function(data) {
        switch (data.responseJSON.code) {
          case 40401:
            userId.setError(data.responseJSON.message);
            break;
          default:
            userByIdError.innerHTML = data.responseJSON.message;
            userByIdError.style.display = "block";
            break;
        }
      },
      success: function(crap, mor_crap, data) {
        window.currentUser = data.responseJSON.data;
        window.currentUser.etag = data.getResponseHeader("ETag");

        if (window.currentUser.display_name) {
          text.innerHTML =
            "<b>Username: </b>" +
            window.currentUser.name +
            " (" +
            window.currentUser.display_name +
            ")" +
            "&nbsp;&nbsp;&nbsp;&nbsp;<b>User ID: </b>" +
            window.currentUser.id;
        } else {
          text.innerHTML =
            "<b>Username: </b>" +
            window.currentUser.name +
            "&nbsp;&nbsp;&nbsp;&nbsp;<b>User ID: </b>" +
            window.currentUser.id;
        }

        var bitmask = window.currentUser.permissions;

        extended.value = (bitmask & 0x1) == 0x1;
        list_helper.value = (bitmask & 0x2) == 0x2;
        list_mod.value = (bitmask & 0x4) == 0x4;
        list_admin.value = (bitmask & 0x8) == 0x8;
        mod.value = (bitmask & 0x2000) == 0x2000;
        admin.value = (bitmask & 0x4000) == 0x4000;

        htmlEditForm2.style.display = "block";
      }
    });
  });

  editForm2.onSubmit(function(event) {
    $.ajax({
      method: "PATCH",
      url: "/api/v1/users/" + window.currentUser.id + "/",
      dataType: "json",
      contentType: "application/json",
      headers: {
        "X-CSRF-TOKEN": csrfToken,
        "If-Match": window.currentUser.etag
      },
      data: JSON.stringify({
        permissions:
          extended.value * 0x1 +
          list_helper.value * 0x2 +
          list_mod.value * 0x4 +
          list_admin.value * 0x8 +
          mod.value * 0x2000 +
          admin.value * 0x4000
      }),
      error: function(data) {
        edit2Success.style.display = "";
        edit2Error.innerHTML = data.responseJSON.message;
        edit2Error.style.display = "block";
      },
      success: function(crap, crap2, data) {
        edit2Error.style.display = "";
        if (data.status == 200) {
          window.currentUser = data.responseJSON.data;
          window.currentUser.etag = data.getResponseHeader("ETag");

          edit2Error.style.display = "";
          edit2Success.style.display = "block";
          edit2Success.innerHTML = "Successfully modified user!";
        } else {
          edit2Error.style.display = "";
          edit2Success.style.display = "block";
          edit2Success.innerHTML = "No changes made!";
        }
      }
    });
  });
});

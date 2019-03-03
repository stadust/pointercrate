"use strict";

function setupGetAccessToken() {
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
}

function setupEditAccount() {
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

    var data = {};

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
            editError.style.display = "block";
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
}

function setupInvalidateToken() {
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
}

function setupEditUser() {}

$(document).ready(function() {
  var csrfTokenSpan = document.getElementById("chicken-salad-red-fish");
  var csrfToken = csrfTokenSpan.innerHTML;

  csrfTokenSpan.remove();

  setupGetAccessToken();
  setupEditAccount();
  setupInvalidateToken();

  var deleteUserButton = document.getElementById("delete-user");

  deleteUserButton.addEventListener(
    "click",
    function(event) {
      $.ajax({
        method: "DELETE",
        url: "/api/v1/users/" + window.currentUser.id + "/",
        headers: {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": window.currentUser.etag
        },
        error: function(data) {
          edit2Success.style.display = "";
          edit2Error.innerHTML = data.responseJSON.message;
          edit2Error.style.display = "block";
        },
        success: function(data) {
          edit2Error.style.display = "";
          edit2Success.style.display = "block";
          edit2Success.innerHTML = "Successfully deleted user!";
        }
      });
    },
    false
  );

  var htmlEditForm = document.getElementById("patch-permissions");
  var editForm2 = new Form(htmlEditForm);
  var edit2Error = htmlEditForm.getElementsByClassName("output")[0];
  var edit2Success = htmlEditForm.getElementsByClassName("output")[1];

  var extended = editForm2.input("perm-extended");
  var list_helper = editForm2.input("perm-list-helper");
  var list_mod = editForm2.input("perm-list-mod");
  var list_admin = editForm2.input("perm-list-admin");
  var mod = editForm2.input("perm-mod");
  var admin = editForm2.input("perm-admin");

  var text = document.getElementById("text");

  var htmlUserByIdForm = document.getElementById("find-id-form");
  var htmlUserByNameForm = document.getElementById("find-name-form");
  var userByIdForm = new Form(htmlUserByIdForm);
  var userByNameForm = new Form(htmlUserByNameForm);
  var userByIdError = htmlUserByIdForm.getElementsByClassName("output")[0];
  var userByNameError = htmlUserByNameForm.getElementsByClassName("output")[0];

  var userId = userByIdForm.input("find-id");
  var userName = userByNameForm.input("find-name");

  userId.addValidator(valueMissing, "User ID required");
  userName.addValidator(valueMissing, "Username required");

  function requestUserForEdit(userId, onError) {
    edit2Error.style.display = "";
    edit2Success.style.display = "";

    $.ajax({
      method: "GET",
      url: "/api/v1/users/" + userId + "/",
      dataType: "json",
      error: onError,
      success: function(crap, mor_crap, data) {
        window.currentUser = data.responseJSON.data;
        window.currentUser.etag = data.getResponseHeader("ETag");

        if (window.currentUser.name == window.username) {
          edit2Error.style.display = "block";
          edit2Error.innerHTML =
            "This is your own account. You cannot modify your own account using this interface!";
        }

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

        htmlEditForm.style.display = "block";
      }
    });
  }

  userByIdForm.onSubmit(function(event) {
    userByIdError.style.display = "";

    requestUserForEdit(userId.value, function(data) {
      switch (data.responseJSON.code) {
        case 40401:
          userId.setError(data.responseJSON.message);
          break;
        default:
          userByIdError.innerHTML = data.responseJSON.message;
          userByIdError.style.display = "block";
          break;
      }
    });
  });

  userByNameForm.onSubmit(function(event) {
    userByNameError.style.display = "";

    $.ajax({
      method: "GET",
      url: "/api/v1/users/?name=" + userName.value,
      dataType: "json",
      error: function(data) {
        userByNameError.innerHTML = data.responseJSON.message;
        userByNameError.style.display = "block";
      },
      success: function(crap, mor_crap, data) {
        var json = data.responseJSON;

        console.log(json);

        if (!json || json.length == 0) {
          userName.setError("No user with that name found!");
        } else {
          requestUserForEdit(json[0].id, function(data) {
            userByNameError.innerHTML = data.responseJSON.message;
            userByNameError.style.display = "block";
          });
        }
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

  var loadUsersButton = document.getElementById("load-users");
  var loadUsersError = document.getElementById("load-users-error");
  var userList = document.getElementById("user-list");
  var nextUserButton = document.getElementById("next-user");
  var prevUserButton = document.getElementById("prev-user");

  function populateUserlist(data) {
    loadUsersButton.style.display = "none";

    window.userPagination = parsePagination(data.getResponseHeader("Links"));
    var userString = "";

    for (var user of data.responseJSON) {
      userString +=
        "<li data-uid=" +
        user.id +
        "><b>" +
        user.name +
        "</b> (ID: " +
        user.id +
        ")<br><i>Display name: " +
        (user.display_name || "None") +
        "</i>";
    }

    userList.innerHTML = userString;

    for (var li of userList.getElementsByTagName("li")) {
      // javascript is really fucking stupid
      function set(li) {
        li.addEventListener(
          "click",
          () => requestUserForEdit(li.dataset.uid),
          false
        );
      }
      set(li);
    }

    document.getElementById("hidden-user-list").style.display = "block";
  }

  loadUsersButton.addEventListener("click", () =>
    makeRequest("GET", "/users/?limit=5", loadUsersError, populateUserlist)
  );

  nextUserButton.addEventListener("click", () => {
    if (window.userPagination.next) {
      makeRequest(
        "GET",
        window.userPagination.next,
        loadUsersError,
        populateUserlist
      );
    }
  });

  prevUserButton.addEventListener("click", () => {
    if (window.userPagination.prev) {
      makeRequest(
        "GET",
        window.userPagination.prev,
        loadUsersError,
        populateUserlist
      );
    }
  });
});

function parsePagination(linkHeader) {
  var links = {};
  if (linkHeader) {
    for (var link of linkHeader.split(",")) {
      var s = link.split(";");

      links[s[1].substring(5)] = s[0].substring(8, s[0].length - 1);
    }
  }
  return links;
}

function makeRequest(
  method,
  endpoint,
  errorOutput,
  onSuccess,
  errorCodes = {},
  headers = {},
  data = {}
) {
  errorOutput.style.display = "";

  headers["Accept"] = "application/json";

  $.ajax({
    method: method,
    url: "/api/v1" + endpoint,
    contentType: "application/json",
    data: JSON.stringify(data),
    headers: headers,
    error: function(data, code, errorThrown) {
      if (!data.responseJSON) {
        errorOutput.innerHTML =
          "Server unexpectedly returned " + code + " (" + errorThrown + ")";
        errorOutput.style.display = "block";
      } else {
        var error = data.responseJSON;

        if (error.code in errorCodes) {
          errorCodes[error.code](error.message, error.data);
        } else {
          errorOutput.innerHTML = error.message;
          errorOutput.style.display = "block";
        }
      }
    },
    success: function(crap, crap2, data) {
      errorOutput.style.display = "";

      onSuccess(data);
    }
  });
}

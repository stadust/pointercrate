"use strict";

function setupGetAccessToken() {
  var accessTokenArea = document.getElementById("token-area");
  var accessToken = document.getElementById("access-token");
  var getTokenButton = document.getElementById("get-token");

  var htmlLoginForm = document.getElementById("login-form");
  var loginForm = new Form(htmlLoginForm);

  getTokenButton.addEventListener(
    "click",
    function(event) {
      getTokenButton.style.display = "none";
      accessTokenArea.style.display = "none";
      htmlLoginForm.style.display = "block";
    },
    false
  );

  var loginPassword = loginForm.input("login-password");

  loginPassword.setClearOnInvalid(true);
  loginPassword.addValidators({
    "Password required": valueMissing,
    "Password too short. It needs to be at least 10 characters long.": tooShort
  });

  loginForm.onSubmit(function(event) {
    makeRequest(
      "POST",
      "/api/v1/auth/",
      htmlLoginForm.getElementsByClassName("output")[0],
      function(data) {
        loginPassword.value = "";
        accessToken.innerHTML = data.responseJSON.token;
        htmlLoginForm.style.display = "none";
        accessTokenArea.style.display = "block";
      },
      { 40100: () => loginPassword.setError("Invalid credentials") },
      {
        Authorization:
          "Basic " + btoa(window.username + ":" + loginPassword.value)
      }
    );
  });
}

function setupEditAccount() {
  var editForm = new Form(document.getElementById("edit-form"));

  var editDisplayName = editForm.input("edit-display-name");
  var editYtChannel = editForm.input("edit-yt-channel");
  var editPassword = editForm.input("edit-password");
  var authPassword = editForm.input("auth-password");

  editForm.addValidators({
    "edit-yt-channel": {
      "Please enter a valid URL": typeMismatch
    },
    "edit-password": {
      "Password too short. It needs to be at least 10 characters long.": tooShort
    },
    "edit-password-repeat": {
      "Password too short. It needs to be at least 10 characters long.": tooShort,
      "Passwords don't match": rpp => rpp.value == editPassword.value
    },
    "auth-password": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort
    }
  });

  editForm.onSubmit(function(event) {
    var data = {};

    if (editDisplayName.value) data["display_name"] = editDisplayName.value;
    if (editYtChannel.value) data["youtube_channel"] = editYtChannel.value;
    if (editPassword.value) data["password"] = editPassword.value;

    makeRequest(
      "PATCH",
      "/api/v1/auth/me/",
      editForm.errorOutput,
      () => window.location.reload(),
      {
        40100: () => authPassword.setError("Invalid credentials"),
        41200: () =>
          editForm.setError(
            "Concurrent account access was made. Please reload the page"
          ),
        41800: () =>
          editForm.setError(
            "Concurrent account access was made. Please reload the page"
          ),
        42225: message => editYtChannel.setError(message)
      },
      {
        "If-Match": window.etag,
        Authorization:
          "Basic " + btoa(window.username + ":" + authPassword.value)
      },
      data
    );
  });
}

function setupInvalidateToken() {
  var invalidateButton = document.getElementById("invalidate-token");
  var htmlInvalidateForm = document.getElementById("invalidate-form");
  var invalidateForm = new Form(htmlInvalidateForm);

  invalidateButton.addEventListener(
    "click",
    function(event) {
      invalidateButton.style.display = "none";
      htmlInvalidateForm.style.display = "block";
    },
    false
  );

  var invalidatePassword = invalidateForm.input("invalidate-auth-password");

  invalidatePassword.setClearOnInvalid(true);
  invalidateForm.addValidators({
    "invalidate-auth-password": {
      "Password required": valueMissing,
      "Password too short. It needs to be at least 10 characters long.": tooShort
    }
  });

  invalidateForm.onSubmit(function(event) {
    makeRequest(
      "POST",
      "/api/v1/auth/invalidate/",
      invalidateForm.errorOutput,
      () => window.location.reload(),
      { 40100: () => loginPassword.setError("Invalid credentials") },
      {
        Authorization:
          "Basic " + btoa(window.username + ":" + invalidatePassword.value)
      }
    );
  });
}

function setupDeleteUser(csrfToken) {
  var deleteUserButton = document.getElementById("delete-user");
  var editForm = window.patchUserPermissionsForm;

  deleteUserButton.addEventListener(
    "click",
    function(event) {
      makeRequest(
        "DELETE",
        "/api/v1/users/" + window.currentUser.id + "/",
        window.patchUserPermissionsForm.errorOutput,
        () => editForm.setSuccess("Successfully deleted user!"),
        {},
        {
          "X-CSRF-TOKEN": csrfToken,
          "If-Match": window.currentUser.etag
        }
      );
    },
    false
  );
}

function setupPatchUserPermissionsForm(csrfToken) {
  var htmlEditForm = document.getElementById("patch-permissions");
  window.patchUserPermissionsForm = new Form(htmlEditForm);

  var editForm = window.patchUserPermissionsForm;

  editForm.onSubmit(function(event) {
    makeRequest(
      "PATCH",
      "/api/v1/users/" + window.currentUser.id + "/",
      editForm.errorOutput,
      data => {
        if (data.status == 200) {
          window.currentUser = data.responseJSON.data;
          window.currentUser.etag = data.getResponseHeader("ETag");

          editForm.setSuccess("Successfully modified user!");
        } else {
          editForm.setSuccess("No changes made!");
        }
      },
      {},
      {
        "X-CSRF-TOKEN": csrfToken,
        "If-Match": window.currentUser.etag
      },
      {
        permissions:
          editForm.input("perm-extended").value * 0x1 +
          editForm.input("perm-list-helper").value * 0x2 +
          editForm.input("perm-list-mod").value * 0x4 +
          editForm.input("perm-list-admin").value * 0x8 +
          editForm.input("perm-mod").value * 0x2000 +
          editForm.input("perm-admin").value * 0x4000
      }
    );
  });
}

function setupUserByIdForm() {
  var userByIdForm = new Form(document.getElementById("find-id-form"));
  var userId = userByIdForm.input("find-id");

  userId.addValidator(valueMissing, "User ID required");
  userByIdForm.onSubmit(function(event) {
    makeRequest(
      "GET",
      "/api/v1/users/" + userId.value + "/",
      userByIdForm.errorOutput,
      response => window.userPaginator.onReceive(response),
      {
        40401: message => userId.setError(message)
      }
    );
  });
}

function setupUserByNameForm() {
  var userByNameForm = new Form(document.getElementById("find-name-form"));
  var userName = userByNameForm.input("find-name");

  userName.addValidators({
    "Username required": valueMissing,
    "Username is at least 3 characters long": tooShort
  });

  userByNameForm.onSubmit(function(event) {
    makeRequest(
      "GET",
      "/api/v1/users/?name=" + userName.value,
      userByNameForm.errorOutput,
      data => {
        let json = data.responseJSON;

        if (!json || json.length == 0) {
          userName.setError("No user with that name found!");
        } else {
          makeRequest(
            "GET",
            "/api/v1/users/" + json[0].id + "/",
            userByNameForm.errorOutput,
            response => window.userPaginator.onReceive(response)
          );
        }
      }
    );
  });
}

$(document).ready(function() {
  var csrfTokenSpan = document.getElementById("chicken-salad-red-fish");
  var csrfToken = csrfTokenSpan.innerHTML;

  csrfTokenSpan.remove();

  setupGetAccessToken();
  setupEditAccount();
  setupInvalidateToken();

  TABBED_PANES["account-tabber"].addSwitchListener("2", () => {
    if (window.userPaginator === undefined) {
      window.userPaginator = new UserPaginator();
      window.userPaginator.initialize();
    }

    setupDeleteUser(csrfToken);
    setupPatchUserPermissionsForm(csrfToken);
    setupUserByIdForm();
    setupUserByNameForm();
  });
});

function generateUser(userData) {
  var li = document.createElement("li");
  var b = document.createElement("b");
  var i = document.createElement("i");

  li.dataset.id = userData.id;

  b.appendChild(document.createTextNode(userData.name));
  i.appendChild(
    document.createTextNode(
      "Display name: " + (userData.display_name || "None")
    )
  );

  li.appendChild(b);
  li.appendChild(document.createTextNode(" (ID: " + userData.id + ")"));
  li.appendChild(document.createElement("br"));
  li.appendChild(i);

  return li;
}

class UserPaginator extends Paginator {
  constructor() {
    super("user-pagination", { limit: 5 }, generateUser);
  }

  onReceive(response) {
    let editForm = window.patchUserPermissionsForm;

    window.currentUser = response.responseJSON.data;
    window.currentUser.etag = response.getResponseHeader("ETag");

    editForm.setError(null);

    if (window.currentUser.name == window.username) {
      editForm.setError(
        "This is your own account. You cannot modify your own account using this interface!"
      );
    }

    var text = document.getElementById("text"); // TODO: What ever the fuck was I thinking when I named this

    if (window.window.currentUser.display_name) {
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

    let bitmask = window.currentUser.permissions;

    editForm.input("perm-extended").value = (bitmask & 0x1) == 0x1;
    editForm.input("perm-list-helper").value = (bitmask & 0x2) == 0x2;
    editForm.input("perm-list-mod").value = (bitmask & 0x4) == 0x4;
    editForm.input("perm-list-admin").value = (bitmask & 0x8) == 0x8;
    editForm.input("perm-mod").value = (bitmask & 0x2000) == 0x2000;
    editForm.input("perm-admin").value = (bitmask & 0x4000) == 0x4000;

    editForm.html.style.display = "block";
  }
}

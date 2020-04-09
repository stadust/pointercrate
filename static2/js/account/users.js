"use strict";

import {
  del,
  displayError,
  patch,
  valueMissing,
  tooShort,
  get,
  Paginator,
  Form
} from "../modules/form.mjs";

let selectedUser;
let userPaginator;
let editForm;

function setupPatchUserPermissionsForm(csrfToken) {
  editForm = new Form(document.getElementById("patch-permissions"));
  editForm.onSubmit(function(event) {
    patch(
      "/api/v1/users/" + selectedUser.id + "/",
      {
        "X-CSRF-TOKEN": csrfToken,
        "If-Match": selectedUser.etag
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
    )
      .then(response => {
        if (response.status == 200) {
          selectedUser = response.data.data;
          selectedUser.etag = response.headers["etag"];

          editForm.setSuccess("Successfully modified user!");
        } else {
          editForm.setSuccess("No changes made!");
        }
      })
      .catch(displayError(editForm.errorOutput));
  });

  let deleteUserButton = document.getElementById("delete-user");

  if (deleteUserButton) {
    // The button isn't generated server sided for people who don't have permissions to delete users (aka aren't pointercrate admins)
    deleteUserButton.addEventListener("click", () => {
      del("/api/v1/users/" + selectedUser.id + "/", {
        "X-CSRF-TOKEN": csrfToken,
        "If-Match": selectedUser.etag
      })
        .then(response => editForm.setSuccess("Successfully deleted user!"))
        .catch(displayError(editForm.errorOutput));
    });
  }
}

function setupUserByIdForm() {
  var userByIdForm = new Form(document.getElementById("find-id-form"));
  var userId = userByIdForm.input("find-id");

  userId.addValidator(valueMissing, "User ID required");

  userByIdForm.onSubmit(function(event) {
    get("/api/v1/users/" + userId.value + "/")
      .then(response => userPaginator.onReceive(response))
      .catch(response => {
        if (response.data.code == 40401) {
          userId.setError(response.data.message);
        } else {
          userByIdForm.setError(response.data.message);
        }
      });
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
    get("/api/v1/users/?name=" + userName.value)
      .then(response => {
        if (!response.data || response.data.length == 0) {
          userName.setError("No user with that name found!");
        } else {
          get("/api/v1/users/" + response.data[0].id + "/").then(response =>
            userPaginator.onReceive(response)
          );
        }
      })
      .catch(displayError(userByNameForm.errorOutput));
  });
}

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
    super("user-pagination", { limit: 10 }, generateUser);
  }

  onReceive(response) {
    selectedUser = response.data.data;
    selectedUser.etag = response.headers["etag"];

    editForm.setError(null);

    if (selectedUser.name == window.username) {
      editForm.setError(
        "This is your own account. You cannot modify your own account using this interface!"
      );
    }

    var text = document.getElementById("text"); // TODO: What ever the fuck was I thinking when I named this

    while (text.lastChild) {
      text.removeChild(text.lastChild);
    }

    let b = document.createElement("b");
    b.innerText = "Username: ";

    text.appendChild(b);
    text.appendChild(document.createTextNode(selectedUser.name));

    if (selectedUser.display_name) {
      text.appendChild(
        document.createTextNode(" (" + selectedUser.display_name + ")")
      );
    }

    let b2 = document.createElement("b");
    b2.innerText = " - User ID: ";

    text.appendChild(b2);
    text.appendChild(document.createTextNode(selectedUser.id));

    let bitmask = selectedUser.permissions;

    editForm.input("perm-extended").value = (bitmask & 0x1) == 0x1;
    editForm.input("perm-list-helper").value = (bitmask & 0x2) == 0x2;
    editForm.input("perm-list-mod").value = (bitmask & 0x4) == 0x4;
    editForm.input("perm-list-admin").value = (bitmask & 0x8) == 0x8;
    editForm.input("perm-mod").value = (bitmask & 0x2000) == 0x2000;
    editForm.input("perm-admin").value = (bitmask & 0x4000) == 0x4000;

    editForm.html.style.display = "block";
  }
}

export function initialize(csrfToken) {
  setupPatchUserPermissionsForm(csrfToken);
  setupUserByIdForm();
  setupUserByNameForm();

  userPaginator = new UserPaginator();
  userPaginator.initialize();
}

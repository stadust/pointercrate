"use strict";

import {
  del,
  displayError,
  patch,
  valueMissing,
  FilteredPaginator,
  Form,
  Viewer,
} from "/static/core/js/modules/form.js?v=4";

let selectedUser;
let userPaginator;
let editForm;

function setupPatchUserPermissionsForm() {
  editForm = new Form(document.getElementById("patch-permissions"));
  editForm.onSubmit(function () {
    patch(
      "/api/v1/users/" + selectedUser.id + "/",
      {
        "If-Match": selectedUser.etag,
      },
      {
        permissions: editForm.inputs.map(input => input.value * parseInt(input.span.dataset.bit)).reduce((a,b) => a+b, 0)
      }
    )
      .then((response) => {
        if (response.status == 200) {
          selectedUser = response.data.data;
          selectedUser.etag = response.headers["etag"];

          editForm.setSuccess("Successfully modified user!");
        } else {
          editForm.setSuccess("No changes made!");
        }
      })
      .catch(displayError(editForm));
  });

  let deleteUserButton = document.getElementById("delete-user");

  if (deleteUserButton) {
    // The button isn't generated server sided for people who don't have permissions to delete users (aka aren't pointercrate admins)
    deleteUserButton.addEventListener("click", () => {
      del("/api/v1/users/" + selectedUser.id + "/", {
        "If-Match": selectedUser.etag,
      })
        .then(() => editForm.setSuccess("Successfully deleted user!"))
        .catch(displayError(editForm));
    });
  }
}

function setupUserByIdForm() {
  var userByIdForm = new Form(document.getElementById("find-id-form"));
  var userId = userByIdForm.input("find-id");

  userId.addValidator(valueMissing, "User ID required");

  userByIdForm.onSubmit(function () {
    userPaginator.selectArbitrary(userId.value).catch((response) => {
      if (response.data.code == 40401) {
        userId.errorText = response.data.message;
      } else {
        userByIdForm.setError(response.data.message);
      }
    });
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

class UserPaginator extends FilteredPaginator {
  constructor() {
    super("user-pagination", generateUser, "name_contains", { limit: 10 });

    this.output = new Viewer(
      this.html.parentNode.getElementsByClassName("viewer-content")[0],
      this
    );
  }

  onReceive(response) {
    super.onReceive(response);

    selectedUser = response.data.data;
    selectedUser.etag = response.headers["etag"];

    editForm.setError(null);

    if (selectedUser.name == window.username) {
      editForm.setError(
        "This is your own account. You cannot modify your own account using this interface!"
      );
      for(let btn of this.output.html.getElementsByTagName("input")) {
        btn.classList.add("disabled");
        btn.disabled = true;
      }
    } else {
      for(let btn of this.output.html.getElementsByTagName("input")) {
        btn.classList.remove("disabled");
        btn.disabled = false;
      }
    }

    document.getElementById("user-user-name").innerText = selectedUser.name;
    document.getElementById("user-user-id").innerText = selectedUser.id;
    document.getElementById("user-display-name").innerText =
      selectedUser.display_name || "None";

    let bitmask = selectedUser.permissions;

    for(let input of editForm.inputs) {
      let bit = parseInt(input.span.dataset.bit);

      input.value = (bitmask & bit) === bit;
    }

    editForm.html.style.display = "block";
  }
}

export function initialize() {
  setupPatchUserPermissionsForm();
  setupUserByIdForm();

  userPaginator = new UserPaginator();
  userPaginator.initialize();
}

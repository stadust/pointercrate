import {Form, post, tooShort, valueMissing} from "/static/core/js/modules/form";


function intializeRegisterForm() {
    var registerForm = new Form(document.getElementById("register-form"));

    var registerUsername = registerForm.input("register-username");
    var registerPassword = registerForm.input("register-password");
    var registerPasswordRepeat = registerForm.input("register-password-repeat");

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
        (rpp) => rpp.value == registerPassword.value,
        "Passwords don't match"
    );

    registerForm.onSubmit(function (event) {
        post("/register/", {}, registerForm.serialize())
            .then((response) => {
                window.location = "/account/";
            })
            .catch((response) => {
                if (response.status === 409) {
                    registerUsername.errorText =
                        "This username is already taken. Please choose another one";
                } else {
                    registerForm.setError(response.data.message);
                }
            });
    });
}

$(document).ready(function () {

    intializeRegisterForm();
})
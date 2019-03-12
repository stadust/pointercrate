class Input {
  constructor(span) {
    this.span = span;
    this.input = span.getElementsByTagName("input")[0];
    this.error = span.getElementsByTagName("p")[0];
    this.clearOnInvalid = false;
    this.validators = [];

    this.input.addEventListener(
      "input",
      () => {
        if (this.validity.valid || this.validity.customError) {
          this.resetError();
        }
      },
      false
    );
  }

  resetError() {
    if (this.error) this.error.innerHTML = "";
    this.input.setCustomValidity("");
  }

  setError(errorString) {
    this.resetError();
    this.appendError(errorString);
  }

  appendError(errorString) {
    if (this.error) {
      if (this.error.innerHTML != "") {
        this.error.innerHTML += "<br>";
      }

      this.error.innerHTML += errorString;
    }
    this.input.setCustomValidity(this.error.innerHTML);

    if (this.clearOnInvalid) {
      this.value = "";
    }
  }

  addValidator(validator, msg) {
    this.validators.push({
      validator: validator,
      message: msg
    });
  }

  addValidators(validators) {
    Object.keys(validators).forEach(message =>
      this.addValidator(validators[message], message)
    );
  }

  setClearOnInvalid(clear) {
    this.clearOnInvalid = clear;
  }

  validate(event) {
    this.resetError();

    var isValid = this.validity.valid;

    for (var validator of this.validators) {
      if (!validator.validator(this, event)) {
        isValid = false;

        if (typeof validator.message === "string") {
          this.appendError(validator.message);
        } else {
          this.appendError(validator.message(this.value));
        }
      }
    }

    if (!isValid && this.clearOnInvalid) {
      this.value = "";
    }

    return isValid;
  }

  get id() {
    return this.span.id;
  }

  get validity() {
    return this.input.validity;
  }

  get value() {
    if (this.input.type == "checkbox") {
      return this.input.checked;
    }
    return this.input.value;
  }

  set value(value) {
    if (this.input.type == "checkbox") {
      this.input.checked = value;
    } else {
      this.input.value = value;
    }
  }
}

class Form {
  constructor(form) {
    this.inputs = [];
    this.submitHandler = undefined;
    this.invalidHandler = undefined;
    this.errorOutput = form.getElementsByClassName("output")[0];
    this.successOutput = form.getElementsByClassName("output")[1];

    for (var input of form.getElementsByClassName("form-input")) {
      this.inputs.push(new Input(input));
    }

    form.addEventListener(
      "submit",
      event => {
        event.preventDefault();

        if (this.errorOutput) this.errorOutput.style.display = "none";
        if (this.successOutput) this.successOutput.style.display = "none";

        var isValid = true;

        for (var input of this.inputs) {
          isValid &= input.validate(event);
        }

        if (isValid) {
          if (this.submitHandler != undefined) {
            this.submitHandler(event);
          }
        } else if (this.invalidHandler != undefined) {
          this.invalidHandler();
        }
      },
      false
    );
  }

  setError(message) {
    if (this.successOutput) this.successOutput.style.display = "none";

    this.errorOutput.innerHTML = message;
    this.errorOutput.style.display = "block";
  }

  setSuccess(message) {
    if (this.errorOutput) this.errorOutput.style.display = "none";

    this.successOutput.innerHTML = message;
    this.successOutput.style.display = "block";
  }

  onSubmit(handler) {
    this.submitHandler = handler;
  }

  onInvalid(handler) {
    this.invalidHandler = handler;
  }

  input(id) {
    for (var input of this.inputs) {
      if (input.id == id) {
        return input;
      }
    }
    return null;
  }

  value(id) {
    this.input(id).value();
  }

  addValidators(validators) {
    Object.keys(validators).forEach(input_id =>
      this.input(input_id).addValidators(validators[input_id])
    );
  }
}

function badInput(input) {
  return !input.validity.badInput;
}

function patternMismatch(input) {
  return !input.validity.patternMismatch;
}

function rangeOverflow(input) {
  return !input.validity.rangeOverflow;
}

function rangeUnderflow(input) {
  return !input.validity.rangeUnderflow;
}

function stepMismatch(input) {
  return !input.validity.stepMismatch;
}

function tooLong(input) {
  return !input.validity.tooLong;
}

function tooShort(input) {
  return !input.validity.tooShort;
}

function typeMismatch(input) {
  return !input.validity.typeMismatch;
}

function valueMissing(input) {
  return !input.validity.valueMissing;
}

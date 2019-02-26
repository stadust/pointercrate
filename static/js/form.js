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
    this.error.innerHTML = "";
    this.input.setCustomValidity("");
  }

  setError(errorString) {
    this.resetError();
    this.appendError(errorString);
  }

  appendError(errorString) {
    if (this.error.innerHTML != "") {
      this.error.innerHTML += "<br>";
    }

    this.error.innerHTML += errorString;
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
    return this.input.value;
  }

  set value(value) {
    this.input.value = value;
  }
}

class Form {
  constructor(form) {
    this.inputs = [];
    this.submitHandler = undefined;
    this.invalidHandler = undefined;

    for (var input of form.getElementsByClassName("form-input")) {
      this.inputs.push(new Input(input));
    }

    form.addEventListener(
      "submit",
      event => {
        event.preventDefault();

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

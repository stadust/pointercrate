/**
 * Class for those dropdown selectors we use throughout the website
 */
export class Dropdown {
  /**
   * Creates an instance of Dropdown.
   * @param {HTMLElement} html
   * @memberof Dropdown
   */
  constructor(html) {
    this.html = html;
    this.input = this.html.getElementsByTagName("input")[0];
    this.menu = $(this.html.getElementsByClassName("menu")[0]); // we need jquery for the animations
    this.listeners = [];

    this.values = {};

    for (let li of this.html.querySelectorAll("ul li")) {
      li.addEventListener("click", () => this.select(li.dataset.value));

      this.values[li.dataset.value] = li.dataset.display || li.innerHTML;
    }

    this.selected = this.input.dataset.default;
    this.input.value = this.values[this.selected]; // in case some browser randomly decide to store text field values

    // temporarily variable to store selection while we clear the text field when the dropdown is opened
    var value;

    this.input.addEventListener("focus", () => {
      value = this.input.value;
      this.input.value = "";
      this.input.dispatchEvent(new Event("change"));
      this.menu.fadeTo(300, 0.95);
    });

    this.input.addEventListener("focusout", () => {
      this.menu.fadeOut(300);
      this.input.value = value;
    });
  }

  select(entry) {
    if (entry in this.values) {
      this.selected = entry;
      this.input.value = this.values[entry];

      for (let listener of this.listeners) {
        listener(entry);
      }
    }
  }

  addEventListener(listener) {
    this.listeners.push(listener);
  }
}

export class Paginator {
  /**
   * Creates an instance of Paginator. Retrieves its endpoint from the `data-endpoint` data attribute of `html`.
   *
   * @param {String} elementId The Id of the DOM element of this paginator
   * @param {Object} queryData The initial query data to use
   * @param {*} itemConstructor Callback used to construct the list items of this Paginator
   * @memberof Paginator
   */
  constructor(elementId, queryData, itemConstructor) {
    this.html = document.getElementById(elementId);

    // Next and previous buttons
    this.next = this.html.getElementsByClassName("next")[0];
    this.prev = this.html.getElementsByClassName("prev")[0];

    // The li that was last clicked and thus counts as "selected"
    this.currentlySelected = null;

    // The endpoint which will be paginated. By storing this, we assume that the 'Links' header never redirects
    // us to a different endpoint (this is the case with the pointercrate API)
    this.endpoint = this.html.dataset.endpoint;
    // The link for the request that was made to display the current data (required for refreshing)
    this.currentLink = this.endpoint + "?" + $.param(queryData);
    // The query data for the first request. Pagination may only update the 'before' and 'after' parameter,
    // meaning everything else will always stay the same.
    // Storing this means we won't have to parse the query data of the links from the 'Links' header, and allows
    // us to easily update some parameters later on
    this.queryData = queryData;

    // The (parsed) values of the HTTP 'Links' header, telling us how what requests to make then next or prev is clicked
    this.links = undefined;
    // The callback that constructs list entries for us
    this.itemConstructor = itemConstructor;

    // The list displaying the results of the request
    this.list = this.html.getElementsByClassName("selection-list")[0];

    // Some HTML element where we will display errors messages
    this.errorOutput = this.html.getElementsByClassName("output")[0];

    this.nextHandler = this.onNextClick.bind(this);
    this.prevHandler = this.onPreviousClick.bind(this);

    if (this.html.style.display === "none") {
      this.html.style.display = "block";
    }

    this.next.addEventListener("click", this.nextHandler, false);
    this.prev.addEventListener("click", this.prevHandler, false);
  }

  /**
   * Programmatically selects an object with the given id
   *
   * The selected object does not have to be currently visible in the paginator. On success, `onReceive` is called. Returns a promise without a registered error handler (meaning the error message will not automatically get displayed in the paginator)
   *
   * @param id The ID of the object to select
   *
   * @returns A promise
   */
  selectArbitrary(id) {
    return get(this.endpoint + id + "/").then(this.onReceive.bind(this));
  }

  /**
   * Realizes a callback for when a user selects a list item.
   *
   * The default implementation takes the value of the `data-id` attribute of the selected item,
   * concatenates it to the pagination request URL,
   * makes a request to that URL and calls `onReceive` with the result
   *
   * @param {*} selected The selected list item
   * @memberof Paginator
   */
  onSelect(selected) {
    this.currentlySelected = selected;
    this.selectArbitrary(selected.dataset.id).catch(
      displayError(this.errorOutput)
    );
  }

  /**
   * Realizes a callback for when the request made in onSelect is successful
   *
   * @param {*} response
   * @memberof Paginator
   */
  onReceive(response) {}

  /**
   * Initializes this Paginator by making the request using the query data specified in the constructor.
   *
   * Calling any other method on this before calling initialize is considered an error.
   * Calling this more than once has no additional effect.
   *
   * @memberof Paginator
   */
  initialize() {
    if (this.links === undefined) this.refresh();
  }

  handleResponse(response) {
    this.links = parsePagination(response.headers["links"]);
    this.list.scrollTop = 0;

    // Clear the current list.
    // list.innerHtml = '' is horrible and should never be used. It causes memory leaks and is terribly slow
    while (this.list.lastChild) {
      this.list.removeChild(this.list.lastChild);
    }

    for (var result of response.data) {
      let item = this.itemConstructor(result);
      item.addEventListener("click", e => this.onSelect(e.currentTarget));
      this.list.appendChild(item);
    }
  }

  /**
   * Updates a single key in the query data. Refreshes the paginator and resets it to the first page,
   * meaning 'before' and 'after' fields are reset to the values they had at the time of construction.
   *
   * @param {String} key The key
   * @param {String} value The value
   * @memberof Paginator
   */
  updateQueryData(key, value) {
    if (value === undefined) delete this.queryData[key];
    else this.queryData[key] = value;

    this.currentLink = this.endpoint + "?" + $.param(this.queryData);
    this.refresh();
  }

  /**
   * Sets this Paginators query data, overriding the values provided at the time of construction. Refreshes the paginator by making a request with the given query data
   *
   * @param {*} queryData The new query data
   * @memberof Paginator
   */
  setQueryData(queryData) {
    this.queryData = queryData;
    this.currentLink = this.endpoint + "?" + $.param(queryData);
    this.refresh();
  }

  /**
   * Refreshes the paginator, by reissuing the request that was made to display the current data
   *
   * @memberof Paginator
   */
  refresh() {
    get(this.currentLink)
      .then(this.handleResponse.bind(this))
      .catch(displayError(this.errorOutput));
  }

  onPreviousClick() {
    if (this.links.prev) {
      get(this.links.prev)
        .then(this.handleResponse.bind(this))
        .catch(displayError(this.errorOutput));
    }
  }

  onNextClick() {
    if (this.links.next) {
      get(this.links.next)
        .then(this.handleResponse.bind(this))
        .catch(displayError(this.errorOutput));
    }
  }

  stop() {
    this.next.removeEventListener("click", this.nextHandler, false);
    this.prev.removeEventListener("click", this.prevHandler, false);
  }
}

function parsePagination(linkHeader) {
  var links = {};
  if (linkHeader) {
    for (var link of linkHeader.split(",")) {
      var s = link.split(";");

      links[s[1].substring(5)] = s[0].substring(1, s[0].length - 1);
    }
  }
  return links;
}

/**
 * A Wrapper around a paginator that includes a search/filter bar at the top
 *
 * @class FilteredPaginator
 */
export class FilteredPaginator extends Paginator {
  /**
   * Creates an instance of FilteredPaginator.
   *
   * @param {String} paginatorID HTML id of this viewer
   * @param {*} itemConstructor Callback used to construct the list entries on the left side
   * @param {String} filterParam Name of the API field that should be set for filtering the list
   * @memberof FilteredPaginator
   */
  constructor(paginatorID, itemConstructor, filterParam) {
    super(paginatorID, {}, itemConstructor);

    let filterInput = this.html.getElementsByTagName("input")[0];

    filterInput.value = "";

    // Apply filter when enter is pressed
    filterInput.addEventListener("keypress", event => {
      if (event.keyCode == 13) {
        this.updateQueryData(filterParam, filterInput.value);
      }
    });

    // Apply filter when input is changed externally
    filterInput.addEventListener("change", () =>
      this.updateQueryData(filterParam, filterInput.value)
    );

    filterInput.parentNode.addEventListener("click", event => {
      if (event.offsetX > filterInput.offsetWidth) {
        filterInput.value = "";
        this.updateQueryData(filterParam, "");
      }
    });

    var timeout = undefined;

    // Upon input, wait a second before applying the filter (to ensure the user is actually done writing in the text field)
    filterInput.addEventListener("input", () => {
      if (timeout) {
        clearTimeout(timeout);
      }

      timeout = setTimeout(
        () => this.updateQueryData(filterParam, filterInput.value),
        1000
      );
    });
  }
}

export class Input {
  constructor(span) {
    this.span = span;
    this.input =
      span.getElementsByTagName("input")[0] ||
      span.getElementsByTagName("textarea")[0];
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

  // TODO: maybe just make this a normal `set` property lol
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

  get name() {
    return this.input.name;
  }

  get type() {
    if (this.input.tagName == "textarea") {
      return "text";
    }
    return this.input.type;
  }

  get value() {
    // extend this switch to other input types as required.
    switch (this.type) {
      case "checkbox":
        return this.input.checked;
      case "number":
        if (this.input.value === "" || this.input.value === null) return null;
        return parseInt(this.input.value);
      case "text": // also handles the text area case
      default:
        if (this.input.value === "" || this.input.value === null) return null;
        return this.input.value;
    }
  }

  set value(value) {
    if (this.input.type == "checkbox") {
      this.input.checked = value;
    } else {
      this.input.value = value;
    }
  }
}

export class Form {
  constructor(form) {
    this.html = form;
    this.inputs = [];
    this.submitHandler = undefined;
    this.invalidHandler = undefined;
    this.errorOutput = form.getElementsByClassName("output")[0];
    this.successOutput = form.getElementsByClassName("output")[1];
    this._clearOnSubmit = false;

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

        for (let input of this.inputs) {
          isValid &= input.validate(event);
        }

        if (isValid) {
          if (this.submitHandler !== undefined) {
            // todo: maybe just pass the result of .serialize here?
            this.submitHandler(event);

            if (this._clearOnSubmit) {
              for (let input of this.inputs) {
                input.value = "";
              }
            }
          }
        } else if (this.invalidHandler !== undefined) {
          this.invalidHandler();
        }
      },
      false
    );
  }

  setClearOnSubmit(clear) {
    this._clearOnSubmit = clear;
  }

  serialize() {
    let data = {};

    for (let input of this.inputs) {
      if (input.value !== null) {
        data[input.name] = input.value;
      }
    }

    return data;
  }

  setError(message) {
    if (this.successOutput) this.successOutput.style.display = "none";

    if (message === null || message === undefined) {
      this.errorOutput.style.display = "none";
    } else {
      this.errorOutput.innerHTML = message;
      this.errorOutput.style.display = "block";
    }
  }

  setSuccess(message) {
    if (this.errorOutput) this.errorOutput.style.display = "none";

    if (message === null || message === undefined) {
      this.successOutput.style.display = "none";
    } else {
      this.successOutput.innerHTML = message;
      this.successOutput.style.display = "block";
    }
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

export function badInput(input) {
  return !input.validity.badInput;
}

export function patternMismatch(input) {
  return !input.validity.patternMismatch;
}

export function rangeOverflow(input) {
  return !input.validity.rangeOverflow;
}

export function rangeUnderflow(input) {
  return !input.validity.rangeUnderflow;
}

export function stepMismatch(input) {
  return !input.validity.stepMismatch;
}

export function tooLong(input) {
  return !input.validity.tooLong;
}

export function tooShort(input) {
  return !input.validity.tooShort;
}

export function typeMismatch(input) {
  return !input.validity.typeMismatch;
}

export function valueMissing(input) {
  return !input.validity.valueMissing;
}

/**
 * Standard error handler for a promise returned by `get`, `post`, `del` or `patch` which displays the error message in an html element.
 *
 * @param errorOutput The HTML element whose `innerHtml` property should be set to the error message
 */
export function displayError(errorOutput) {
  return function(response) {
    errorOutput.innerHTML = response.data.message;
    errorOutput.style.display = "block";
    throw new Error(response.data.message);
  };
}

/**
 * Makes a GET request to the given endpoint
 *
 * @param endpoint The endpoint to make the GET request to
 * @param headers The headers to
 *
 * @returns A promise that resolves to the server response along with server headers both on success and error.
 */
export function get(endpoint, headers = {}) {
  return mkReq("GET", endpoint, headers);
}

export function post(endpoint, headers = {}, data = {}) {
  return mkReq("POST", endpoint, headers, data);
}

export function del(endpoint, headers = {}) {
  return mkReq("DELETE", endpoint, headers);
}

export function patch(endpoint, headers, data) {
  return mkReq("PATCH", endpoint, headers, data);
}

const SEVERE_ERROR = {
  message:
    "Severe internal server error: The error response could not be processed. This is most likely due to an internal panic in the request handler and might require a restart! Please report this immediately!",
  code: 50000,
  data: null
};

const UNEXPECTED_REDIRECT = {
  message:
    "Unexpected redirect. This is a front-end error, most likely caused by a missing trailing slash",
  code: 50000,
  data: null
};

function mkReq(method, endpoint, headers = {}, data = null) {
  headers["Content-Type"] = "application/json";
  headers["Accept"] = "application/json";

  return new Promise(function(resolve, reject) {
    let xhr = new XMLHttpRequest();

    xhr.open(method, endpoint);
    xhr.onload = () => {
      if ((xhr.status >= 200 && xhr.status < 300) || xhr.status == 304) {
        resolve({
          data:
            xhr.status != 204 && xhr.status != 304
              ? JSON.parse(xhr.responseText)
              : null,
          headers: parseHeaders(xhr),
          status: xhr.status
        });
      } else if (xhr.status < 400) {
        reject({
          data: UNEXPECTED_REDIRECT,
          headers: parseHeaders(xhr),
          status: xhr.status
        });
      } else {
        try {
          var jsonError = JSON.parse(xhr.responseText);
        } catch (e) {
          return reject({
            data: SEVERE_ERROR,
            headers: parseHeaders(xhr),
            status: xhr.status
          });
        }
        reject({
          data: jsonError,
          headers: parseHeaders(xhr),
          status: xhr.status
        });
      }
    };

    for (let header of Object.keys(headers)) {
      xhr.setRequestHeader(header, headers[header]);
    }

    if (data) {
      data = JSON.stringify(data);
    }

    xhr.send(data);
  });
}

function parseHeaders(xhr) {
  return xhr
    .getAllResponseHeaders()
    .split("\r\n")
    .reduce((result, current) => {
      let [name, value] = current.split(": ");
      result[name] = value;
      return result;
    }, {});
}

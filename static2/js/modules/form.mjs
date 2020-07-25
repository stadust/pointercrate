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

  selectSilently(entry) {
    if (entry in this.values) {
      this.selected = entry;
      this.input.value = this.values[entry];
    }
  }

  addEventListener(listener) {
    this.listeners.push(listener);
  }
}

/**
 * Class representing complex HTML components that contain elements with the `.output` class, meaning we can display success and error messages in them somewhere.
 *
 * @export
 * @class Output
 */
export class Output {
  constructor(html) {
    this.html = html;

    this.errorOutput = this.html.getElementsByClassName("output")[0];
    this.successOutput = this.html.getElementsByClassName("output")[1];
  }

  setError(message, errorCode) {
    if (this.successOutput) this.successOutput.style.display = "none";

    if (this.errorOutput) {
      if (message === null || message === undefined) {
        this.errorOutput.style.display = "none";
      } else {
        this.errorOutput.innerHTML = message;
        this.errorOutput.style.display = "block";
      }
    }
  }

  setSuccess(message) {
    if (this.errorOutput) this.errorOutput.style.display = "none";

    if (this.successOutput) {
      if (message === null || message === undefined) {
        this.successOutput.style.display = "none";
      } else {
        this.successOutput.innerHTML = message;
        this.successOutput.style.display = "block";
      }
    }
  }
}

export class EditorBackend {
  url() {
    throw new Error("unimplemented");
  }

  headers() {
    throw new Error("unimplemented");
  }

  onSuccess(response) {}

  onError(response) {}

  edit(data, successCallback, errorCallback, unchangedCallback) {
    return patch(this.url(), this.headers(), data)
      .then((response) => {
        if (response.status == 304) {
          unchangedCallback();
        } else {
          this.onSuccess(response);
          successCallback(response);
        }
      })
      .catch((response) => {
        this.onError(response);
        errorCallback(response);
      });
  }
}

export class PaginatorEditorBackend extends EditorBackend {
  constructor(paginator, csrf, shouldRefresh) {
    super();

    this._paginator = paginator;
    this._csrf = csrf;
    this._shouldRefresh = shouldRefresh;
  }

  headers() {
    return {
      "X-CSRF-TOKEN": this._csrf,
      "If-Match": this._paginator.currentEtag,
    };
  }

  url() {
    return (
      this._paginator.retrievalEndpoint +
      this._paginator.currentlySelected.dataset.id +
      "/"
    );
  }

  onSuccess(response) {
    this._paginator.onReceive(response);
    if (this._shouldRefresh) {
      this._paginator.refresh();
    }
  }
}

export function setupDropdownEditor(
  backend,
  dropdownId,
  field,
  output,
  translationTable = {}
) {
  let dropdown = new Dropdown(document.getElementById(dropdownId));

  dropdown.addEventListener((selected) => {
    let data = {};
    if (translationTable.hasOwnProperty(selected)) {
      data[field] = translationTable[selected];
    } else {
      data[field] = selected;
    }

    backend.edit(
      data,
      () => output.setSuccess("Edit successful!"),
      (response) => displayError(output)(response),
      () => output.setSuccess("Nothing changed!")
    );
  });

  return dropdown;
}

export function setupDialogEditor(backend, dialogId, buttonId, output) {
  let dialog = document.getElementById(dialogId);
  let button = document.getElementById(buttonId);

  button.addEventListener("click", () => $(dialog.parentNode).fadeIn(300));

  return function (data) {
    backend.edit(
      data,
      () => {
        output.setSuccess("Edit successful!");
        $(dialog.parentNode).fadeOut(300);
      },
      (response) => {
        displayError(output)(response);
        $(dialog.parentNode).fadeOut(300);
      },
      () => {
        output.setSuccess("Nothing changed");
        $(dialog.parentNode).fadeOut(300);
      }
    );
  };
}

export function setupFormDialogEditor(backend, dialogId, buttonId, output) {
  let dialog = document.getElementById(dialogId);
  let button = document.getElementById(buttonId);

  button.addEventListener("click", () => $(dialog.parentNode).fadeIn(300));

  let form = new Form(dialog.getElementsByTagName("form")[0]);
  form.onSubmit(() => {
    backend.edit(
      form.serialize(),
      () => {
        output.setSuccess("Edit successful!");
        $(dialog.parentNode).fadeOut(300);
      },
      (response) => {
        displayError(form)(response);
      },
      () => {
        output.setSuccess("Nothing changed");
        $(dialog.parentNode).fadeOut(300);
      }
    );
  });

  return form;
}

export class Paginator extends Output {
  /**
   * Creates an instance of Paginator. Retrieves its endpoint from the `data-endpoint` data attribute of `html`.
   *
   * @param {String} elementId The Id of the DOM element of this paginator
   * @param {Object} queryData The initial query data to use
   * @param {*} itemConstructor Callback used to construct the list items of this Paginator
   * @memberof Paginator
   */
  constructor(elementId, queryData, itemConstructor) {
    super(document.getElementById(elementId));

    // Next and previous buttons
    this.next = this.html.getElementsByClassName("next")[0];
    this.prev = this.html.getElementsByClassName("prev")[0];

    // The li that was last clicked and thus counts as "selected"
    this.currentlySelected = null;
    // The 'data' part of the response that the server sent after clicking 'currentlySelected'
    this.currentObject = null;
    // The etag of 'currentObject'
    this.currentEtag = null;

    // external selection listeners
    this.selectionListeners = [];

    // The endpoint which will be paginated. By storing this, we assume that the 'Links' header never redirects
    // us to a different endpoint (this is the case with the pointercrate API)
    this.endpoint = this.html.dataset.endpoint;
    // The endpoint from which the actual objects will be retrieved. By default equal to the pagination endpoint.
    this.retrievalEndpoint = this.endpoint;

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
    return get(this.retrievalEndpoint + id + "/").then(
      this.onReceive.bind(this)
    );
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
    this.selectArbitrary(selected.dataset.id).catch(displayError(this));
  }

  /**
   * Realizes a callback for when the request made in onSelect is successful
   *
   * @param {*} response
   * @memberof Paginator
   */
  onReceive(response) {
    // I dont know why we check this everywhere, and at this point I'm too afraid to ask. But the API shouldn't return a 204 on a GET.
    if (response.status != 204) {
      if (response.status == 200 || response.status == 201) {
        this.currentObject = response.data.data;
        this.currentEtag = response.headers["etag"];
      }

      for (let listener of this.selectionListeners) {
        listener(this.currentObject);
      }
    }
  }

  addSelectionListener(listener) {
    this.selectionListeners.push(listener);
  }

  /**
   * Initializes this Paginator by making the request using the query data specified in the constructor.
   *
   * Calling any other method on this before calling initialize is considered an error.
   * Calling this more than once has no additional effect.
   *
   * @memberof Paginator
   */
  initialize() {
    if (this.links === undefined) return this.refresh();
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
      item.addEventListener("click", (e) => this.onSelect(e.currentTarget));
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
    return get(this.currentLink)
      .then(this.handleResponse.bind(this))
      .catch(displayError(this));
  }

  onPreviousClick() {
    if (this.links.prev) {
      get(this.links.prev)
        .then(this.handleResponse.bind(this))
        .catch(displayError(this));
    }
  }

  onNextClick() {
    if (this.links.next) {
      get(this.links.next)
        .then(this.handleResponse.bind(this))
        .catch(displayError(this));
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

export function findParentWithClass(element, clz) {
  let parent = element;

  while (parent !== null && parent.classList !== null) {
    if (parent.classList.contains(clz)) {
      return parent;
    }
    parent = parent.parentNode;
  }
}

export class Viewer extends Output {
  constructor(elementId, paginator) {
    super(elementId);

    this.viewer = findParentWithClass(this.html, "viewer");
    this.paginator = paginator;

    this._welcome = this.viewer.getElementsByClassName("viewer-welcome")[0];
    this._content = this.viewer.getElementsByClassName("viewer-content")[0];

    this.paginator.addSelectionListener(() => {
      this.setError(null);
      this.setSuccess(null);

      $(this._welcome).fadeOut(100);
      $(this._content).fadeIn(100);
    });
  }
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
  constructor(
    paginatorID,
    itemConstructor,
    filterParam,
    initialQueryData = {}
  ) {
    super(paginatorID, initialQueryData, itemConstructor);

    let filterInput = this.html.getElementsByTagName("input")[0];

    filterInput.value = "";

    // Apply filter when enter is pressed
    filterInput.addEventListener("keypress", (event) => {
      if (event.keyCode == 13) {
        this.updateQueryData(filterParam, filterInput.value);
      }
    });

    // Apply filter when input is changed externally
    filterInput.addEventListener("change", () =>
      this.updateQueryData(filterParam, filterInput.value)
    );

    filterInput.parentNode.addEventListener("click", (event) => {
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
      message: msg,
    });
  }

  addValidators(validators) {
    Object.keys(validators).forEach((message) =>
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

  get required() {
    return this.input.hasAttribute("required");
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

export class Form extends Output {
  constructor(form) {
    super(form);

    this.inputs = [];
    this.submitHandler = undefined;
    this.invalidHandler = undefined;
    this.errorOutput = this.html.getElementsByClassName("output")[0];
    this.successOutput = this.html.getElementsByClassName("output")[1];
    this._errorRedirects = {};

    for (var input of this.html.getElementsByClassName("form-input")) {
      this.inputs.push(new Input(input));
    }

    this.html.addEventListener(
      "submit",
      (event) => {
        event.preventDefault();

        this.setError(null);
        this.setSuccess(null);

        var isValid = true;

        for (let input of this.inputs) {
          isValid &= input.validate(event);
        }

        if (isValid) {
          if (this.submitHandler !== undefined) {
            // todo: maybe just pass the result of .serialize here?
            this.submitHandler(event);
          }
        } else if (this.invalidHandler !== undefined) {
          this.invalidHandler();
        }
      },
      false
    );
  }

  clear() {
    for (let input of this.inputs) {
      input.value = "";
    }
  }

  /**
   * Adds an override to have errors with the given code be displayed as an error at the given input element instead of globally
   *
   * @param {int} errorCode The error code
   * @param {string} inputId The id of the input
   * @memberof Form
   */
  addErrorOverride(errorCode, inputId) {
    this._errorRedirects[errorCode] = inputId;
  }

  serialize() {
    let data = {};

    for (let input of this.inputs) {
      if (input.name !== null && (input.value !== null || !input.required)) {
        data[input.name] = input.value;
      }
    }

    return data;
  }

  setError(message, errorCode) {
    if (this.successOutput) this.successOutput.style.display = "none";

    if (this.errorOutput) {
      if (message === null || message === undefined) {
        this.errorOutput.style.display = "none";
      } else {
        if (errorCode in this._errorRedirects) {
          let input = this.input(this._errorRedirects[errorCode]);
          if (input) {
            input.setError(message);
          } else {
            this.errorOutput.style.display = "block";
            this.errorOutput.innerHTML = message;
          }
        } else {
          this.errorOutput.style.display = "block";
          this.errorOutput.innerHTML = message;
        }
      }
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
    Object.keys(validators).forEach((input_id) =>
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
 * @param specialCodes Special error handlers for specific error codes. Special handlers should be keyed by pointercrate error code and take the error object as only argument
 */
export function displayError(output, specialCodes = {}) {
  return function (response) {
    if (response.data) {
      if (response.data.code in specialCodes) {
        specialCodes[response.data.code](response.data);
      } else {
        output.setError(response.data.message, response.data.code);
      }
    } else {
      output.setError(
        "FrontEnd JavaScript Error. Please notify an administrator and tell them as accurately as possible how to replicate this bug!"
      );
      throw new Error("FrontendError");
    }
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
  data: null,
};

const UNEXPECTED_REDIRECT = {
  message:
    "Unexpected redirect. This is a front-end error, most likely caused by a missing trailing slash",
  code: 50000,
  data: null,
};

function mkReq(method, endpoint, headers = {}, data = null) {
  headers["Content-Type"] = "application/json";
  headers["Accept"] = "application/json";

  return new Promise(function (resolve, reject) {
    let xhr = new XMLHttpRequest();

    xhr.open(method, endpoint);
    xhr.onload = () => {
      if ((xhr.status >= 200 && xhr.status < 300) || xhr.status == 304) {
        resolve({
          data:
            xhr.status != 204 && xhr.status != 304 && xhr.responseText // sometimes 201 responses dont have any json body
              ? JSON.parse(xhr.responseText)
              : null,
          headers: parseHeaders(xhr),
          status: xhr.status,
        });
      } else if (xhr.status < 400) {
        reject({
          data: UNEXPECTED_REDIRECT,
          headers: parseHeaders(xhr),
          status: xhr.status,
        });
      } else {
        try {
          var jsonError = JSON.parse(xhr.responseText);
        } catch (e) {
          return reject({
            data: SEVERE_ERROR,
            headers: parseHeaders(xhr),
            status: xhr.status,
          });
        }
        reject({
          data: jsonError,
          headers: parseHeaders(xhr),
          status: xhr.status,
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

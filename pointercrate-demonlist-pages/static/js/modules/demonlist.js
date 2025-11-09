import {
  Dropdown,
  Form,
  post,
  valueMissing,
  typeMismatch,
  badInput,
  stepMismatch,
  rangeUnderflow,
  rangeOverflow,
  tooLong,
  findParentWithClass,
  FilteredPaginator,
  Viewer,
  setupFormDialogEditor,
  FormDialog,
  setupEditorDialog,
  get,
} from "/static/core/js/modules/form.js";
import { tr, trp } from "/static/core/js/modules/localization.js";

export function embedVideo(video) {
  if (!video) return;
  // welcome to incredibly fragile string parsing with stadust
  // see pointercrate::video::embed for a proper implementation of this

  if (video.startsWith("https://www.youtube")) {
    return "https://www.youtube.com/embed/" + video.substring(32);
  }

  if (video.startsWith("https://www.twitch")) {
    return (
      "https://player.twitch.tv/?autoplay=false&parent=pointercrate.com&video=" +
      video.substring(29)
    );
  }
}

export function initializeTimeMachine() {
  let formHtml = document.getElementById("time-machine-form");

  if (formHtml === null) return;

  var timeMachineForm = new Form(formHtml);
  var destination = timeMachineForm.input("time-machine-destination");

  destination.addValidator(
    valueMissing,
    tr(
      "demonlist",
      "overview",
      "time-machine.destination-validator-valuemissing"
    )
  );
  destination.addValidator(
    rangeUnderflow,
    tr(
      "demonlist",
      "overview",
      "time-machine.destination-validator-rangeunderflow"
    )
  );

  var now = new Date();
  var year = now.getFullYear();
  var month = String(now.getMonth() + 1).padStart(2, "0");
  var day = String(now.getDate()).padStart(2, "0");
  var hours = String(now.getHours()).padStart(2, "0");
  var minutes = String(now.getMinutes()).padStart(2, "0");

  destination.value = `${year}-${month}-${day}T${hours}:${minutes}`;

  var offset = new Date().getTimezoneOffset();
  var offsetHours = Math.trunc(offset / 60); // round towards zero to ensure things like GMT-2.5 work
  var offsetMinutes = Math.abs(offset) % 60;

  timeMachineForm.onSubmit(() => {
    // datetime-local gives us a string in the format YYYY-MM-DDThh:mm. Thus, pad it with :ss and timezone information, as the backend expects (aka a rfc3339 date)
    let when =
      destination.value +
      ":00" +
      (offsetHours < 0 ? "%2B" : "-") +
      (Math.abs(offsetHours) + "").padStart(2, "0") +
      ":" +
      (offsetMinutes + "").padStart(2, "0");

    document.cookie = "when=" + when;

    window.location = "/demonlist/";
  });
}

export function initializeRecordSubmitter(submitApproved = false) {
  var submissionForm = new Form(document.getElementById("submission-form"));

  var demon = submissionForm.input("id_demon");
  var player = submissionForm.input("id_player");
  var progress = submissionForm.input("id_progress");
  var video = submissionForm.input("id_video");
  var rawFootage = submissionForm.input("submit-raw-footage");

  demon.addValidator(
    (input) => input.dropdown.selected !== undefined,
    tr(
      "demonlist",
      "submitter",
      "record-submission.demon-validator-valuemissing"
    )
  );
  demon.setTransform(parseInt);

  player.addValidator(
    (input) => input.value !== undefined,
    tr(
      "demonlist",
      "submitter",
      "record-submission.holder-validator-valuemissing"
    )
  );
  player.addValidator(
    (input) => input.value === undefined || input.value.length <= 50,
    tr(
      "demonlist",
      "submitter",
      "record-submission.holder-validator-rangeoverflow"
    )
  );

  progress.addValidator(
    valueMissing,
    tr(
      "demonlist",
      "submitter",
      "record-submission.progress-validator-valuemissing"
    )
  );
  progress.addValidator(
    rangeUnderflow,
    tr(
      "demonlist",
      "submitter",
      "record-submission.progress-validator-rangeunderflow"
    )
  );
  progress.addValidator(
    rangeOverflow,
    tr(
      "demonlist",
      "submitter",
      "record-submission.progress-validator-rangeoverflow"
    )
  );
  progress.addValidator(
    badInput,
    tr(
      "demonlist",
      "submitter",
      "record-submission.progress-validator-badinput"
    )
  );
  progress.addValidator(
    stepMismatch,
    tr(
      "demonlist",
      "submitter",
      "record-submission.progress-validator-stepmismatch"
    )
  );

  video.addValidator(
    valueMissing,
    tr(
      "demonlist",
      "submitter",
      "record-submission.video-validator-valuemissing"
    )
  );
  video.addValidator(
    typeMismatch,
    tr(
      "demonlist",
      "submitter",
      "record-submission.video-validator-typemismatch"
    )
  );

  rawFootage.addValidator(
    typeMismatch,
    tr(
      "demonlist",
      "submitter",
      "record-submission.raw-footage-validator-typemismatch"
    )
  );

  submissionForm.onSubmit(function () {
    let data = submissionForm.serialize();
    let headers = {};

    if (submitApproved) {
      data.status = "approved";
    }
    post("/api/v1/records/", headers, data)
      .then((response) => {
        let queue_position = response.headers["x-submission-count"];

        if (queue_position)
          submissionForm.setSuccess(
            trp(
              "demonlist",
              "submitter",
              "record-submission.submission-success.queue",
              {
                ["queue-position"]: queue_position,
              }
            )
          );
        else
          submissionForm.setSuccess(
            tr("demonlist", "submitter", "record-submission.submission-success")
          );
        submissionForm.clear();
      })
      .catch((response) => {
        switch (response.data.code) {
          case 40401:
            demon.errorText = response.data.message;
            break;
          case 42218:
            player.errorText = response.data.message;
            break;
          case 42215:
          case 42220:
            progress.errorText = response.data.message;
            break;
          case 42222:
          case 42223:
          case 42224:
          case 42225:
            video.errorText = response.data.message;
            break;
          case 42232:
          case 42233:
            rawFootage.errorText = response.data.message;
            break;
          default:
            submissionForm.setError(response.data.message);
        }
      }); // TODO: maybe specially handle some error codes
  });
}

export function getCountryFlag(title, countryCode) {
  let countrySpan = document.createElement("span");
  countrySpan.classList.add("flag-icon");
  countrySpan.title = title;
  countrySpan.style.backgroundImage =
    "url(/static/demonlist/images/flags/" + countryCode.toLowerCase() + ".svg";
  return countrySpan;
}

export function getSubdivisionFlag(title, countryCode, subdivisionCode) {
  let stateSpan = document.createElement("span");
  stateSpan.classList.add("flag-icon");
  stateSpan.title = title;
  stateSpan.style.backgroundImage =
    "url(/static/demonlist/images/flags/" +
    countryCode.toLowerCase() +
    "/" +
    subdivisionCode.toLowerCase() +
    ".svg";
  return stateSpan;
}

export function populateSubdivisionDropdown(dropdown, countryCode) {
  dropdown.clearOptions();

  return get("/api/v1/nationalities/" + countryCode + "/subdivisions/").then(
    (result) => {
      for (let subdivision of result.data) {
        let flag = getSubdivisionFlag(
          subdivision.name,
          countryCode,
          subdivision.iso_code
        );

        flag.style.marginLeft = "-10px";
        flag.style.paddingRight = "1em";

        let li = document.createElement("li");

        li.className = "white hover";
        li.dataset.value = subdivision.iso_code;
        li.dataset.display = subdivision.name;
        li.appendChild(flag);
        li.appendChild(document.createTextNode(subdivision.name));

        dropdown.addListItem(li);
      }
    }
  );
}

export function generatePlayer(player) {
  var li = document.createElement("li");
  var b = document.createElement("b");
  var b2 = document.createElement("b");

  if (player.banned) {
    li.classList.add("err");
  } else {
    li.classList.add("ok");
  }

  li.dataset.name = player.name;
  li.dataset.id = player.id;

  b2.appendChild(document.createTextNode(player.id));

  if (player.nationality) {
    var span = document.createElement("span");

    span.className =
      "flag-icon flag-icon-" + player.nationality.country_code.toLowerCase();

    li.appendChild(span);
    li.appendChild(document.createTextNode(" "));
  }

  li.appendChild(b);
  li.appendChild(document.createTextNode(player.name + " - "));
  li.appendChild(b2);

  return li;
}

export function generateDemon(demon) {
  let li = document.createElement("li");
  let b = document.createElement("b");

  li.dataset.id = demon.id;

  b.innerText = "#" + demon.position + " - ";

  li.appendChild(b);
  li.appendChild(
    document.createTextNode(
      trp("demonlist", "demon", "demon-listed", {
        ["demon"]: demon.name,
        ["demon-id"]: demon.id.toString(),
      })
    )
  );
  li.appendChild(document.createElement("br"));
  li.appendChild(
    document.createTextNode(
      trp("demonlist", "demon", "demon-listed.publisher", {
        ["publisher"]: demon.publisher.name,
      })
    )
  );

  return li;
}

export function generateRecord(record) {
  var li = document.createElement("li");
  var recordId = document.createElement("b");

  li.dataset.id = record.id;

  switch (record.status) {
    case "approved":
      li.classList.add("ok");
      break;
    case "rejected":
      li.classList.add("err");
      break;
    case "submitted":
      li.classList.add("warn");
      break;
    case "under consideration":
      li.classList.add("consider");
      break;
    default:
      break;
  }

  recordId.appendChild(
    document.createTextNode(
      trp("demonlist", "record", "record-listed", {
        ["record-id"]: record.id.toString(),
      })
    )
  );

  li.appendChild(recordId);
  li.appendChild(document.createElement("br"));
  li.appendChild(
    document.createTextNode(record.player.name + " (" + record.player.id + ")")
  );
  li.appendChild(document.createElement("br"));
  li.appendChild(
    document.createTextNode(
      trp("demonlist", "record", "record-listed.progress", {
        ["percent"]: record.progress,
        ["demon"]: record.demon.name,
      })
    )
  );
  li.appendChild(document.createElement("br"));

  return li;
}

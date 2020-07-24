import { FilteredViewer, get, patch } from "../modules/form.mjs";
import { generateDemon, embedVideo } from "../modules/demonlist.mjs";

export let demonManager;

export class DemonManager extends FilteredViewer {
  constructor(csrfToken) {
    super("demon-pagination", generateDemon, "name_contains");

    this.currentDemon = null;
    this.currentDemonEtag = null;

    this._id = document.getElementById("demon-demon-id");
    this._name = document.getElementById("demon-demon-name");

    this._video = document.getElementById("demon-video");
    this._video_link = document.getElementById("demon-video-link");

    this._position = document.getElementById("demon-position");
    this._requirement = document.getElementById("demon-requirement");

    this._verifier = document.getElementById("demon-verifier");
    this._publisher = document.getElementById("demon-publisher");

    this._creators = document.getElementById("demon-creators");
  }

  selectArbitrary(id) {
    // pagination endpoint different from data retrieval endpoint!
    return get("/api/v2/demons/" + id + "/").then(this.onReceive.bind(this));
  }

  onReceive(response) {
    super.onReceive(response);

    if (response.status == 204) {
      return;
    }

    this.currentDemon = response.data.data;
    this.currentDemonEtag = response.headers["etag"];

    this._id.innerText = this.currentDemon.id;
    this._name.innerText = this.currentDemon.name;
    this._position.innerText = this.currentDemon.position;
    this._requirement.innerText = this.currentDemon.requirement;

    var embeddedVideo = embedVideo(this.currentDemon.video);

    if (embeddedVideo !== undefined) {
      this._video.style.display = "block";
      this._video_link.style.display = "initial";
      this._video.src = embeddedVideo;
      this._video_link.href = this.currentDemon.video;
      this._video_link.innerHTML = this.currentDemon.video;
    } else {
      this._video.style.display = "none";
      this._video_link.style.display = "none";
    }

    this._publisher.innerHTML =
      this.currentDemon.publisher.name +
      " (" +
      this.currentDemon.publisher.id +
      ")";
    this._verifier.innerHTML =
      this.currentDemon.verifier.name +
      " (" +
      this.currentDemon.verifier.id +
      ")";
  }
}

export function initialize(csrfToken) {
  demonManager = new DemonManager(csrfToken);
  demonManager.initialize();
}

class Submitter {
  constructor() {
    this.domElement = $("#submitter");
    this.form = this.domElement.find("#submission-form");
    this._output = this.domElement.find("#submission-output");

    this._demon = this.domElement.find("#id_demon");
    this._player = this.domElement.find("#id_player");
    this._video = this.domElement.find("#id_video");
    this._progress = this.domElement.find("#id_progress");
    this._check = this.domElement.find("#id_check");
  }

  show() {
    this.form[0].reset();
    this._output.hide();

    Dialog.showById("submission-dialog");
  }

  submit() {
    this._output.slideUp(100);

    $.ajax({
      method: "POST",
      url: "/api/v1/records/",
      contentType: "application/json",
      dataType: "json",
      data: JSON.stringify({
        demon: this.demon,
        player: this.player,
        video: this.video,
        progress: this.progress,
        check: this.check
      }),
      statusCode: {
        204: () => (this.output = "This record can be submitted!"),
        429: () =>
          (this.output =
            "You are submitting too many records too fast! Try again later")
      },
      error: data => (this.output = data.responseJSON.message),
      success: () => (this.output = "Record successfully submitted")
    });

    return false;
  }

  get demon() {
    return this._demon.val();
  }

  get player() {
    return this._player.val();
  }

  get video() {
    return this._video.val();
  }

  get progress() {
    return parseInt(this._progress.val());
  }

  get check() {
    return this._check[0].checked;
  }

  set output(data) {
    this._output.text(data);
    this._output.slideDown(100);
  }
}

class StatsViewer {
  constructor() {
    this.domElement = $("#statsviewer");
    this._created = this.domElement.find("#created");
    this._beaten = this.domElement.find("#beaten");
    this._verified = this.domElement.find("#verified");
    this._published = this.domElement.find("#published");
    this._hardest = this.domElement.find("#hardest");
    this._score = this.domElement.find("#score");
    this._rank = this.domElement.find("#rank");
    this._amountBeaten = this.domElement.find("#amount-beaten");
    this._amountLegacy = this.domElement.find("#amount-legacy");
    this._current = this.domElement.find("#name");
    this._error = this.domElement.find("#error-output");
    this._progress = this.domElement.find("#progress");
    this._content = this.domElement.find("#stats-data");

    $("#players li").click(e => this.updateView(e));
  }

  updateView(event) {
    let selected = $(event.currentTarget);

    $.ajax({
      method: "GET",
      url: "/api/v1/players/" + selected.data("id") + "/",
      dataType: "json",
      error: data => {
        this._content.hide(100);

        if (data.responseJSON) this._error.text(data.responseJSON.message);
        else this._error.text("Something went wrong!");

        this._error.show(100);
      },
      success: data => {
        let json = data.data;

        this._current.text(selected.find(".player-name").text());
        this._rank.text(selected.data("rank"));
        this._score.text(selected.find("i").text());

        this.setFields(
          json.created,
          json.published,
          json.verified,
          json.records
        );

        this._error.hide(100);
        this._content.show(100);
      }
    });
  }

  setFields(created, published, verified, records) {
    this._created.html(formatDemons(created) || "None");
    this._published.html(formatDemons(published) || "None");
    this._verified.html(formatDemons(verified) || "None");

    let beaten = records
      .filter(record => record.progress == 100)
      .map(record => record.demon);

    let legacy = beaten.filter(
      demon => demon.position > window.extended_list_length
    ).length;
    let extended = beaten.filter(
      demon =>
        demon.position > window.list_length &&
        demon.position <= window.extended_list_length
    ).length;

    this._beaten.html(formatDemons(beaten) || "None");
    this._amountBeaten.text(
      beaten.length - legacy - extended + " ( + " + extended + " )"
    );
    this._amountLegacy.text(legacy);

    var hardest = verified
      .concat(beaten)
      .reduce((acc, next) => (acc.position > next.position ? next : acc));

    this._hardest.text(hardest.name || "None");

    var non100Records = records
      .filter(record => record.progress != 100)
      .sort((r1, r2) => r1.progress - r2.progress)
      .map(record => formatDemon(record.demon) + " (" + record.progress + "%)")
      .join(", ");

    this._progress.html(non100Records || "None");
  }
}

$(document).ready(function() {
  window.statsViewer = new StatsViewer();
  window.submitter = new Submitter();
});

function formatDemon(demon) {
  if (demon.position < window.list_length) {
    return "<b>" + demon.name + "</b>";
  } else if (demon.position < window.extended_list_length) {
    return demon.name;
  } else {
    return "<i>" + demon.name + "</i>";
  }
}

function formatDemons(demons) {
  return demons.map(formatDemon).join(", ");
}

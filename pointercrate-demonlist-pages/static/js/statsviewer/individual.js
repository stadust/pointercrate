import { tr } from "/static/core/js/modules/localization.js";
import { displayError, Dropdown, get } from "/static/core/js/modules/form.js";
import {
  getCountryFlag,
  populateSubdivisionDropdown,
} from "/static/demonlist/js/modules/demonlist.js";
import {
  formatInto,
  InteractiveWorldMap,
  StatsViewer,
} from "/static/demonlist/js/modules/statsviewer.js";

class IndividualStatsViewer extends StatsViewer {
  constructor(html) {
    super(html, {
      retrievalEndpoint: "/api/v1/players/",
      rankingEndpoint: "/api/v1/players/ranking/",
      entryGenerator: generateStatsViewerPlayer,
    });
  }

  onReceive(response) {
    super.onReceive(response);

    var playerData = response.data.data;

    const rankKey = window.active_list == "demonlist" ? "rated_rank" : "rank";
    const scoreKey = window.active_list == "demonlist" ? "rated_score" : "score";
    // this doesn't need to be used in sorting operations because position and rated_position will always be in the same order (and position is non-nullable)
    const demonPositionKey = window.active_list == "demonlist" ? "rated_position" : "position";

    this._rank.innerText = playerData[rankKey] || "-";
    this._score.innerText = playerData[scoreKey].toFixed(2);

    this.setName(playerData.name, playerData.nationality);

    const selectedSort = this.demonSortingModeDropdown.selected;

    let created = playerData.created.filter((demon) => demon[demonPositionKey] != null);
    let published = playerData.published.filter((demon) => demon[demonPositionKey] != null);
    let verified = playerData.verified.filter((demon) => demon[demonPositionKey] != null);

    this.formatDemonsInto(
      this._created,
      this.sortStatsViewerRow(selectedSort, created)
    );
    this.formatDemonsInto(
      this._published,
      this.sortStatsViewerRow(selectedSort, published)
    );
    this.formatDemonsInto(
      this._verified,
      this.sortStatsViewerRow(selectedSort, verified)
    );

    let beaten = playerData.records.filter((record) => record.progress === 100 && record.demon[demonPositionKey] != null);

    beaten.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));
    this.formatRecordsInto(this._beaten, beaten);

    beaten.sort((r1, r2) => r1.demon.position - r2.demon.position);

    let legacy = beaten.filter(
      (record) => record.demon[demonPositionKey] > this.extended_list_size
    );
    let extended = beaten.filter(
      (record) =>
        record.demon[demonPositionKey] > this.list_size &&
        record.demon[demonPositionKey] <= this.extended_list_size
    );
    let main = beaten.filter(
      (record) => record.demon[demonPositionKey] <= this.list_size
    );

    this.formatRecordsInto(this._main_beaten, main, true);
    this.formatRecordsInto(this._extended_beaten, extended, true);
    this.formatRecordsInto(this._legacy_beaten, legacy, true);

    let verifiedExtended = verified.filter(
      (demon) =>
        demon[demonPositionKey] <= this.extended_list_size &&
        demon[demonPositionKey] > this.list_size
    ).length;
    let verifiedLegacy = verified.filter(
      (demon) =>  demon[demonPositionKey] > this.extended_list_size
    ).length;

    this.setCompletionNumber(
      main.length +
        verified.length -
        verifiedExtended -
        verifiedLegacy,
      extended.length + verifiedExtended,
      legacy.length + verifiedLegacy
    );

    let hardest = verified
      .concat(beaten.map((record) => record.demon))
      .reduce((acc, next) => (acc.position > next.position ? next : acc), {
        name: tr("demonlist", "statsviewer", "statsviewer.value-none"),
        position: 321321321,
      });

    this.setHardest(
      hardest.name === tr("demonlist", "statsviewer", "statsviewer.value-none")
        ? undefined
        : hardest
    );

    let non100Records = playerData.records.filter(
      (record) => record.progress !== 100 && record.demon[demonPositionKey] != null
    );

    this.formatRecordsInto(
      this._progress,
      this.sortStatsViewerRow(selectedSort, non100Records)
    );

    this.demonSortingModeDropdown.addEventListener((selected) => {
      this.formatDemonsInto(
        this._created,
        this.sortStatsViewerRow(selected, created)
      );
      this.formatDemonsInto(
        this._published,
        this.sortStatsViewerRow(selected, published)
      );
      this.formatDemonsInto(
        this._verified,
        this.sortStatsViewerRow(selected, verified)
      );
      this.formatRecordsInto(
        this._progress,
        this.sortStatsViewerRow(selected, non100Records)
      );
    });
  }

  formatDemonsInto(element, demons) {
    formatInto(
      element,
      demons.map((demon) => this.formatDemon(demon))
    );
  }

  formatRecordsInto(element, records, dontStyle) {
    formatInto(
      element,
      records.map((record) => {
        let demon = this.formatDemon(record.demon, record.video, dontStyle);
        if (record.progress !== 100) {
          demon.appendChild(
            document.createTextNode(" (" + record.progress + "%)")
          );
        }
        return demon;
      })
    );
  }
  onSelect(selected) {
    let params = new URLSearchParams(window.location.href.split("?")[1]);
    params.set("player", selected.dataset.id);
    const urlWithoutParam = `${window.location.origin}${
      window.location.pathname
    }?${params.toString()}`;
    window.history.replaceState({}, "", urlWithoutParam);
    super.onSelect(selected);
  }
}

$(window).on("load", function () {
  window.map = new InteractiveWorldMap();
  map.showSubdivisions();

  let subdivisionCheckbox = document.getElementById(
    "show-subdivisions-checkbox"
  );
  subdivisionCheckbox.addEventListener("change", () => {
    if (subdivisionCheckbox.checked) map.showSubdivisions();
    else map.hideSubdivisions();
  });

  window.statsViewer = new IndividualStatsViewer(
    document.getElementById("statsviewer")
  );

  window.statsViewer.initialize().then(() => {
    let url = window.location.href;
    let params = new URLSearchParams(url.split("?")[1]);
    let playerId = parseInt(params.get("player"));
    if (playerId !== undefined && !isNaN(playerId)) {
      window.statsViewer.selectArbitrary(playerId).catch((err) => {
        displayError(window.statsViewer)(err);

        // if the param failed, set the URL bar's value to the same location, but with the
        // "player" parameter removed
        params.delete("player");
        const urlWithoutParam = `${window.location.origin}${
          window.location.pathname
        }?${params.toString()}`;
        window.history.replaceState({}, "", urlWithoutParam);
      });
    }
  });

  window.subdivisionDropdown = new Dropdown(
    document.getElementById("subdivision-dropdown")
  );

  subdivisionDropdown.addEventListener((selected) => {
    if (selected === "None") {
      map.deselectSubdivision();
      statsViewer.updateQueryData("subdivision", undefined);
    } else {
      let countryCode = statsViewer.queryData["nation"];

      map.select(countryCode, selected);
      statsViewer.updateQueryData2({
        nation: countryCode,
        subdivision: selected,
      });
    }
  });

  new Dropdown(document.getElementById("continent-dropdown")).addEventListener(
    (selected) => {
      if (selected === "All") {
        window.statsViewer.updateQueryData("continent", undefined);
        map.resetContinentHighlight();
      } else {
        window.statsViewer.updateQueryData("continent", selected);
        map.highlightContinent(selected);
      }
    }
  );

  statsViewer.dropdown.addEventListener((selected) => {
    if (selected === "International") {
      map.deselect();
    } else {
      map.select(selected);
    }

    // if 'countryCode == International' we send a nonsense request which results in a 404 and causes the dropdown to clear. That's exactly what we want, though.
    populateSubdivisionDropdown(subdivisionDropdown, selected);

    statsViewer.updateQueryData("subdivision", undefined);
  });

  map.addSelectionListener((countryCode, subdivisionCode) => {
    populateSubdivisionDropdown(subdivisionDropdown, countryCode).then(() =>
      subdivisionDropdown.selectSilently(subdivisionCode)
    );

    statsViewer.dropdown.selectSilently(countryCode);

    statsViewer.updateQueryData2({
      nation: countryCode,
      subdivision: subdivisionCode,
    });
  });

  map.addDeselectionListener(() => {
    statsViewer.dropdown.selectSilently("International");
    subdivisionDropdown.clearOptions();
    statsViewer.updateQueryData2({
      nation: undefined,
      subdivision: undefined,
    });
  });
});

function generateStatsViewerPlayer(player) {
  var li = document.createElement("li");
  var b = document.createElement("b");
  var i = document.createElement("i");

  li.className = "white hover";
  li.dataset.id = player.id;

  b.appendChild(document.createTextNode("#" + player.rank + " "));
  i.appendChild(document.createTextNode(player.score.toFixed(2)));

  if (player.nationality) {
    li.appendChild(
      getCountryFlag(player.nationality.nation, player.nationality.country_code)
    );
    li.appendChild(document.createTextNode(" "));
  }

  li.appendChild(b);
  li.appendChild(document.createTextNode(player.name));
  li.appendChild(i);

  return li;
}

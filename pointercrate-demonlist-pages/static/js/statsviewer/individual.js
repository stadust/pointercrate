import { Dropdown } from "/static/core/js/modules/form.js";
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

    this._rank.innerText = playerData.rank;
    this._score.innerText = playerData.score.toFixed(2);

    this.setName(playerData.name, playerData.nationality);

    const selectedSort = this.demonSortingModeDropdown.selected;

    this.formatDemonsInto(
      this._created,
      this.sortStatsViewerRow(selectedSort, playerData.created)
    );
    this.formatDemonsInto(
      this._published,
      this.sortStatsViewerRow(selectedSort, playerData.published)
    );
    this.formatDemonsInto(
      this._verified,
      this.sortStatsViewerRow(selectedSort, playerData.verified)
    );

    let beaten = playerData.records.filter((record) => record.progress === 100);

    beaten.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));
    this.formatRecordsInto(this._beaten, beaten);

    beaten.sort((r1, r2) => r1.demon.position - r2.demon.position);

    let legacy = beaten.filter(
      (record) => record.demon.position > this.extended_list_size
    );
    let extended = beaten.filter(
      (record) =>
        record.demon.position > this.list_size &&
        record.demon.position <= this.extended_list_size
    );
    let main = beaten.filter(
      (record) => record.demon.position <= this.list_size
    );

    this.formatRecordsInto(this._main_beaten, main, true);
    this.formatRecordsInto(this._extended_beaten, extended, true);
    this.formatRecordsInto(this._legacy_beaten, legacy, true);

    let verifiedExtended = playerData.verified.filter(
      (demon) =>
        demon.position <= this.extended_list_size &&
        demon.position > this.list_size
    ).length;
    let verifiedLegacy = playerData.verified.filter(
      (demon) => demon.position > this.extended_list_size
    ).length;

    this.setCompletionNumber(
      main.length +
        playerData.verified.length -
        verifiedExtended -
        verifiedLegacy,
      extended.length + verifiedExtended,
      legacy.length + verifiedLegacy
    );

    let hardest = playerData.verified
      .concat(beaten.map((record) => record.demon))
      .reduce((acc, next) => (acc.position > next.position ? next : acc), {
        name: "None",
        position: 321321321321,
      });

    this.setHardest(hardest.name === "None" ? undefined : hardest);

    let non100Records = playerData.records.filter(
      (record) => record.progress !== 100
    );

    this.formatRecordsInto(
      this._progress,
      this.sortStatsViewerRow(selectedSort, non100Records)
    );

    this.demonSortingModeDropdown.addEventListener((selected) => {
      this.formatDemonsInto(
        this._created,
        this.sortStatsViewerRow(selected, playerData.created)
      );
      this.formatDemonsInto(
        this._published,
        this.sortStatsViewerRow(selected, playerData.published)
      );
      this.formatDemonsInto(
        this._verified,
        this.sortStatsViewerRow(selected, playerData.verified)
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
}

$(window).on("load", function () {
  let map = new InteractiveWorldMap();
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

  window.statsViewer.initialize();

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

  let subdivisionDropdown = new Dropdown(
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
    statsViewer.updateQueryData2({ nation: undefined, subdivision: undefined });
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

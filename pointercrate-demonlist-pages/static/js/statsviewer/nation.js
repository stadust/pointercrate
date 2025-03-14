import {
  formatInto,
  InteractiveWorldMap,
  StatsViewer,
} from "/static/demonlist/js/modules/statsviewer.js";
import { Dropdown } from "/static/core/js/modules/form.js";
import { getCountryFlag } from "/static/demonlist/js/modules/demonlist.js";

class NationStatsViewer extends StatsViewer {
  constructor(html) {
    super(html, {
      retrievalEndpoint: "/api/v1/nationalities/",
      rankingEndpoint: "/api/v1/nationalities/ranking/",
      entryGenerator: generateStatsViewerNation,
    });

    this._players = document.getElementById("players");
    this._unbeaten = document.getElementById("unbeaten");
  }

  onReceive(response) {
    super.onReceive(response);

    let nationData = response.data.data;

    this.setName(nationData.nation, nationData);

    let beaten = [];
    let progress = [];

    let legacy = 0;
    let extended = 0;

    let hardest = undefined;

    let players = new Set();

    for (let record of nationData.records) {
      record.players.forEach(players.add, players);

      if (record.progress !== 100) {
        if (!nationData.verified.some((d) => d.id === record.id))
          progress.push(record);
      } else {
        beaten.push(record);

        if (hardest === undefined || record.demon.position < hardest.position) {
          hardest = record.demon;
        }

        if (record.demon.position > this.list_size)
          if (record.demon.position <= this.extended_list_size) ++extended;
          else ++legacy;
      }
    }

    let amountBeaten = beaten.length - extended - legacy;

    for (let record of nationData.verified) {
      record.players.forEach(players.add, players);

      if (hardest === undefined || record.demon.position < hardest.position) {
        hardest = record.demon
      }

      if (!beaten.some((d) => d.demon.id === record.demon.id))
        if (record.demon.position > this.list_size)
          if (record.demon.position <= this.extended_list_size) ++extended;
          else ++legacy;
        else ++amountBeaten;
    }

    this._players.innerText = players.size.toString();

    this.setHardest(hardest);
    this.setCompletionNumber(amountBeaten, extended, legacy);

    nationData.unbeaten.sort((r1, r2) => r1.name.localeCompare(r2.name));
    beaten.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));
    progress.sort((r1, r2) => r2.progress - r1.progress);
    nationData.created.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));

    formatInto(
      this._unbeaten,
      nationData.unbeaten.map((demon) =>
        this.formatDemon(demon)
      )
    );
    formatInto(
      this._beaten,
      beaten.map((record) => this.formatDemonFromRecord(record))
    );
    formatInto(
      this._progress,
      progress.map((record) => this.formatDemonFromRecord(record))
    );
    formatInto(
      this._created,
      nationData.created.map((creation) => {
        return this.makeTooltip(
          this.formatDemon(creation.demon),
          "(Co)created&nbsp;by&nbsp;" +
            creation.players.length +
            "&nbsp;player" +
            (creation.players.length === 1 ? "" : "s") +
            "&nbsp;in&nbsp;this&nbsp;country: ",
          creation.players.join(", ")
        );
      })
    );
    formatInto(
      this._verified,
      nationData.verified.map((verification) => {
        return this.makeTooltip(
          this.formatDemon(verification.demon),
          "Verified&nbsp;by: ",
          verification.players.join(", ")
        );
      })
    );
    formatInto(
      this._published,
      nationData.published.map((publication) => {
        return this.makeTooltip(
          this.formatDemon(publication.demon),
          "Published&nbsp;by: ",
          publication.players.join(", ")
        );
      })
    );
  }

  makeTooltip(hoverElement, title, content) {
    let tooltipText = document.createElement("div");
    let b = document.createElement("b");

    b.innerHTML = title;
    tooltipText.appendChild(b);
    tooltipText.appendChild(document.createTextNode(content));
    tooltipText.classList.add("tooltiptext", "fade");

    let tooltip = document.createElement("div");

    tooltip.classList.add("tooltip");

    tooltip.appendChild(hoverElement);
    tooltip.appendChild(tooltipText);

    return tooltip;
  }

  formatDemonFromRecord(record) {
    let baseElement = this.formatDemon(record.demon);

    if (record.progress !== 100)
      baseElement.appendChild(
        document.createTextNode(" (" + record.progress + "%)")
      );

    let title =
      (record.progress === 100 ? "Beaten" : "Achieved") +
      "&nbsp;by&nbsp;" +
      record.players.length +
      "&nbsp;player" +
      (record.players.length === 1 ? "" : "s") +
      "&nbsp;in&nbsp;this&nbsp;country: ";

    return this.makeTooltip(baseElement, title, record.players.join(", "));
  }
}

$(window).on("load", function () {
  let map = new InteractiveWorldMap();

  window.statsViewer = new NationStatsViewer(
    document.getElementById("statsviewer")
  );
  window.statsViewer.initialize();
  window.statsViewer.addSelectionListener((selected) =>
    map.select(selected.country_code)
  );

  map.addSelectionListener((country, _) => {
    for (let li of window.statsViewer.list.children) {
      if (li.dataset.id === country) window.statsViewer.onSelect(li);
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
});

function generateStatsViewerNation(nation) {
  var li = document.createElement("li");
  var b = document.createElement("b");
  var i = document.createElement("i");

  li.className = "white hover";
  li.dataset.id = nation.country_code;
  li.dataset.rank = nation.rank;

  b.appendChild(document.createTextNode("#" + nation.rank + " "));
  i.appendChild(document.createTextNode(nation.score.toFixed(2)));

  li.appendChild(getCountryFlag(nation.nation, nation.country_code));
  li.appendChild(document.createTextNode(" "));

  li.appendChild(b);
  li.appendChild(document.createTextNode(nation.nation));
  li.appendChild(i);

  return li;
}

import {Dropdown} from "../modules/formv2.mjs";
import {getCountryFlag, populateSubdivisionDropdown} from "../modules/demonlistv2.mjs";
import {InteractiveWorldMap, StatsViewer} from "../statsviewer.js";

class IndividualStatsViewer extends StatsViewer {
    constructor(html) {
        super(html, {
            retrievalEndpoint: "/api/v1/players/",
            rankingEndpoint: "/api/v1/players/ranking/",
            entryGenerator: generateStatsViewerPlayer
        });
    }

    onReceive(response) {
        super.onReceive(response);

        var playerData = response.data.data;

        this.setName(playerData.name, playerData.nationality);

        this.formatDemonsInto(this._created, playerData.created);
        this.formatDemonsInto(this._published, playerData.published);
        this.formatDemonsInto(this._verified, playerData.verified);

        let beaten = playerData.records.filter((record) => record.progress === 100);

        beaten.sort((r1, r2) => r1.demon.name.localeCompare(r2.demon.name));

        let legacy = beaten.filter(
            (record) => record.demon.position > this.extended_list_size
        ).length;
        let extended = beaten.filter(
            (record) =>
                record.demon.position > this.list_size &&
                record.demon.position <= this.extended_list_size
        ).length;

        let verifiedExtended = playerData.verified.filter(demon => demon.position <= this.extended_list_size && demon.position > this.list_size).length;
        let verifiedLegacy = playerData.verified.filter(demon => demon.position > this.extended_list_size).length;

        this.formatRecordsInto(this._beaten, beaten);
        this.setCompletionNumber(beaten.length - legacy - extended + playerData.verified.length - verifiedExtended - verifiedLegacy, extended + verifiedExtended, legacy + verifiedLegacy);

        let hardest = playerData.verified
            .concat(beaten.map((record) => record.demon))
            .reduce((acc, next) => (acc.position > next.position ? next : acc), {name: "None", position: 321321321321});

        this.setHardest(hardest.name === "None" ? undefined : hardest);

        let non100Records = playerData.records
            .filter((record) => record.progress !== 100)
            .sort((r1, r2) => r1.progress - r2.progress);

        this.formatRecordsInto(this._progress, non100Records);
    }

    formatDemonsInto(element, demons) {
        while (element.lastChild) {
            element.removeChild(element.lastChild);
        }

        if (demons.length) {
            for (var demon of demons) {
                element.appendChild(
                    this.formatDemon(demon, "/demonlist/permalink/" + demon.id + "/")
                );
                element.appendChild(document.createTextNode(" - "));
            }
            element.removeChild(element.lastChild);
        } else {
            element.appendChild(document.createTextNode("None"));
        }
    }

    formatRecordsInto(element, records) {
        while (element.lastChild) {
            element.removeChild(element.lastChild);
        }

        if (records.length) {
            for (var record of records) {
                let demon = this.formatDemon(record.demon, "/demonlist/permalink/" + record.demon.id + "/");
                if (record.progress !== 100) {
                    demon.appendChild(
                        document.createTextNode(" (" + record.progress + "%)")
                    );
                }
                element.appendChild(demon);
                element.appendChild(document.createTextNode(" - "));
            }
            element.removeChild(element.lastChild);
        } else {
            element.appendChild(document.createTextNode("None"));
        }
    }
}

$(window).on("load", function () {
    let map = new InteractiveWorldMap();

    window.statsViewer = new IndividualStatsViewer(document.getElementById("statsviewer"));
    window.statsViewer.initialize();

    new Dropdown(
        document
            .getElementById("continent-dropdown")
    ).addEventListener(selected => {
        if(selected === "All") {
            window.statsViewer.updateQueryData("continent", undefined);
            map.resetContinentHighlight();
        } else {
            window.statsViewer.updateQueryData("continent", selected);
            map.highlightContinent(selected);
        }
    });

    let subdivisionDropdown = new Dropdown(document.getElementById("subdivision-dropdown"));

    subdivisionDropdown.addEventListener(selected => {
        if(selected === 'None') {
            map.deselectSubdivision();
            statsViewer.updateQueryData('subdivision', undefined);
        } else {
            let countryCode = statsViewer.queryData['nation'];

            map.select(countryCode, selected);
            statsViewer.updateQueryData2({nation: countryCode, subdivision: selected});
        }
    });

    statsViewer.dropdown.addEventListener(selected => {
        if(selected === 'International') {
            map.deselect();
        } else {
            map.select(selected);
        }

        // if 'countryCode == International' we send a nonsense request which results in a 404 and causes the dropdown to clear. That's exactly what we want, though.
        populateSubdivisionDropdown(subdivisionDropdown, selected);

        statsViewer.updateQueryData('subdivision', undefined);
    });

    map.addSelectionListener((countryCode, subdivisionCode) => {
        populateSubdivisionDropdown(subdivisionDropdown, countryCode).then(() => subdivisionDropdown.selectSilently(subdivisionCode));

        statsViewer.dropdown.selectSilently(countryCode);

        statsViewer.updateQueryData2({nation: countryCode, subdivision: subdivisionCode});
    });

    map.addDeselectionListener(() => {
        statsViewer.dropdown.selectSilently("International");
        subdivisionDropdown.clearOptions();
        statsViewer.updateQueryData2({nation: undefined, subdivision: undefined});
    });
});

function generateStatsViewerPlayer(player) {
    var li = document.createElement("li");
    var b = document.createElement("b");
    var i = document.createElement("i");

    li.className = "white hover";
    li.dataset.id = player.id;
    li.dataset.rank = player.rank;

    b.appendChild(document.createTextNode("#" + player.rank + " "));
    i.appendChild(document.createTextNode(player.score.toFixed(2)));

    if (player.nationality) {
        li.appendChild(getCountryFlag(player.nationality.nation, player.nationality.country_code));
        li.appendChild(document.createTextNode(" "));
    }

    li.appendChild(b);
    li.appendChild(document.createTextNode(player.name));
    li.appendChild(i);

    return li;
}

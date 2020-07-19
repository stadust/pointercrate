import {
  initializeRecordSubmitter,
  StatsViewer,
} from "./modules/demonlist.mjs";

$(document).ready(function () {
  initializePositionChart();
  initializeRecordSubmitter();

  window.statsViewer = new StatsViewer(document.getElementById("statsviewer"));

  document
    .getElementById("show-stats-viewer")
    .addEventListener("click", () => window.statsViewer.initialize());
});

function initializePositionChart() {
  if (window.positionChartData) {
    let highestPosition = Math.max(...window.positionChartData);
    let lowestPosition = Math.min(...window.positionChartData);

    let ticks = [-lowestPosition, -highestPosition];

    let span = highestPosition - lowestPosition;

    for (let historyPosition of window.positionChartData) {
      let shouldAdd = true;
      for (let tick of ticks) {
        if (Math.abs(historyPosition + tick) <= span / 10) shouldAdd = false;
      }
      if (shouldAdd) ticks.push(-historyPosition);
    }

    let chart = new Chartist.Line(
      "#position-chart",
      {
        labels: window.positionChartLabels,
        series: [window.positionChartData],
      },
      {
        lineSmooth: Chartist.Interpolation.step({ postpone: false }),
        axisX: {
          stretch: true,
          ticks: window.positionChartLabels,
          labelOffset: {
            x: -20,
          },
          type: Chartist.StepAxis,
        },
        axisY: {
          high: -lowestPosition,
          low: -highestPosition,
          ticks: ticks,
          type: Chartist.FixedScaleAxis,
          labelInterpolationFnc: function (value) {
            return -value;
          },
        },
      }
    );

    chart.on("data", function (context) {
      context.data.series = context.data.series.map(function (series) {
        return series.map(function (value) {
          return -value;
        });
      });
    });

    let observer = new MutationObserver(() => chart.update());
    observer.observe(document.getElementById("position-chart").parentNode, {
      attributes: true,
    });
  }
}

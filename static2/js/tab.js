$(document).ready(function() {
  $(".tabbed").each((_, tab) => {
    tab = $(tab);
    tab.find(".tab-selection .tab").each((idx, elem) => {
      var jelem = $(elem);
      jelem.click(() => {
        var tab_id = jelem.data("tab-id");
        tab.find(".tab-display .tab-content").each((idx, elem) => {
          var jelem2 = $(elem);
          if (jelem2.data("tab-id") == tab_id) {
            jelem2.addClass("tab-content-active");
          } else {
            jelem2.removeClass("tab-content-active");
          }
          tab.find(".tab-selection .tab").removeClass("tab-active");
          jelem.addClass("tab-active");
        });
      });
    });
  });
});

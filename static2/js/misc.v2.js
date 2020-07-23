function forceRatio(element, wRatio, hRatio) {
  var target = $(element);
  var width = target.width();
  var calculatedHeight = (width * hRatio) / wRatio;
  if (Math.abs(target.height() - calculatedHeight) > 20) {
    target.height((target.width() * hRatio) / wRatio);
  }
}

$(window).on("load resize", function () {
  // back to top things

  var scrollers = $(".js-scroll");
  var scrollTarget = $("html, body");

  scrollers.each((i, elem) => {
    var src = $(elem);

    src.click(() => {
      var dest = src.data("destination");
      var destination = $("#" + dest);

      if (src.data("reveal")) destination.fadeIn(1000);

      if (dest !== undefined)
        scrollTarget.animate({ scrollTop: destination.offset().top - 60 }, 400);
      else scrollTarget.animate({ scrollTop: 0 }, 400);
    });
  });

  // sometimes animating scrolltop causes things to get stuck. This fixes it.

  $(window).bind("mousewheel touchmove touchstart", function () {
    $("html, body").stop();
  });

  // Closable panels

  for (let x of document.querySelectorAll(".plus.cross")) {
    let parent = x.parentNode;

    while (parent !== null && parent.classList !== null) {
      if (parent.classList.contains("closable")) {
        x.addEventListener("click", () => $(parent).fadeOut(1000));
        break;
      }
      parent = parent.parentNode;
    }
  }

  // Animation stuff when scrolling
  var toAnimate = $(".js-scroll-anim");
  var wnd = $(window);

  wnd.on("scroll resize", checkAnimations);
  toAnimate.each((i, elem) => {
    var obj = $(elem);

    if (obj.data("js-shown") !== undefined) return;

    var observer = new MutationObserver(checkAnimations);
    var conf = {
      childList: false,
      attributes: true,
      characterData: false,
      attributeFilter: ["style"],
      subtree: true,
    };
    observer.observe(elem.parentElement, conf);

    obj.data("js-shown", true);
  });

  checkAnimations();

  function checkAnimations() {
    var viewportBottom = wnd.scrollTop() + wnd.innerHeight();

    toAnimate.each((i, elem) => {
      var obj = $(elem);
      var objBottom = obj.offset().top;

      if (objBottom <= viewportBottom && !obj.data("js-shown")) {
        switch (obj.data("anim")) {
          default:
          case "fade":
            obj.stop().fadeTo(500, 1);
            break;
        }
        obj.data("js-shown", true);
      } else if (objBottom > viewportBottom && obj.data("js-shown")) {
        switch (obj.data("anim")) {
          default:
          case "fade":
            obj.stop().fadeTo(500, 0);
            break;
        }
        obj.data("js-shown", false);
      }
    });
  }

  $(".js-collapse").each(function (i, elem) {
    var collapse = $(elem);
    var content = collapse.find(".js-collapse-content");
    var arrow = collapse.find(".arrow");

    arrow.parent().click(function () {
      if (!collapse.hasClass("js-sliding")) {
        collapse.addClass("js-sliding");
        if (collapse.hasClass("active")) {
          content.slideUp(250, () => {
            collapse.removeClass("active");
            collapse.removeClass("js-sliding");
          });
        } else {
          content.slideDown(250, () => {
            collapse.addClass("active");
            collapse.removeClass("js-sliding");
          });
        }
      }
    });
  });

  // ratio things

  $(".ratio-16-9").each(function () {
    forceRatio(this, 16, 9);
    if (this.tagName == "IFRAME") this.onload = () => forceRatio(this, 16, 9);
  });
  $(".ratio-4-3").each(function () {
    forceRatio(this, 4, 3);
    if (this.tagName == "IFRAME") this.onload = () => forceRatio(this, 4, 3);
  });

  $(".js-delay-css").each((i, elem) => {
    var elem = $(elem);
    var attr = elem.data("property");
    var value = elem.data("property-value");

    if (elem.css(attr) != value) elem.css(attr, value);
  });

  $(".js-delay-attr").each((i, elem) => {
    var elem = $(elem);
    var attr = elem.data("attr");
    var value = elem.data("attr-value");

    if (elem.attr(attr) != value) elem.attr(attr, value);
  });
});

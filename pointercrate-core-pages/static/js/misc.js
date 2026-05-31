function forceRatio(element, wRatio, hRatio) {
  var width = element.offsetWidth;
  var calculatedHeight = (width * hRatio) / wRatio;
  if (Math.abs(element.offsetHeight - calculatedHeight) > 20) {
    element.style.height = ((element.offsetWidth * hRatio) / wRatio) + "px";
  }
}

function initMisc() {
  // back to top things

  var scrollers = document.querySelectorAll(".js-scroll");

  scrollers.forEach(function (elem) {
    if (elem.dataset.miscBound) return;
    elem.dataset.miscBound = "true";

    elem.addEventListener("click", function () {
      var dest = elem.dataset.destination;
      var destination = dest ? document.getElementById(dest) : null;

      if (elem.dataset.reveal && destination) fadeIn(destination, 1000);

      if (destination) {
        var destinationTop = destination.getBoundingClientRect().top + window.scrollY;
        window.scrollTo({ top: destinationTop - 60, behavior: "smooth" });
      } else {
        window.scrollTo({ top: 0, behavior: "smooth" });
      }
    });
  });

  // Closable panels

  for (var x of document.querySelectorAll(".plus.cross")) {
    if (x.dataset.closableBound) continue;
    x.dataset.closableBound = "true";

    var parent = x.parentNode;

    while (parent !== null && parent.classList !== null) {
      if (parent.classList.contains("closable")) {
        x.addEventListener("click", function () {
          fadeOut(parent, 1000);
        });
        break;
      }
      parent = parent.parentNode;
    }
  }

  // Animation stuff when scrolling
  var toAnimate = document.querySelectorAll(".js-scroll-anim");

  window.addEventListener("scroll", checkAnimations);
  window.addEventListener("resize", checkAnimations);
  toAnimate.forEach(function (elem) {
    if (elem.dataset.jsShown !== undefined) return;

    var observer = new MutationObserver(checkAnimations);
    var conf = {
      childList: false,
      attributes: true,
      characterData: false,
      attributeFilter: ["style"],
      subtree: true,
    };
    observer.observe(elem.parentElement, conf);

    elem.dataset.jsShown = "true";
  });

  checkAnimations();

  function checkAnimations() {
    var viewportBottom = window.scrollY + window.innerHeight;

    toAnimate.forEach(function (elem) {
      var objBottom = elem.offsetTop;

      if (objBottom <= viewportBottom && elem.dataset.jsShown !== "true") {
        switch (elem.dataset.anim) {
          default:
          case "fade":
            stopAnimation(elem);
            fadeTo(elem, 500, 1);
            break;
        }
        elem.dataset.jsShown = "true";
      } else if (objBottom > viewportBottom && elem.dataset.jsShown === "true") {
        switch (elem.dataset.anim) {
          default:
          case "fade":
            stopAnimation(elem);
            fadeTo(elem, 500, 0);
            break;
        }
        elem.dataset.jsShown = "false";
      }
    });
  }

  document.querySelectorAll(".js-collapse").forEach(function (elem) {
    if (elem.dataset.collapseBound) return;
    elem.dataset.collapseBound = "true";

    var content = elem.querySelector(".js-collapse-content");
    var arrow = elem.querySelector(".arrow");

    if (!arrow || !arrow.parentElement) return;

    arrow.parentElement.addEventListener("click", function () {
      if (!elem.classList.contains("js-sliding")) {
        elem.classList.add("js-sliding");
        if (elem.classList.contains("active")) {
          slideUp(content, 250, function () {
            elem.classList.remove("active");
            elem.classList.remove("js-sliding");
          });
        } else {
          slideDown(content, 250, function () {
            elem.classList.add("active");
            elem.classList.remove("js-sliding");
          });
        }
      }
    });
  });

  // ratio things

  document.querySelectorAll(".ratio-16-9").forEach(function (elem) {
    forceRatio(elem, 16, 9);
    if (elem.tagName === "IFRAME") elem.onload = function () { forceRatio(elem, 16, 9); };
  });
  document.querySelectorAll(".ratio-4-3").forEach(function (elem) {
    forceRatio(elem, 4, 3);
    if (elem.tagName === "IFRAME") elem.onload = function () { forceRatio(elem, 4, 3); };
  });

  document.querySelectorAll(".js-delay-css").forEach(function (elem) {
    var attr = elem.dataset.property;
    var value = elem.dataset.propertyValue;

    if (getComputedStyle(elem).getPropertyValue(attr) != value) elem.style.setProperty(attr, value);
  });

  document.querySelectorAll(".js-delay-attr").forEach(function (elem) {
    var attr = elem.dataset.attr;
    var value = elem.dataset.attrValue;

    if (elem.getAttribute(attr) != value) elem.setAttribute(attr, value);
  });
}

window.addEventListener("load", initMisc);
window.addEventListener("resize", initMisc);

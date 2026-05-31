// quick replacements for jquery fade/slide animations

function fadeIn(element, duration, callback) {
  if (!element) return;
  var computed = getComputedStyle(element);
  if (computed.display === "none") {
    element.style.opacity = "0";
    element.style.display = "block";
  }
  element.offsetHeight;
  element.style.transition = "opacity " + duration + "ms ease";
  element.style.opacity = "1";
  if (callback) setTimeout(callback, duration);
}

function fadeOut(element, duration, callback) {
  if (!element) return;
  var computed = getComputedStyle(element);
  if (computed.display === "none") {
    if (callback) callback();
    return;
  }
  element.style.transition = "opacity " + duration + "ms ease";
  element.style.opacity = "0";
  setTimeout(function () {
    element.style.display = "none";
    if (callback) callback();
  }, duration);
}

function fadeTo(element, duration, opacity, callback) {
  if (!element) return;
  var computed = getComputedStyle(element);
  if (computed.display === "none" && opacity > 0) {
    element.style.opacity = "0";
    element.style.display = "block";
  }
  element.offsetHeight;
  element.style.transition = "opacity " + duration + "ms ease";
  element.style.opacity = opacity.toString();
  if (callback) setTimeout(callback, duration);
}

function slideDown(element, duration, callback) {
  if (!element) return;
  var computed = getComputedStyle(element);
  if (computed.display === "none") {
    element.style.maxHeight = "0";
    element.style.overflow = "hidden";
    element.style.display = "block";
  }
  var targetHeight = element.scrollHeight + "px";
  element.offsetHeight;
  element.style.transition = "max-height " + duration + "ms cubic-bezier(0.45, 0.05, 0.55, 0.95)";
  element.style.maxHeight = targetHeight;
  if (callback) setTimeout(callback, duration);
}

function slideUp(element, duration, callback) {
  if (!element) return;
  var computed = getComputedStyle(element);
  if (computed.display === "none") {
    if (callback) callback();
    return;
  }
  element.style.transition = "max-height " + duration + "ms cubic-bezier(0.45, 0.05, 0.55, 0.95)";
  element.style.maxHeight = "0";
  element.style.overflow = "hidden";
  setTimeout(function () {
    element.style.display = "none";
    if (callback) callback();
  }, duration);
}

function stopAnimation(element) {
  if (!element) return;
  var style = getComputedStyle(element);
  var opacity = style.opacity;
  var maxHeight = style.maxHeight;
  element.style.transition = "";
  element.style.opacity = opacity;
  element.style.maxHeight = maxHeight;
}

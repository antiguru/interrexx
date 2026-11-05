// Version picker for the published specification. The site holds one
// directory per version (`main/`, `v0.1.0/`, ...) beside a `versions.json`
// listing them; this adds a picker to the menu bar that opens the same page in
// another version. When `versions.json` is absent, as under `mdbook serve`,
// nothing is added.
(function () {
  "use strict";
  if (typeof path_to_root === "undefined") {
    return;
  }
  var bookRoot = new URL(path_to_root || "./", window.location.href);
  var siteRoot = new URL("../", bookRoot);
  var current = bookRoot.pathname.slice(siteRoot.pathname.length).replace(/\/$/, "");
  var page = window.location.pathname.slice(bookRoot.pathname.length);

  fetch(new URL("versions.json", siteRoot))
    .then(function (response) {
      return response.ok ? response.json() : null;
    })
    .then(function (versions) {
      if (!Array.isArray(versions) || versions.length === 0) {
        return;
      }
      var bar = document.querySelector(".right-buttons");
      if (!bar) {
        return;
      }
      var select = document.createElement("select");
      select.className = "version-picker";
      select.style.margin = "0 0.5em";
      select.setAttribute("aria-label", "Specification version");
      versions.forEach(function (v) {
        var option = document.createElement("option");
        option.value = v;
        option.textContent = v;
        option.selected = v === current;
        select.appendChild(option);
      });
      select.addEventListener("change", function () {
        window.location.href =
          new URL(select.value + "/" + page, siteRoot).href + window.location.hash;
      });
      bar.insertBefore(select, bar.firstChild);
    })
    .catch(function () {});
})();

// Open external links in new tab
document.addEventListener("DOMContentLoaded", function () {
  const links = document.querySelectorAll("a[href^='http']");
  const host = window.location.hostname;

  links.forEach(function (link) {
    if (!link.hostname.includes(host)) {
      link.setAttribute("target", "_blank");
      link.setAttribute("rel", "noopener noreferrer");
    }
  });
});


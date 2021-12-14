(async () => {
	const index = await fetch(chrome.runtime.getURL("/dist/generated_files.json"))
		.then(r => r.json());

	const fontawesome = document.createElement("script");
	fontawesome.src = "https://kit.fontawesome.com/67998b1eca.js";
	fontawesome.crossOrigin = "anonymous";

	const bulma = document.createElement("link");
	bulma.rel = "stylesheet";
	bulma.href = "https://cdn.jsdelivr.net/npm/bulma@0.9.3/css/bulma.min.css";

	const css = document.createElement("link");
	css.rel = "stylesheet";
	css.href = chrome.runtime.getURL("/dist/" + index[".css"]);

	document.head.append(fontawesome, bulma, css);

	const src = chrome.runtime.getURL("/dist/" + index[".js"]);
	const contentMain = await import(src);
	contentMain.default();
})();
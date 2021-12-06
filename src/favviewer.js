/*import init from "./init";

async function initFavViewer() {
	console.log("Init FavViewer");
	const css = await fetch(new URL('index.css', import.meta.url)).then(r => r.text());
	
	const fontawesome = document.createElement("script");
	fontawesome.src = "https://kit.fontawesome.com/67998b1eca.js";
	fontawesome.crossorigin = "anonymous";
	document.head.append(fontawesome);

	const bulma = document.createElement("link");
	bulma.rel = "stylesheet";
	bulma.href = "https://cdn.jsdelivr.net/npm/bulma@0.9.3/css/bulma.min.css";
	document.head.append(bulma);

	const style = document.createElement("style");
	style.innerHTML = css;
	document.head.append(style);

	init()
}

export default initFavViewer;*/
(async () => {	
	const fontawesome = document.createElement("script");
	fontawesome.src = "https://kit.fontawesome.com/67998b1eca.js";
	fontawesome.crossorigin = "anonymous";
	document.head.append(fontawesome);

	const bulma = document.createElement("link");
	bulma.rel = "stylesheet";
	bulma.href = "https://cdn.jsdelivr.net/npm/bulma@0.9.3/css/bulma.min.css";
	document.head.append(bulma);

	const css = await fetch("http://localhost:3000/favviewer/index.css").then(r => r.text());

	const style = document.createElement("style");
	style.innerHTML = css;
	document.head.append(style);

	const m = await import('http://localhost:3000/favviewer/init')
	m.default()
})()
const importObject = {
	imports: {
		imported_func: function(arg) {
			console.log(arg);
		}
	}
};

const response = null;
const bytes = null;
const results = null;

const wasmPath = chrome.runtime.getURL("index-780e0c641604af13_bg.wasm");
console.log("myPath: " + wasmPath);

(async () => {
	const src = chrome.runtime.getURL("/dist/index-780e0c641604af13.js");
	const contentMain = await import(src);
	contentMain.default();
})();
/*fetch(wasmPath).then(response =>
	response.arrayBuffer()
).then(bytes =>
	WebAssembly.instantiate(bytes, importObject)
).then(results => {
	results.instance.exports.exported_func();
});*/

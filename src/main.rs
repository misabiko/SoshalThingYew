use soshalthing_yew::{Model, favviewer::try_inject};

fn main() {
	wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

	match web_sys::window()
		.map(|w| w.location())
		.map(|l| l.href()) {
		Some(Ok(href)) => {
			let href = href.as_str();
			if !try_inject(href) {
				yew::start_app::<Model>();
			}
		},
		None => log::error!("Failed to get location.href."),
		Some(Err(err)) => log::error!("Failed to get location.href.\n{}", &err.as_string().unwrap_or("Failed to parse the error.".to_string())),
	};
}

//TODO Profile lag when redrawing articles
//TODO Youtube articles
//TODO Have custom service setting view
//TODO Show quato units for Youtube service
//TODO Cache playlist id for each subscribed channel
//TODO Custom social buttons per article type
//TODO Notifications
//TODO Save timeline data
//TODO Display timeline errors
//TODO Prompt on not logged in
//TODO Avoid refreshing endpoint every watch update
//TODO Add "Open @myusername on soshalthing" context menu?

//TODO Show multiple article types in same timeline
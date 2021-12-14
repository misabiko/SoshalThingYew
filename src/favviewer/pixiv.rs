use yew::prelude::*;
use std::collections::HashMap;

use crate::favviewer::PageInfo;

#[derive(PartialEq, Eq, Hash)]
pub enum Style {
	Hidden,
	Pixiv,
}

pub struct PixivPageInfo {
	style_html: HashMap<Style, Html>,
	style: Style,
	favviewer_button: Html,
}

impl PixivPageInfo {
	pub fn new(on_activator_click: Callback<web_sys::MouseEvent>) -> Self {
		let document_head = gloo_utils::document().head().expect("head element to be present");
		let mut style_html = HashMap::new();
		style_html.insert(Style::Pixiv, create_portal(html! {
                <style>{"#root {width: 100%} #root > :nth-child(2), .sc-1nr368f-2.bGUtlw { height: 100%; } .sc-jgyytr-1 {display: none}"}</style>
			}, document_head.clone().into()
		));
		style_html.insert(Style::Hidden, create_portal(html! {
                <style>{"#favviewer {display: none;} #root {width: 100%} "}</style>
			}, document_head.into()
		));

		let favviewer_button_mount = gloo_utils::document()
			.query_selector(".sc-s8zj3z-6.kstoDd")
			.expect("couldn't query activator mount point")
			.expect("couldn't find activator mount point");
		let favviewer_button = create_portal(html! {
				<a class="sc-d98f2c-0" onclick={on_activator_click}>
					<span class="sc-93qi7v-2 ibdURy">{"FavViewer"}</span>
				</a>
			}, favviewer_button_mount.into());

		Self {
			style_html,
			style: Style::Hidden,
			favviewer_button,
		}
	}
}

impl PageInfo for PixivPageInfo {
	fn style_html(&self) -> Html {
		self.style_html[&self.style].clone()
	}

	fn favviewer_button(&self) -> Html {
		self.favviewer_button.clone()
	}

	fn toggle_hidden(&mut self) {
		self.style = match &self.style {
			Style::Hidden => Style::Pixiv,
			Style::Pixiv => Style::Hidden,
		}
	}
}
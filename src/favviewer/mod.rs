use std::collections::HashMap;
use yew::prelude::*;

mod pixiv;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FavViewerStyle {
	Hidden,
	Normal,
}

#[derive(Clone, PartialEq)]
pub enum PageInfo {
	Setup {
		style_html: HashMap<FavViewerStyle, Html>,
		initial_style: FavViewerStyle,
		make_activator: fn(Callback<MouseEvent>) -> Html,
		add_timelines: fn(),
	},
	Ready {
		style_html: HashMap<FavViewerStyle, Html>,
		style: FavViewerStyle,
		favviewer_button: Html,
	}
}

impl PageInfo {
	pub fn toggle_hidden(&mut self) {
		if let PageInfo::Ready {style, ..} = self {
			*style = match style {
				FavViewerStyle::Hidden => FavViewerStyle::Normal,
				FavViewerStyle::Normal => FavViewerStyle::Hidden,
			};
		}
	}

	pub fn view(&self) -> Html {
		match &self {
			PageInfo::Ready { favviewer_button, style, style_html} => html! {
				<>
					{ favviewer_button.clone() }
					{ style_html[&style].clone() }
				</>
			},
			_ => html! {}
		}
	}
}

pub fn try_inject(href: &str, ) -> bool {
	pixiv::setup(href)
}
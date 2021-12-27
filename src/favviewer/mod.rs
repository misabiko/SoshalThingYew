use yew::prelude::*;

mod pixiv;

use crate::Model;

pub trait PageInfo {
	fn style_html(&self) -> Html;

	fn favviewer_button(&self) -> Html;

	fn toggle_hidden(&mut self);

	fn view(&self) -> Html {
		html! {
			<>
				{ self.favviewer_button() }
				{ self.style_html() }
			</>
		}
	}

	fn add_timeline(&self, ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>);
}

pub fn try_inject(href: &str, ) -> bool {
	pixiv::setup(href)
}

pub fn page_info(ctx: &Context<Model>) -> Option<Box<dyn PageInfo>> {
	pixiv::page_info(ctx)
}
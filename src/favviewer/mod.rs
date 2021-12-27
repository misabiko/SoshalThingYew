use yew::prelude::*;

mod pixiv;

pub use pixiv::FollowPageInfo;
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
}

pub fn try_inject(href: &str, ) -> bool {
	pixiv::setup_pixiv(href)
}

pub fn page_info(ctx: &Context<Model>) -> Option<Box<dyn PageInfo>> {
	pixiv::page_info(ctx)
}
use yew::prelude::*;

pub mod pixiv;

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
use yew::prelude::*;

mod pixiv;

pub use pixiv::PixivPageInfo;

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
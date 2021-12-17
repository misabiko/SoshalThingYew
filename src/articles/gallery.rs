use yew::prelude::*;

use crate::articles::Props;

pub struct GalleryArticle {
	compact: Option<bool>,
}

pub enum Msg {
	ToggleCompact,
	OnImageClick,
}

impl Component for GalleryArticle {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			compact: None,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleCompact => match self.compact {
				Some(compact) => self.compact = Some(!compact),
				None => self.compact = Some(!ctx.props().compact),
			},
			Msg::OnImageClick => ctx.link().send_message(Msg::ToggleCompact)
		};

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let strong = ctx.props().data.upgrade().unwrap();
		let actual_article = strong.referenced_article().and_then(|w| w.upgrade()).unwrap_or_else(|| strong.clone());
		html! {
			<article class="article galleryArticle" articleId={actual_article.id()} style={ctx.props().style.clone()}>
				{ for actual_article.media().iter().map(|m| html! {
					<img src={m.clone()}/>
				}) }
			</article>
		}
	}
}
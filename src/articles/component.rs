use std::cell::RefCell;
use std::rc::Weak;
use yew::prelude::*;
use super::{ArticleData, ArticleView, SocialArticle, GalleryArticle};

pub struct ArticleComponent {

}

#[derive(Properties, Clone)]
pub struct Props {
	pub article: Weak<RefCell<dyn ArticleData>>,
	pub article_view: ArticleView,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	#[prop_or_default]
	pub style: Option<String>
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.article, &other.article) &&
			self.article_view == other.article_view &&
			self.compact == other.compact &&
			self.animated_as_gifs == other.animated_as_gifs &&
			self.hide_text == other.hide_text &&
			self.style == other.style
	}
}

pub enum Msg {

}

impl Component for ArticleComponent {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		Self {}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		if let Some(strong) = ctx.props().article.upgrade() {
			match &ctx.props().article_view {
				ArticleView::Social => html! {
					<SocialArticle
						key={strong.borrow().id()}
						compact={ctx.props().compact.clone()}
						animated_as_gifs={ctx.props().animated_as_gifs.clone()}
						hide_text={ctx.props().hide_text.clone()}
						style={ctx.props().style.clone()}
						data={ctx.props().article.clone()}
					/>
				},
				ArticleView::Gallery => html! {
					<GalleryArticle
						key={strong.borrow().id()}
						compact={ctx.props().compact.clone()}
						animated_as_gifs={ctx.props().animated_as_gifs.clone()}
						hide_text={ctx.props().hide_text.clone()}
						style={ctx.props().style.clone()}
						data={ctx.props().article.clone()}
					/>
				},
			}
		}else {
			html! {}
		}
	}
}
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use std::rc::Weak;

use crate::articles::{ArticleRefType, Props, ArticleMedia};
use crate::services::article_actions::{ArticleActionsAgent, Response as ArticleActionsResponse};

pub struct GalleryArticle {
	compact: Option<bool>,
	_article_actions: Box<dyn Bridge<ArticleActionsAgent>>,
}

pub enum Msg {
	ToggleCompact,
	OnImageClick,
	ActionsCallback(ArticleActionsResponse),
}

impl Component for GalleryArticle {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			compact: None,
			_article_actions: ArticleActionsAgent::bridge(ctx.link().callback(Msg::ActionsCallback)),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleCompact => {
				match self.compact {
					Some(compact) => self.compact = Some(!compact),
					None => self.compact = Some(!ctx.props().compact),
				};
				true
			},
			Msg::OnImageClick => {
				ctx.link().send_message(Msg::ToggleCompact);
				true
			},
			Msg::ActionsCallback(response) => {
				match response {
					ArticleActionsResponse::Callback(articles)
					=> articles.iter().any(|a| Weak::ptr_eq(a, &ctx.props().data))
				}
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let strong = ctx.props().data.upgrade().unwrap();
		let borrow = strong.borrow();
		let actual_article = match &borrow.referenced_article() {
			ArticleRefType::NoRef => strong.clone(),
			ArticleRefType::Repost(a) => a.upgrade().unwrap(),
			ArticleRefType::Quote(_) => strong.clone()
		};
		let actual_borrow = actual_article.borrow();

		html! {
			<article class="article galleryArticle" articleId={actual_borrow.id()} style={ctx.props().style.clone()}>
				{ for actual_borrow.media().iter().map(|m| match m {
					ArticleMedia::Image(src) => html! {
						<img src={src.clone()}/>
					},
					ArticleMedia::Video(video_src) => html! {
						<video controls=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
							<source src={video_src.clone()} type="video/mp4"/>
						</video>
					},
					ArticleMedia::Gif(gif_src) => html! {
						<video controls=true autoplay=true loop=true muted=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
							<source src={gif_src.clone()} type="video/mp4"/>
						</video>
					},
				}) }
			</article>
		}
	}
}
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use std::rc::Weak;
use wasm_bindgen::closure::Closure;

use crate::articles::{ArticleRefType, Props, ArticleMedia};
use crate::services::article_actions::{ArticleActionsAgent, Response as ArticleActionsResponse};

pub struct GalleryArticle {
	compact: Option<bool>,
	video_ref: NodeRef,
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
			video_ref: NodeRef::default(),
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
			<article class="article galleryArticle" articleId={actual_borrow.id()} key={borrow.id()} style={ctx.props().style.clone()}>
				{ for actual_borrow.media().iter().map(|m| match (&ctx.props().animated_as_gifs, m) {
					(_, ArticleMedia::Image(src)) => html! {
						<img src={src.clone()}/>
					},
					(false, ArticleMedia::Video(video_src)) => html! {
						<video ref={self.video_ref.clone()} controls=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
							<source src={video_src.clone()} type="video/mp4"/>
						</video>
					},
					(_, ArticleMedia::Gif(gif_src)) | (true, ArticleMedia::Video(gif_src)) => html! {
						<video ref={self.video_ref.clone()} controls=true autoplay=true loop=true muted=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
							<source src={gif_src.clone()} type="video/mp4"/>
						</video>
					},
				}) }
			</article>
		}
	}

	fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
		if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
			match ctx.props().animated_as_gifs {
				true => {
					video.set_muted(true);
					match video.play() {
						Ok(promise) => {
							let _ = promise.catch(&Closure::once(Box::new(|err| log::warn!("Failed to play video.\n{:?}", &err))));
						}
						Err(err) => log::warn!("Failed to try and play the video.\n{:?}", &err)
					}
				},
				false => {
					video.set_muted(false);
					match video.pause() {
						Err(err) => log::warn!("Failed to try and pause the video.\n{:?}", &err),
						Ok(_) => {}
					}
				},
			};
		}
	}
}
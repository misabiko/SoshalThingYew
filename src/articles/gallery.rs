use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
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
					ArticleActionsResponse::Callback(articles) => {
						//For some reason Weak::ptr_eq() always returns false
						let strong = ctx.props().data.upgrade().unwrap();
						let borrow = strong.borrow();
						articles.iter().any(|a| {
							let strong_a = a.upgrade().unwrap();
							let eq = borrow.id() == strong_a.borrow().id();
							eq
						})
					}
				}
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let strong = ctx.props().data.upgrade().unwrap();
		let borrow = strong.borrow();
		let actual_article = match &borrow.referenced_article() {
			ArticleRefType::NoRef | ArticleRefType::Quote(_) => strong.clone(),
			ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a.upgrade().unwrap(),
		};
		let actual_borrow = actual_article.borrow();

		html! {
			<article class="article galleryArticle" articleId={actual_borrow.id()} key={borrow.id()} style={ctx.props().style.clone()}>
				{ for actual_borrow.media().iter().map(|m| match (&ctx.props().animated_as_gifs, m) {
					(_, ArticleMedia::Image(src, _) | ArticleMedia::Gif(src, _)) => html! {
						<img src={src.clone()}/>
					},
					(false, ArticleMedia::Video(video_src, _)) => html! {
						<video ref={self.video_ref.clone()} controls=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
							<source src={video_src.clone()} type="video/mp4"/>
						</video>
					},
					(_, ArticleMedia::VideoGif(gif_src, _)) | (true, ArticleMedia::Video(gif_src, _)) => html! {
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
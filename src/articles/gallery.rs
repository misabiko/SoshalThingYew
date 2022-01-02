use std::cell::Ref;
use yew::prelude::*;
use wasm_bindgen::closure::Closure;

use crate::articles::{ArticleRefType, ArticleMedia, ArticleData};
use crate::articles::component::{ViewProps, Msg as ParentMsg};
use crate::dropdown::{Dropdown, DropdownLabel};

pub struct GalleryArticle {
	draw_on_top: bool,
}

pub enum Msg {
	ParentCallback(ParentMsg),
	SetDrawOnTop(bool),
}

impl Component for GalleryArticle {
	type Message = Msg;
	type Properties = ViewProps;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			draw_on_top: false,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ParentCallback(message) => {
				ctx.props().parent_callback.emit(message);
				false
			},
			Msg::SetDrawOnTop(value) => {
				self.draw_on_top = value;
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let strong = ctx.props().article.upgrade().unwrap();
		let borrow = strong.borrow();
		let actual_article = match &borrow.referenced_article() {
			ArticleRefType::NoRef | ArticleRefType::Quote(_) => strong.clone(),
			ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a.upgrade().unwrap(),
		};
		let actual_borrow = actual_article.borrow();

		let style = match self.draw_on_top {
			false => ctx.props().style.clone(),
			true => Some(format!("{} z-index: 20", ctx.props().style.clone().unwrap_or_default())),
		};

		html! {
			<article class="article galleryArticle" articleId={actual_borrow.id()} key={borrow.id()} {style}>
				{ self.view_media(ctx, &actual_borrow) }
				{ self.view_nav(ctx, &actual_borrow) }
			</article>
		}
	}

	fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
		if let Some(video) = ctx.props().video_ref.cast::<web_sys::HtmlVideoElement>() {
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

impl GalleryArticle {
	fn view_media(&self, ctx: &Context<Self>, actual_article: &Ref<dyn ArticleData>) -> Html {
		html! {
			<>
				{ for actual_article.media().iter().map(|m| match (&ctx.props().animated_as_gifs, m) {
					(_, ArticleMedia::Image(src, _) | ArticleMedia::Gif(src, _)) => html! {
								<img src={src.clone()} onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnImageClick))}/>
							},
					(false, ArticleMedia::Video(video_src, _)) => html! {
								<video ref={ctx.props().video_ref.clone()} controls=true onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnImageClick))}>
									<source src={video_src.clone()} type="video/mp4"/>
								</video>
							},
					(_, ArticleMedia::VideoGif(gif_src, _)) | (true, ArticleMedia::Video(gif_src, _)) => html! {
								<video ref={ctx.props().video_ref.clone()} controls=true autoplay=true loop=true muted=true onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnImageClick))}>
									<source src={gif_src.clone()} type="video/mp4"/>
								</video>
							},
				}) }
			</>
		}
	}

	fn view_nav(&self, ctx: &Context<Self>, actual_article: &Ref<dyn ArticleData>) -> Html {
		html! {
			<>
				<div class="holderBox holderBoxTop">
					<a class="button" title="External Link" href={actual_article.url()} target="_blank">
						<span class="icon darkIcon is-small">
							<i class="fas fa-external-link-alt"/>
						</span>
					</a>
					<button class="button" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleInModal))}>
						<span class="icon darkIcon is-small">
							<i class="fas fa-expand-arrows-alt"/>
						</span>
					</button>
					<Dropdown on_expanded_change={ctx.link().callback(Msg::SetDrawOnTop)} is_right=true current_label={DropdownLabel::Icon("fas fa-ellipsis-h".to_owned())} label_classes={classes!("articleButton")}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleMarkAsRead))}> {"Mark as read"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleHide))}> {"Hide"} </a>
						<a
							class="dropdown-item"
							href={ actual_article.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::LogData))}>{"Log Data"}</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::FetchData))}>{"Fetch Data"}</a>
					</Dropdown>
				</div>
				<div class="holderBox holderBoxBottom">
					<button class="button" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Like))}>
						<span class="icon darkIcon is-small">
							<i class="fas fa-heart"/>
						</span>
					</button>
					<button class="button" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Repost))}>
						<span class="icon darkIcon is-small">
							<i class="fas fa-retweet"/>
						</span>
					</button>
				</div>
			</>
		}
	}
}
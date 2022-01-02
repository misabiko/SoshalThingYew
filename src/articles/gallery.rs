use std::cell::Ref;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use wasm_bindgen::closure::Closure;
use web_sys::console;
use wasm_bindgen::JsValue;

use crate::articles::{ArticleRefType, Props, ArticleMedia, ArticleData};
use crate::services::article_actions::{ArticleActionsAgent, Request as ArticleActionsRequest, Response as ArticleActionsResponse};
use crate::dropdown::{Dropdown, DropdownLabel};

pub struct GalleryArticle {
	compact: Option<bool>,
	video_ref: NodeRef,
	article_actions: Box<dyn Bridge<ArticleActionsAgent>>,
	in_modal: bool,
	draw_on_top: bool,
}

pub enum Msg {
	ToggleCompact,
	OnImageClick,
	ActionsCallback(ArticleActionsResponse),
	ToggleMarkAsRead,
	ToggleHide,
	ToggleInModal,
	LogData,
	FetchData,
	SetDrawOnTop(bool),
}

impl Component for GalleryArticle {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			compact: None,
			article_actions: ArticleActionsAgent::bridge(ctx.link().callback(Msg::ActionsCallback)),
			video_ref: NodeRef::default(),
			in_modal: false,
			draw_on_top: false,
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
			//TODO Move to articles/mod.rs
			Msg::LogData => {
				let strong = ctx.props().data.upgrade().unwrap();
				let json = &strong.borrow().json();
				let is_mobile = web_sys::window().expect("couldn't get global window")
					.navigator().user_agent()
					.map(|n| n.contains("Mobile"))
					.unwrap_or(false);
				if is_mobile {
					log::info!("{}", serde_json::to_string_pretty(json).unwrap_or("Couldn't parse json data.".to_owned()));
				}else {
					console::dir_1(&JsValue::from_serde(&json).unwrap_or_default());
				}
				false
			}
			Msg::FetchData => {
				let strong = ctx.props().data.upgrade().unwrap();
				let borrow = strong.borrow();

				self.article_actions.send(ArticleActionsRequest::FetchData(match borrow.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => ctx.props().data.clone(),
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a,
				}));
				false
			}
			Msg::ToggleMarkAsRead => {
				if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
					video.set_muted(true);
					match video.pause() {
						Err(err) => log::warn!("Failed to try and pause the video.\n{:?}", &err),
						Ok(_) => {}
					}
				}

				let strong = ctx.props().data.upgrade().unwrap();
				let mut borrow = strong.borrow_mut();

				match borrow.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => {
						let marked_as_read = borrow.marked_as_read();
						borrow.set_marked_as_read(!marked_as_read);
						self.article_actions.send(ArticleActionsRequest::MarkAsRead(ctx.props().data.clone(), !marked_as_read));
					},
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => {
						let strong = a.upgrade().unwrap();
						let mut borrow = strong.borrow_mut();

						let marked_as_read = borrow.marked_as_read();
						borrow.set_marked_as_read(!marked_as_read);
						self.article_actions.send(ArticleActionsRequest::MarkAsRead(a.clone(), !marked_as_read));
					}
				};

				true
			}
			Msg::ToggleHide => {
				if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
					video.set_muted(true);
					match video.pause() {
						Err(err) => log::warn!("Failed to try and pause the video.\n{:?}", &err),
						Ok(_) => {}
					}
				}

				let strong = ctx.props().data.upgrade().unwrap();
				let mut borrow = strong.borrow_mut();

				match borrow.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => {
						let hidden = borrow.hidden();
						borrow.set_hidden(!hidden);
					},
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => {
						let strong = a.upgrade().unwrap();
						let mut borrow = strong.borrow_mut();

						let hidden = borrow.hidden();
						borrow.set_hidden(!hidden);
					}
				};

				true
			}
			Msg::ToggleInModal => {
				self.in_modal = !self.in_modal;
				true
			}
			Msg::SetDrawOnTop(value) => {
				self.draw_on_top = value;
				true
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

impl GalleryArticle {
	fn view_media(&self, ctx: &Context<Self>, actual_article: &Ref<dyn ArticleData>) -> Html {
		html! {
			<>
				{ for actual_article.media().iter().map(|m| match (&ctx.props().animated_as_gifs, m) {
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
					/*<button class="button" onclick={actual_article.url()} target="_blank">
						<span class="icon darkIcon is-small">
							<i class="fas fa-external-link-alt"/>
						</span>
					</button>*/
					<Dropdown on_expanded_change={ctx.link().callback(Msg::SetDrawOnTop)} is_right=true current_label={DropdownLabel::Icon("fas fa-ellipsis-h".to_owned())} label_classes={classes!("articleButton")}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ToggleMarkAsRead)}> {"Mark as read"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ToggleHide)}> {"Hide"} </a>
						<a
							class="dropdown-item"
							href={ actual_article.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::LogData)}>{"Log Data"}</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::FetchData)}>{"Fetch Data"}</a>
					</Dropdown>
				</div>
				<div class="holderBox holderBoxBottom">
				</div>
			</>
		}
	}
}
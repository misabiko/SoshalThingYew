use std::cell::RefCell;
use std::rc::Weak;
use yew::prelude::*;
use yew_agent::{Dispatcher, Dispatched, Bridge, Bridged};
use web_sys::console;
use wasm_bindgen::JsValue;
use std::convert::identity;

use super::{ArticleView, SocialArticle, GalleryArticle};
use crate::articles::{ArticleData, ArticleRefType};
use crate::articles::media_load_queue::{MediaLoadAgent, Request as MediaLoadRequest, Response as MediaLoadResponse, MediaLoadState};
use crate::services::article_actions::{ArticleActionsAgent, Request as ArticleActionsRequest};
use crate::modals::Modal;
use crate::error::log_warn;
use crate::services::storages::mark_article_as_read;

pub struct ArticleComponent {
	in_modal: bool,
	article_actions: Dispatcher<ArticleActionsAgent>,
	video_ref: NodeRef,
	media_load_states: Vec<MediaLoadState>,
	media_load_queue: Box<dyn Bridge<MediaLoadAgent>>,
}

pub enum Msg {
	OnImageClick,
	LogData,
	LogJsonData,
	FetchData,
	Like,
	Repost,
	ToggleMarkAsRead,
	ToggleHide,
	ToggleInModal,
	LoadMedia(usize),
	MediaLoaded(usize),
	MediaLoadResponse(MediaLoadResponse),
}

#[derive(Properties)]
pub struct Props {
	pub weak_ref: Weak<RefCell<dyn ArticleData>>,
	pub article: Box<dyn ArticleData>,
	pub article_view: ArticleView,
	pub load_priority: u32,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	#[prop_or_default]
	pub style: Option<String>,
	#[prop_or_default]
	pub lazy_loading: bool,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.weak_ref, &other.weak_ref) &&
			self.article_view == other.article_view &&
			self.compact == other.compact &&
			self.animated_as_gifs == other.animated_as_gifs &&
			self.hide_text == other.hide_text &&
			self.style == other.style &&
			self.lazy_loading == other.lazy_loading &&
			self.load_priority == other.load_priority &&
			&self.article == &other.article
	}
}

impl Clone for Props {
	fn clone(&self) -> Self {
		Props {
			weak_ref: self.weak_ref.clone(),
			article: self.article.clone_data(),
			article_view: self.article_view.clone(),
			compact: self.compact,
			animated_as_gifs: self.animated_as_gifs,
			hide_text: self.hide_text,
			style: self.style.clone(),
			lazy_loading: self.lazy_loading,
			load_priority: self.load_priority,
		}
	}
}

#[derive(Properties)]
pub struct ViewProps {
	pub weak_ref: Weak<RefCell<dyn ArticleData>>,
	pub article: Box<dyn ArticleData>,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	pub in_modal: bool,
	pub video_ref: NodeRef,
	//Maybe use ctx.link().get_parent()?
	pub parent_callback: Callback<Msg>,
	pub media_load_states: Vec<MediaLoadState>,
}

impl PartialEq<ViewProps> for ViewProps {
	fn eq(&self, other: &ViewProps) -> bool {
		self.compact == other.compact &&
			self.animated_as_gifs == other.animated_as_gifs &&
			self.hide_text == other.hide_text &&
			self.in_modal == other.in_modal &&
			self.media_load_states == other.media_load_states &&
			Weak::ptr_eq(&self.weak_ref, &other.weak_ref) &&
			&self.article == &other.article
	}
}

impl Clone for ViewProps {
	fn clone(&self) -> Self {
		Self {
			weak_ref: self.weak_ref.clone(),
			article: self.article.clone_data(),
			compact: self.compact.clone(),
			animated_as_gifs: self.animated_as_gifs.clone(),
			hide_text: self.hide_text.clone(),
			in_modal: self.in_modal.clone(),
			video_ref: self.video_ref.clone(),
			parent_callback: self.parent_callback.clone(),
			media_load_states: self.media_load_states.clone(),
		}
	}
}

impl Component for ArticleComponent {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut media_load_queue = MediaLoadAgent::bridge(ctx.link().callback(Msg::MediaLoadResponse));

		//TODO Avoir first-come-first-serve on initial load
		if ctx.props().lazy_loading {
			let id = ctx.props().article.id();
			let media = ctx.props().article.media();
			let media_to_queue = media.iter()
				.enumerate()
				.filter_map(|(i, m)|
					if m.queue_load_info.is_some() {Some(i) } else { None }
				);

			for i in media_to_queue {
				media_load_queue.send(MediaLoadRequest::QueueMedia(id.clone(), i, ctx.props().load_priority));
			}
		}

		Self {
			in_modal: false,
			article_actions: ArticleActionsAgent::dispatcher(),
			video_ref: NodeRef::default(),
			media_load_states: ctx.props().article.media().iter().map(|m|
				if ctx.props().lazy_loading && m.queue_load_info.as_ref().map(|q| !q.loaded).unwrap_or(false) {
					MediaLoadState::NotLoaded
				}else {
					MediaLoadState::Loaded	//Consider still setting it as loading?
				}
			).collect(),
			media_load_queue,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::OnImageClick => {
				ctx.link().send_message(Msg::ToggleMarkAsRead);
				false
			}
			Msg::LogJsonData => {
				let json = &ctx.props().article.json();
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
			Msg::LogData => {
				log::info!("{:#?}", &ctx.props().article);
				false
			}
			Msg::FetchData => {
				self.article_actions.send(ArticleActionsRequest::FetchData(match ctx.props().article.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => ctx.props().weak_ref.clone(),
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a,
				}));
				false
			}
			Msg::Like => {
				self.article_actions.send(ArticleActionsRequest::Like(match ctx.props().article.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => ctx.props().weak_ref.clone(),
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a,
				}));
				false
			}
			Msg::Repost => {
				self.article_actions.send(ArticleActionsRequest::Repost(match ctx.props().article.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => ctx.props().weak_ref.clone(),
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a,
				}));
				false
			}
			Msg::ToggleMarkAsRead => {
				if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
					video.set_muted(true);
					match video.pause() {
						Err(err) => log_warn(Some("Failed to try and pause the video"), err),
						Ok(_) => {}
					}
				}

				let strong = ctx.props().weak_ref.upgrade().unwrap();
				let mut borrow = strong.borrow_mut();

				match borrow.referenced_article() {
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => {
						let new_marked_as_read = !borrow.marked_as_read();
						borrow.set_marked_as_read(new_marked_as_read);

						mark_article_as_read(borrow.service(), borrow.id(), new_marked_as_read);
					},
					ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => {
						let strong = a.upgrade().unwrap();
						let mut borrow = strong.borrow_mut();

						let new_marked_as_read = !borrow.marked_as_read();
						borrow.set_marked_as_read(new_marked_as_read);
						mark_article_as_read(borrow.service(), borrow.id(), new_marked_as_read);
					}
				};

				self.article_actions.send(ArticleActionsRequest::RedrawTimelines(vec![ctx.props().weak_ref.clone()]));

				true
			}
			Msg::ToggleHide => {
				if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
					video.set_muted(true);
					match video.pause() {
						Err(err) => log_warn(Some("Failed to try and pause the video"), err),
						Ok(_) => {}
					}
				}

				let strong = ctx.props().weak_ref.upgrade().unwrap();
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

				self.article_actions.send(ArticleActionsRequest::RedrawTimelines(vec![ctx.props().weak_ref.clone()]));

				true
			}
			Msg::ToggleInModal => {
				self.in_modal = !self.in_modal;
				true
			}
			Msg::LoadMedia(index) => {
				self.media_load_queue.send(MediaLoadRequest::LoadMedia(ctx.props().article.id(), index));
				self.media_load_states[index] = MediaLoadState::Loading;
				true
			}
			Msg::MediaLoaded(index) => {
				self.media_load_queue.send(MediaLoadRequest::MediaLoaded(ctx.props().article.id(), index));
				self.media_load_states[index] = MediaLoadState::Loaded;
				let article = ctx.props().weak_ref.upgrade().unwrap();
				let mut article = article.borrow_mut();
				article.media_loaded(index);
				//This is for future loads of the article, no need to redraw

				true
			}
			Msg::MediaLoadResponse(response) => match response {
				MediaLoadResponse::UpdateState(index, state) => {
					self.media_load_states[index] = state;
					true
				},
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let view_html = match &ctx.props().article_view {
			ArticleView::Social => html! {
				<SocialArticle
					key={ctx.props().article.id()}
					weak_ref={ctx.props().weak_ref.clone()}
					article={ctx.props().article.clone_data()}
					compact={ctx.props().compact.clone()}
					animated_as_gifs={ctx.props().animated_as_gifs.clone()}
					hide_text={ctx.props().hide_text.clone()}
					in_modal={self.in_modal.clone()}
					video_ref={self.video_ref.clone()}
					parent_callback={ctx.link().callback(identity)}
					media_load_states={self.media_load_states.clone()}
				/>
			},
			ArticleView::Gallery => html! {
				<GalleryArticle
					key={ctx.props().article.id()}
					weak_ref={ctx.props().weak_ref.clone()}
					article={ctx.props().article.clone_data()}
					compact={ctx.props().compact.clone()}
					animated_as_gifs={ctx.props().animated_as_gifs.clone()}
					hide_text={ctx.props().hide_text.clone()}
					in_modal={self.in_modal.clone()}
					video_ref={self.video_ref.clone()}
					parent_callback={ctx.link().callback(identity)}
					media_load_states={self.media_load_states.clone()}
				/>
			},
		};

		let class = match ctx.props().article_view {
			ArticleView::Social => "article socialArticle".to_owned(),
			ArticleView::Gallery => "article galleryArticle".to_owned(),
		};

		//For some reason, the view needs at least a wrapper otherwise when changing article_view, the container draws everything in reverse order...
		let article_html = html! {
			<article {class} articleId={ctx.props().article.id()} key={ctx.props().article.id()} style={ctx.props().style.clone()}>
				{ view_html }
			</article>
		};

		if self.in_modal {
			html! {
				<Modal content_style="width: 75%" close_modal_callback={ctx.link().callback(|_| Msg::ToggleInModal)}>
					{ article_html }
				</Modal>
			}
		}else {
			article_html
		}
	}
}
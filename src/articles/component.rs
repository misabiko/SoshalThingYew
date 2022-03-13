use yew::prelude::*;
use yew_agent::{Dispatcher, Dispatched, Bridge, Bridged};
use web_sys::console;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use std::convert::identity;

use super::{ArticleView, SocialArticle, GalleryArticle};
use crate::articles::{MediaQueueInfo, ArticleMedia, weak_actual_article};
use crate::articles::media_load_queue::{MediaLoadAgent, Request as MediaLoadRequest, Response as MediaLoadResponse, MediaLoadState};
use crate::services::article_actions::{ArticleActionsAgent, Request as ArticleActionsRequest};
use crate::modals::Modal;
use crate::log_warn;
use crate::services::storages::{hide_article, mark_article_as_read};
use crate::settings::{AppSettings, OnMediaClick, ArticleFilteredMode};
use crate::timeline::ArticleStruct;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = twemoji, js_name = parse)]
	fn twemoji_parse(node: web_sys::Node, options: TwemojiOptions);
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct TwemojiOptions {
	folder: &'static str,
	ext: &'static str,
}

pub struct ArticleComponent {
	in_modal: bool,
	article_actions: Dispatcher<ArticleActionsAgent>,
	video_ref: NodeRef,
	component_ref: NodeRef,
	previous_media: Vec<ArticleMedia>,
	media_load_states: Vec<MediaLoadState>,
	media_load_queue: Box<dyn Bridge<MediaLoadAgent>>,
}

pub enum Msg {
	OnMediaClick,
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

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub article_struct: ArticleStruct,
	pub article_view: ArticleView,
	pub load_priority: u32,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	#[prop_or_default]
	pub style: Option<String>,
	#[prop_or_default]
	pub lazy_loading: bool,
	pub column_count: u8,
	pub app_settings: AppSettings,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ViewProps {
	pub article_struct: ArticleStruct,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	pub in_modal: bool,
	pub video_ref: NodeRef,
	//Maybe use ctx.link().get_parent()?
	pub parent_callback: Callback<Msg>,
	pub media_load_states: Vec<MediaLoadState>,
	pub column_count: u8,
	pub app_settings: AppSettings,
}

impl Component for ArticleComponent {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut media_load_queue = MediaLoadAgent::bridge(ctx.link().callback(Msg::MediaLoadResponse));

		//TODO Avoid first-come-first-serve on initial load
		if ctx.props().lazy_loading {
			let id = ctx.props().article_struct.boxed.id();
			let media = ctx.props().article_struct.boxed.media();
			let media_to_queue = media.iter()
				.enumerate()
				.filter_map(|(i, m)|
					if let MediaQueueInfo::LazyLoad {..} = m.queue_load_info { Some(i) } else { None }
				);

			for i in media_to_queue {
				media_load_queue.send(MediaLoadRequest::QueueMedia(id.clone(), i, ctx.props().load_priority));
			}
		}

		Self {
			in_modal: false,
			article_actions: ArticleActionsAgent::dispatcher(),
			video_ref: NodeRef::default(),
			component_ref: NodeRef::default(),
			previous_media: ctx.props().article_struct.boxed.media(),
			media_load_states: make_media_load_states(ctx.props().lazy_loading, ctx.props().article_struct.boxed.media()),
			media_load_queue,
		}
	}

	fn changed(&mut self, ctx: &Context<Self>) -> bool {
		let new_media = ctx.props().article_struct.boxed.media();
		if self.previous_media != new_media {
			self.media_load_states = make_media_load_states(ctx.props().lazy_loading, ctx.props().article_struct.boxed.media());

			//TODO Avoid first-come-first-serve on initial load
			if self.previous_media.is_empty() && ctx.props().lazy_loading {
				let id = ctx.props().article_struct.boxed.id();
				let media = ctx.props().article_struct.boxed.media();
				let media_to_queue = media.iter()
					.enumerate()
					.filter_map(|(i, m)|
						if let MediaQueueInfo::LazyLoad {..} = m.queue_load_info { Some(i) } else { None }
					);

				for i in media_to_queue {
					self.media_load_queue.send(MediaLoadRequest::QueueMedia(id.clone(), i, ctx.props().load_priority));
				}
			}
			self.previous_media = new_media;
		}
		true
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::OnMediaClick => {
				match ctx.props().app_settings.on_media_click {
					OnMediaClick::Like =>
						ctx.link().send_message(Msg::Like),
					OnMediaClick::Repost =>
						ctx.link().send_message(Msg::Repost),
					OnMediaClick::Expand =>
						ctx.link().send_message(Msg::ToggleInModal),
					OnMediaClick::MarkAsRead =>
						ctx.link().send_message(Msg::ToggleMarkAsRead),
					OnMediaClick::Hide =>
						ctx.link().send_message(Msg::ToggleHide),
					OnMediaClick::Nothing => {}
				}
				false
			}
			Msg::LogJsonData => {
				let json = &ctx.props().article_struct.boxed.json();
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
				log::info!("{:#?}", &ctx.props().article_struct.boxed);
				false
			}
			Msg::FetchData => {
				self.article_actions.send(ArticleActionsRequest::FetchData(weak_actual_article(&ctx.props().article_struct.weak)));
				false
			}
			Msg::Like => {
				self.article_actions.send(ArticleActionsRequest::Like(weak_actual_article(&ctx.props().article_struct.weak)));
				false
			}
			Msg::Repost => {
				self.article_actions.send(ArticleActionsRequest::Repost(weak_actual_article(&ctx.props().article_struct.weak)));
				false
			}
			Msg::ToggleMarkAsRead => {
				if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
					video.set_muted(true);
					match video.pause() {
						Err(err) => log_warn!("Failed to try and pause the video", err),
						Ok(_) => {}
					}
				}

				let actual_article = &ctx.props().article_struct.boxed_actual_article();
				let weak_actual_article = weak_actual_article(&ctx.props().article_struct.weak);

				let new_marked_as_read = !actual_article.marked_as_read();
				weak_actual_article.upgrade().unwrap().borrow_mut().set_marked_as_read(new_marked_as_read);
				mark_article_as_read(actual_article.service(), actual_article.id(), new_marked_as_read);

				self.article_actions.send(ArticleActionsRequest::RedrawTimelines(vec![weak_actual_article.clone()]));

				true
			}
			Msg::ToggleHide => {
				if let Some(video) = self.video_ref.cast::<web_sys::HtmlVideoElement>() {
					video.set_muted(true);
					match video.pause() {
						Err(err) => log_warn!("Failed to try and pause the video", err),
						Ok(_) => {}
					}
				}

				let actual_article = &ctx.props().article_struct.boxed_actual_article();
				let weak_actual_article = weak_actual_article(&ctx.props().article_struct.weak);

				let new_hidden = !actual_article.hidden();
				weak_actual_article.upgrade().unwrap().borrow_mut().set_hidden(new_hidden);
				hide_article(actual_article.service(), actual_article.id(), new_hidden);

				self.article_actions.send(ArticleActionsRequest::RedrawTimelines(vec![weak_actual_article.clone()]));

				true
			}
			Msg::ToggleInModal => {
				self.in_modal = !self.in_modal;
				true
			}
			Msg::LoadMedia(index) => {
				self.media_load_queue.send(MediaLoadRequest::LoadMedia(ctx.props().article_struct.boxed.id(), index));
				self.media_load_states[index] = MediaLoadState::Loading;
				true
			}
			Msg::MediaLoaded(index) => {
				self.media_load_queue.send(MediaLoadRequest::MediaLoaded(ctx.props().article_struct.boxed.id(), index));
				self.media_load_states[index] = MediaLoadState::Loaded;
				let article = ctx.props().article_struct.weak.upgrade().unwrap();
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
					key={ctx.props().article_struct.boxed.id()}
					article_struct={ctx.props().article_struct.clone()}
					compact={ctx.props().compact}
					animated_as_gifs={ctx.props().animated_as_gifs}
					hide_text={ctx.props().hide_text}
					in_modal={self.in_modal}
					video_ref={self.video_ref.clone()}
					parent_callback={ctx.link().callback(identity)}
					media_load_states={self.media_load_states.clone()}
					column_count={ctx.props().column_count}
					app_settings={ctx.props().app_settings}
				/>
			},
			ArticleView::Gallery => html! {
				<GalleryArticle
					key={ctx.props().article_struct.boxed.id()}
					article_struct={ctx.props().article_struct.clone()}
					compact={ctx.props().compact}
					animated_as_gifs={ctx.props().animated_as_gifs}
					hide_text={ctx.props().hide_text}
					in_modal={self.in_modal}
					video_ref={self.video_ref.clone()}
					parent_callback={ctx.link().callback(identity)}
					media_load_states={self.media_load_states.clone()}
					column_count={ctx.props().column_count}
					app_settings={ctx.props().app_settings}
				/>
			},
		};

		let class = classes!(
			"article",
			match ctx.props().article_view {
				ArticleView::Social => "socialArticle",
				ArticleView::Gallery => "galleryArticle",
			},
			if !ctx.props().article_struct.included && ctx.props().app_settings.article_filtered_mode == ArticleFilteredMode::Transparent {
				Some("transparent")
			}else {
				None
			},
		);

		//For some reason, the view needs at least a wrapper otherwise when changing article_view, the container draws everything in reverse order...
		let article_html = html! {
			<article {class} articleId={ctx.props().article_struct.boxed.id()} style={ctx.props().style.clone()} ref={self.component_ref.clone()}>
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

	/*fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
		//TODO "Node not found to remove VText"
		if first_render && JsValue::from_str("twemoji").js_in(&gloo_utils::window()) {
			if let Some(component_ref) = self.component_ref.get() {
				twemoji_parse(component_ref, TwemojiOptions {
					folder: "svg",
					ext: ".svg",
				})
			}
		}
	}*/
}

fn make_media_load_states(lazy_loading: bool, media: Vec<ArticleMedia>) -> Vec<MediaLoadState> {
	media.into_iter().map(|m|
		if !lazy_loading {
			MediaLoadState::Loaded
		}else {
			match m.queue_load_info {
				MediaQueueInfo::LazyLoad { loaded, .. } if loaded => MediaLoadState::Loaded,
				_ => MediaLoadState::NotLoaded,
			}
		}
	).collect()
}
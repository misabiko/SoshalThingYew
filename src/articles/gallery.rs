use yew::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;

use crate::articles::{MediaType, MediaQueueInfo, media_load_queue::MediaLoadState};
use crate::articles::component::{ViewProps, ArticleComponentMsg as ParentMsg};
use crate::components::{Dropdown, DropdownLabel};
use crate::components::{FA, font_awesome::FAProps};
use crate::services::article_actions::Action;
use crate::log_warn;

pub struct GalleryArticle {
	draw_on_top: bool,
}

pub enum GalleryArticleMsg {
	ParentCallback(ParentMsg),
	SetDrawOnTop(bool),
}

type Msg = GalleryArticleMsg;

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
		let style = match self.draw_on_top {
			true => Some("z-index: 20".to_owned()),
			false => None,
		};

		html! {
			<div {style}>
				{ self.view_media(ctx) }
				{ self.view_nav(ctx) }
			</div>
		}
	}

	fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
		if let Some(video) = ctx.props().video_ref.cast::<web_sys::HtmlVideoElement>() {
			match ctx.props().animated_as_gifs {
				true => {
					video.set_muted(true);
					match video.play() {
						Ok(promise) => {
							let _ = promise.catch(&Closure::once(Box::new(|err: JsValue| log_warn!("Failed to play video", err))));
						}
						Err(err) => log_warn!("Failed to try and play the video", err)
					}
				},
				false => {
					video.set_muted(false);
					match video.pause() {
						Err(err) => log_warn!("Failed to try and pause the video", err),
						Ok(_) => {}
					}
				},
			};
		}
	}
}

impl GalleryArticle {
	fn view_media(&self, ctx: &Context<Self>) -> Html {
		html! {
			<>
				{ for ctx.props().article_struct.boxed_actual_article().media().iter().enumerate().zip(ctx.props().media_load_states.iter()).map(|((i, m), load_state)| {
					let thumb = match &m.queue_load_info {
						MediaQueueInfo::LazyLoad { thumbnail, .. } => match thumbnail {
							Some((src, _)) => Some(html! {
								<img key={i} class="articleThumb" src={src.clone()} onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}/>
							}),
							None => Some(html! {
								<img key={i} class="articleThumb" style={format!("background-color: grey; height: calc({} * 100vw / {})", &m.ratio, &ctx.props().column_count)}/>
							}),
						},
						_ => None,
					};

					if thumb.is_some() && *load_state == MediaLoadState::NotLoaded {
						thumb.unwrap()
					}else {
						let onloaded = ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::MediaLoaded(i)));
						let is_loading = *load_state == MediaLoadState::Loading;
						match (&ctx.props().animated_as_gifs, m.media_type) {
							(_, MediaType::Image | MediaType::Gif) => html! {
								<>
									<img
										key={i}
										src={m.src.clone()}
										onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}
										onload={if is_loading { Some(onloaded.clone()) } else { None }}
										class={if is_loading { Some("articleMediaLoading") } else { None }}
									/>
									{
										if is_loading {
											thumb.unwrap_or_default()
										}else {
											html! {}
										}
									}
								</>
							},
							(false, MediaType::Video) => html! {
								<video
									key={i}
									ref={ctx.props().video_ref.clone()}
									controls=true
									onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}
									onloadeddata={if is_loading { Some(onloaded.clone()) } else { None }}
									onload={if is_loading { Some(onloaded.clone()) } else { None }}
								>
									<source src={m.src.clone()} type="video/mp4"/>
								</video>
							},
							(_, MediaType::VideoGif) | (true, MediaType::Video) => html! {
								<video
								 	key={i}
									ref={ctx.props().video_ref.clone()}
									controls=true
									autoplay=true
									loop=true
									muted=true
									onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}
									onloadeddata={if is_loading { Some(onloaded.clone()) } else { None }}
									onload={if is_loading { Some(onloaded.clone()) } else { None }}
								>
									<source src={m.src.clone()} type="video/mp4"/>
								</video>
							},
						}
					}
				}) }
			</>
		}
	}

	fn view_nav(&self, ctx: &Context<Self>) -> Html {
		let actual_article = ctx.props().article_struct.boxed_actual_article();
		html! {
			<>
				<div class="holderBox holderBoxTop">
					<a class="button" title="External Link" href={actual_article.url()} target="_blank">
						<FA icon="external-link-alt" span_classes={classes!("darkIcon", "is-small")}/>
					</a>
					{
						if !ctx.props().in_modal {
							html! {
								<button class="button" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleInModal))}>
									<FA icon="expand-arrows-alt" span_classes={classes!("darkIcon", "is-small")}/>
								</button>
							}
						}else {
							html! {}
						}
					}
					<Dropdown on_expanded_change={ctx.link().callback(Msg::SetDrawOnTop)} is_right=true current_label={DropdownLabel::Icon(yew::props! { FAProps {icon: "ellipsis-h".to_owned()}})} label_classes={classes!("articleButton")}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Action(Action::MarkAsRead, None)))}> {"Mark as read"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Action(Action::Hide, None)))}> {"Hide"} </a>
						{ if let Some(index) = ctx.props().media_load_states.iter().enumerate().find_map(|(i, m)| if *m == MediaLoadState::NotLoaded { Some(i) } else { None }) {
							html! {
								<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::LoadMedia(index)))}>{"Load Media"}</a>
							}
						}else {
							html! {}
						} }
						<a
							class="dropdown-item"
							href={ actual_article.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Action(Action::LogData, None)))}>{"Log Data"}</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Action(Action::LogJsonData, None)))}>{"Log Json Data"}</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Action(Action::FetchData, None)))}>{"Fetch Data"}</a>
					</Dropdown>
				</div>
				<div class="holderBox holderBoxBottom">
					<button class="button" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::Like, None)))}>
						<FA icon="heart" span_classes={classes!("darkIcon", "is-small")}/>
					</button>
					<button class="button" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::Repost, None)))}>
						<FA icon="retweet" span_classes={classes!("darkIcon", "is-small")}/>
					</button>
				</div>
			</>
		}
	}
}
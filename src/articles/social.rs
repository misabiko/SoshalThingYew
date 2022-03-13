use yew::prelude::*;
use js_sys::Date;
use std::cell::Ref;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use yew_agent::{Dispatcher, Dispatched};

use crate::articles::{ArticleData, ArticleRefType, MediaType};
use crate::articles::component::{ViewProps, Msg as ParentMsg};
use crate::components::{Dropdown, DropdownLabel, FA, IconType, font_awesome::Props as FAProps};
use crate::timeline::agent::{TimelineAgent, Request as TimelineAgentRequest};
use crate::log_warn;
use crate::settings::ArticleFilteredMode;

pub struct SocialArticle {
	compact: Option<bool>,
	add_timeline_agent: Dispatcher<TimelineAgent>,
}

pub enum Msg {
	ParentCallback(ParentMsg),
	ToggleCompact,
	AddUserTimeline(&'static str, String),
}

impl Component for SocialArticle {
	type Message = Msg;
	type Properties = ViewProps;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			compact: None,
			add_timeline_agent: TimelineAgent::dispatcher(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ParentCallback(message) => {
				ctx.props().parent_callback.emit(message);
				false
			},
			Msg::ToggleCompact => {
				match self.compact {
					Some(compact) => self.compact = Some(!compact),
					None => self.compact = Some(!ctx.props().compact),
				};
				true
			}
			Msg::AddUserTimeline(service, username) => {
				self.add_timeline_agent.send(TimelineAgentRequest::AddUserTimeline(service, username));
				false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let strong = ctx.props().article_struct.weak.upgrade().unwrap();
		let borrow = strong.borrow();

		let (actual_article, retweet_header, quoted_post) = match &borrow.referenced_article() {
			ArticleRefType::NoRef => (strong.clone(), html! {}, html! {}),
			ArticleRefType::Reposted(a) => (
				a.upgrade().unwrap(),
				self.view_repost_label(ctx, &borrow),
				html! {}
			),
			ArticleRefType::Quote(a) => {
				let quote_article = a.upgrade().unwrap();
				let quote_borrow = quote_article.borrow();
				(strong.clone(), html! {}, self.view_quoted_post(ctx, &quote_borrow))
			}
			ArticleRefType::RepostedQuote(a, q) => {
				let reposted_article = a.upgrade().unwrap();

				let quoted_article = q.upgrade().unwrap();
				let quoted_borrow = quoted_article.borrow();
				(reposted_article.clone(), self.view_repost_label(ctx, &borrow), self.view_quoted_post(ctx, &quoted_borrow))
			}
		};
		let actual_borrow = actual_article.borrow();

		let actual_c = actual_article.clone();
		let on_username_click = ctx.link().callback(move |e: MouseEvent| {
			let borrow = actual_c.borrow();
			e.prevent_default();
			Msg::AddUserTimeline(borrow.service(), borrow.author_username())
		});

		html! {
			<>
				{ retweet_header }
				<div class="media">
					{ self.view_avatar(ctx) }
					<div class="media-content">
						<div class="content">
							<div class="articleHeader">
								<a class="names" href={actual_borrow.author_url()} target="_blank" rel="noopener noreferrer" onclick={on_username_click}>
									<strong>{ actual_borrow.author_name() }</strong>
									<small>{ format!("@{}", actual_borrow.author_username()) }</small>
								</a>
								{ self.view_timestamp(&actual_borrow) }
							</div>
							{ match ctx.props().hide_text || self.is_minimized(ctx) {
								false => html! {<p class="articleParagraph">{ actual_borrow.view_text() }</p>},
								true => html! {},
							} }
						</div>
						{ quoted_post }
						{ self.view_nav(ctx, &actual_borrow) }
					</div>
				</div>
				{ match self.is_minimized(ctx) {
					false => self.view_media(ctx, &actual_borrow),
					true => html! {},
				} }
			</>
		}
	}

	fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
		//TODO Make method for video pausing
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

impl SocialArticle {
	fn is_compact(&self, ctx: &Context<Self>) -> bool {
		match self.compact {
			Some(compact) => compact,
			None => ctx.props().compact,
		}
	}

	fn is_minimized(&self, ctx: &Context<Self>) -> bool {
		!ctx.props().article_struct.included && ctx.props().app_settings.article_filtered_mode == ArticleFilteredMode::Minimized
	}

	fn view_timestamp(&self, actual_article: &Ref<dyn ArticleData>) -> Html {
		let label = short_timestamp(&actual_article.creation_time());

		html! {
			<span class="timestamp">
				<small title={actual_article.creation_time().to_string().as_string()}>{ label }</small>
			</span>
		}
	}

	fn view_nav(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>) -> Html {
		//TODO Use cloned data instead of borrowing immutable
		let strong = ctx.props().article_struct.weak.upgrade().unwrap();
		let borrow = strong.borrow();
		let ontoggle_compact = ctx.link().callback(|_| Msg::ToggleCompact);
		let ontoggle_markasread = ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleMarkAsRead));
		let dropdown_buttons = match &borrow.referenced_article() {
			ArticleRefType::NoRef => html! {},
			ArticleRefType::Reposted(_) | ArticleRefType::RepostedQuote(_, _) => html! {
				<a
					class="dropdown-item"
					href={ borrow.url() }
					target="_blank" rel="noopener noreferrer"
				>
					{ "Repost's External Link" }
				</a>
			},
			ArticleRefType::Quote(_) => html! {},
		};

		html! {
			<nav class="level is-mobile">
				<div class="level-left">
					{ match self.is_minimized(ctx) {
						false => html! {
							<>
								<a
									class={classes!("level-item", "articleButton", "repostButton", if actual_borrow.reposted() { Some("repostedPostButton") } else { None })}
									onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Repost))}
								>
									<FA icon="retweet"/>
									{match actual_borrow.repost_count() {
										0 => html! {},
										count => html! {
											<span>{ count }</span>
										}
									}}
								</a>
								<a
									class={classes!("level-item", "articleButton", "likeButton", if actual_borrow.liked() { Some("likedPostButton") } else { None })}
									onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Like))}
								>
									<FA icon="heart" icon_type={if actual_borrow.liked() { IconType::Solid } else { IconType::Regular }}/>
									{match actual_borrow.like_count() {
										0 => html! {},
										count => html! {
											<span>{ count }</span>
										}
									}}
								</a>
								{
									match &actual_borrow.media().iter().map(|m| m.media_type).collect::<Vec<MediaType>>()[..] {
										[MediaType::Image, ..] => html! {
											<a class="level-item articleButton" onclick={&ontoggle_compact}>
												<FA icon={if self.is_compact(ctx) { "compress" } else { "expand" }}/>
											</a>
										},
										_ => html! {},
									}
								}
								<a class="level-item articleButton" onclick={&ontoggle_markasread}>
									<FA icon={if actual_borrow.marked_as_read() { "eye" } else { "eye-slash" }}/>
								</a>
								{
									match ctx.props().in_modal {
										false => html! {
											<a class="level-item articleButton" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleInModal))}>
												<FA icon="expand-alt"/>
											</a>
										},
										true => html! {},
									}
								}

							</>
						},
						true => html! {},
					} }
					<Dropdown current_label={DropdownLabel::Icon(yew::props! { FAProps {icon: "ellipsis-h".to_owned()}})} trigger_classes={classes!("level-item")} label_classes={classes!("articleButton")}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleMarkAsRead))}> {"Mark as read"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleHide))}> {"Hide"} </a>
						<a class="dropdown-item" onclick={&ontoggle_compact}> { if self.is_compact(ctx) { "Show expanded" } else { "Show compact" } } </a>
						<a
							class="dropdown-item"
							href={ actual_borrow.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						{ dropdown_buttons }
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::LogData))}>{"Log Data"}</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::LogJsonData))}>{"Log Json Data"}</a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::FetchData))}>{"Fetch Data"}</a>
					</Dropdown>
				</div>
			</nav>
		}
	}

	fn view_media(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>) -> Html {
		//TODO Show thumbnail if queue_load_info
		let type_src_tuples: Vec<(MediaType, String)> = actual_borrow.media().iter().map(|m| (m.media_type, m.src.clone())).collect();
		match (&ctx.props().animated_as_gifs, &type_src_tuples[..]) {
			(_, [(MediaType::Image, _), ..]) => {
				let images_classes = classes!(
						"postMedia",
						"postImages",
						if self.is_compact(ctx) { Some("postImagesCompact") } else { None }
					);

				html! {
					<div class={images_classes.clone()}> {
						match &type_src_tuples[..] {
							[(MediaType::Image, src)] => self.view_image(ctx, actual_borrow, src.clone(), false),
							[(MediaType::Image, src_1), (MediaType::Image, src_2)] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, src_1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, src_2.clone(), false) }
								</>
							},
							[(MediaType::Image, src_1), (MediaType::Image, src_2), (MediaType::Image, src_3)] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, src_1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, src_2.clone(), false) }
									{ self.view_image(ctx, actual_borrow, src_3.clone(), true) }
								</>
							},
							[(MediaType::Image, src_1), (MediaType::Image, src_2), (MediaType::Image, src_3), (MediaType::Image, src_4), ..] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, src_1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, src_2.clone(), false) }
									{ self.view_image(ctx, actual_borrow, src_3.clone(), false) }
									{ self.view_image(ctx, actual_borrow, src_4.clone(), false) }
								</>
							},
							_ => html! {{"unexpected media format"}}
						}
					} </div>
				}
			}
			(false, [(MediaType::Video, video_src)]) => html! {
				<div class="postMedia postVideo">
					<video ref={ctx.props().video_ref.clone()} controls=true onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}>
						<source src={video_src.clone()} type="video/mp4"/>
					</video>
				</div>
			},
			(_, [(MediaType::VideoGif, gif_src)]) | (true, [(MediaType::Video, gif_src)]) => html! {
				<div class="postMedia postVideo">
					<video ref={ctx.props().video_ref.clone()} controls=true autoplay=true loop=true muted=true onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}>
						<source src={gif_src.clone()} type="video/mp4"/>
					</video>
				</div>
			},
			(_, []) => html! {},
			_ => html! {{"unexpected media format"}}
		}
	}

	fn view_image(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>, image: String, is_large_third: bool) -> Html {
		let media_holder_classes = classes!(
			"mediaHolder",
			if self.is_compact(ctx) { Some("mediaHolderCompact") } else { None },
			if is_large_third { Some("thirdImage") } else { None },
		);

		html! {
			<div class={media_holder_classes}>
				<div class="is-hidden imgPlaceholder"/>
				<img alt={actual_borrow.id()} src={image} onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}/>
			</div>
		}
	}

	fn view_quoted_post(&self, ctx: &Context<Self>, quoted: &Ref<dyn ArticleData>) -> Html {
		html! {
			<div class="quotedPost">
				<div class="articleHeader">
					<a class="names" href={quoted.author_url()} target="_blank" rel="noopener noreferrer">
						<strong>{ quoted.author_name() }</strong>
						<small>{ format!("@{}", quoted.author_username()) }</small>
					</a>
					{ self.view_timestamp(&quoted) }
				</div>
				{ match self.is_minimized(ctx) {
					false => html! {
						<>
							{ match ctx.props().hide_text {
								false => html! { <p class="refArticleParagraph">{quoted.text()}</p> },
								true => html! {},
							} }
							{ self.view_media(ctx, &quoted) }
						</>
					},
					true => html! {},
				} }
			</div>
		}
	}

	fn view_repost_label(&self, ctx: &Context<Self>, repost: &Ref<dyn ArticleData>) -> Html {
		let service = repost.service();
		let username = repost.author_username();
		let onclick = ctx.link().callback(move |e: MouseEvent| {
			e.prevent_default();
			Msg::AddUserTimeline(service.clone(), username.clone())
		});
		html! {
			<div class="repostLabel"
				href={repost.url()}
				target="_blank">
				<a {onclick}>
					{ format!("{} reposted - {}", &repost.author_name(), short_timestamp(&repost.creation_time())) }
				</a>
			</div>
		}
	}

	fn view_avatar(&self, ctx: &Context<Self>) -> Html {
		let article = &ctx.props().article_struct.boxed;
		match article.author_avatar_url().as_str() {
			"" => html! {},
			url => html! {
				<figure class="media-left">
					{ match &ctx.props().article_struct.boxed_ref {
						ArticleRefType::NoRef | ArticleRefType::Quote(_) => html! {
							<p class="image is-64x64">
								<img src={url.to_owned()} alt={format!("{}'s avatar", &article.author_username())}/>
							</p>
						},
						ArticleRefType::Reposted(a) | ArticleRefType::RepostedQuote(a, _) => html! {
							<p class="image is-64x64 sharedAvatar">
								<img src={a.author_avatar_url().as_str().to_owned()} alt={format!("{}'s avatar", &a.author_username())}/>
								<img src={url.to_owned()} alt={format!("{}'s avatar", &article.author_username())}/>
							</p>
						},
					} }
				</figure>
			}
		}
	}
}

static MONTH_ABBREVS: [&'static str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn short_timestamp(date: &Date) -> String {
	let time_since = Date::now() - date.get_time();

	if time_since < 1000.0 {
		"just now".to_owned()
	} else if time_since < 60_000.0 {
		format!("{}s", (time_since / 1000.0).floor())
	} else if time_since < 3600_000.0 {
		format!("{}m", (time_since / 60000.0).floor())
	} else if time_since < 86400000.0 {
		format!("{}h", (time_since / (3600000.0)).floor())
	} else if time_since < 604800000.0 {
		format!("{}d", (time_since / (86400000.0)).floor())
	} else {
		format!("{} {} {}", MONTH_ABBREVS[date.get_month() as usize], date.get_date(), date.get_full_year())
	}
}
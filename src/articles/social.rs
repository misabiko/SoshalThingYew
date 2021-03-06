use yew::prelude::*;
use js_sys::Date;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use yew_agent::{Dispatcher, Dispatched};

use crate::articles::{ArticleBox, ArticleWeak, ArticleRefType, MediaType, UnfetchedArticleRef};
use crate::articles::component::{ViewProps, ArticleComponentMsg as ParentMsg};
use crate::components::{Dropdown, DropdownLabel, FA, IconType, font_awesome::FAProps};
use crate::timeline::agent::{TimelineAgent, TimelineRequest};
use crate::log_warn;
use crate::services::article_actions::Action;
use crate::settings::ArticleFilteredMode;

pub struct SocialArticle {
	compact: Option<bool>,
	add_timeline_agent: Dispatcher<TimelineAgent>,
}

pub enum SocialArticleMsg {
	ParentCallback(ParentMsg),
	ToggleCompact,
	AddUserTimeline(&'static str, String),
}

type Msg = SocialArticleMsg;

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
				//self.add_timeline_agent.send(TimelineRequest::AddUserTimeline(service, username));
				self.add_timeline_agent.send(TimelineRequest::AddQuickUserTimeline(service, username));
				false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let boxed_refs = &ctx.props().article_struct.boxed_refs;

		let quoted_post = {
			let quoted = boxed_refs.iter().enumerate().find_map(|(i, ref_article)|
				if let ArticleRefType::Quote(q) | ArticleRefType::RepostedQuote(_, q) = ref_article {
					Some((i, q))
				}else {
					None
				}
			);
			if let Some((i, q)) = quoted {
				match &ctx.props().article_struct.boxed.referenced_articles()[i] {
					ArticleRefType::Quote(weak) | ArticleRefType::RepostedQuote(_, weak) => {
						self.view_quoted_post(ctx, &q, weak.clone())
					}
					_ => {
						log::warn!("Boxed quote but no weak reference of it.");
						html! {}
					}
				}
			} else {
				html! {}
			}
		};
		let actual_article = ctx.props().article_struct.boxed_actual_article();
		let actual_weak = ctx.props().article_struct.boxed.actual_article().unwrap_or_else(|| ctx.props().article_struct.weak.clone());

		let on_username_click = {
			let service = actual_article.service();
			let author_username = actual_article.author_username();
			ctx.link().callback(move |e: MouseEvent| {
				e.prevent_default();
				Msg::AddUserTimeline(service, author_username.clone())
			})
		};

		html! {
			<>
				{ self.view_repost_label(ctx) }
				{ self.view_reply_label(ctx) }
				<div class="media">
					{ self.view_avatar(ctx) }
					<div class="media-content">
						<div class="content">
							<div class="articleHeader">
								<a class="names" href={actual_article.author_url()} target="_blank" rel="noopener noreferrer" onclick={on_username_click}>
									<strong>{ actual_article.author_name() }</strong>
									<small>{ format!("@{}", actual_article.author_username()) }</small>
								</a>
								{ self.view_timestamp(&actual_article) }
							</div>
							{ match ctx.props().hide_text || self.is_minimized(ctx) {
								false => html! {<p class="articleParagraph">{ actual_article.view_text() }</p>},
								true => html! {},
							} }
						</div>
						{ quoted_post }
						{ self.view_nav(ctx, &actual_article, actual_weak, false) }
					</div>
				</div>
				{ match self.is_minimized(ctx) {
					false => self.view_media(ctx, &actual_article),
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
		!ctx.props().article_struct.in_section && ctx.props().app_settings.article_filtered_mode == ArticleFilteredMode::Minimized
	}

	fn view_timestamp(&self, actual_article: &ArticleBox) -> Html {
		let label = short_timestamp(&actual_article.creation_time());

		html! {
			<span class="timestamp">
				<small title={actual_article.creation_time().to_string().as_string()}>{ label }</small>
			</span>
		}
	}

	fn view_nav(&self, ctx: &Context<Self>, actual_article: &ArticleBox, actual_weak: ArticleWeak, quote: bool) -> Html {
		let actual_weak_c = actual_weak.clone();
		let actual_weak_c_c = actual_weak.clone();
		let actual_weak_c_c_c = actual_weak.clone();
		let ontoggle_compact = ctx.link().callback(|_| Msg::ToggleCompact);
		//TODO Add weak_actual_article to ArticleStruct?
		let ontoggle_markasread = ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::MarkAsRead, Some(actual_weak_c_c_c.clone()))));
		let dropdown_buttons = match &ctx.props().article_struct.boxed_refs.iter().find(|ref_article| matches!(ref_article, ArticleRefType::Reposted(_) | ArticleRefType::RepostedQuote(_, _))) {
			Some(_) => html! {
				<a
					class="dropdown-item"
					href={ ctx.props().article_struct.boxed.url() }
					target="_blank" rel="noopener noreferrer"
				>
					{ "Repost's External Link" }
				</a>
			},
			None => html! {},
		};

		html! {
			<nav class="level is-mobile">
				<div class="level-left">
					{ match self.is_minimized(ctx) {
						false => html! {
							<>
								<a
									class={classes!("level-item", "articleButton", "repostButton", if actual_article.reposted() { Some("repostedPostButton") } else { None })}
									onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::Repost, Some(actual_weak_c.clone()))))}
								>
									<FA icon="retweet"/>
									{match actual_article.repost_count() {
										0 => html! {},
										count => html! {
											<span>{ count }</span>
										}
									}}
								</a>
								<a
									class={classes!("level-item", "articleButton", "likeButton", if actual_article.liked() { Some("likedPostButton") } else { None })}
									onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::Like, Some(actual_weak_c_c.clone()))))}
								>
									<FA icon="heart" icon_type={if actual_article.liked() { IconType::Solid } else { IconType::Regular }}/>
									{match actual_article.like_count() {
										0 => html! {},
										count => html! {
											<span>{ count }</span>
										}
									}}
								</a>
								{ if !quote {
									match &actual_article.media().iter().map(|m| m.media_type).collect::<Vec<MediaType>>()[..] {
										[MediaType::Image, ..] => html! {
											<a class="level-item articleButton" onclick={&ontoggle_compact}>
												<FA icon={if self.is_compact(ctx) { "compress" } else { "expand" }}/>
											</a>
										},
										_ => html! {},
									}
								} else {
									html! {}
								} }
								{ if !quote {
									html! {
										<a class="level-item articleButton" onclick={&ontoggle_markasread}>
											<FA icon={if actual_article.marked_as_read() { "eye" } else { "eye-slash" }}/>
										</a>
										}
								}else {
									html! {}
								} }
								{ if !quote && !ctx.props().in_modal {
									html! {
										<a class="level-item articleButton" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleInModal))}>
											<FA icon="expand-alt"/>
										</a>
									}
								}else {
									html! {}
								} }
							</>
						},
						true => html! {},
					} }
					{ if !quote {
						let actual_weak_markasred = actual_weak.clone();
						let actual_weak_hide = actual_weak.clone();
						let actual_weak_logdata = actual_weak.clone();
						let actual_weak_logjsondata = actual_weak.clone();
						let actual_weak_fetchdata = actual_weak.clone();
						html! {
							<Dropdown current_label={DropdownLabel::Icon(yew::props! { FAProps {icon: "ellipsis-h".to_owned()}})} trigger_classes={classes!("level-item")} label_classes={classes!("articleButton")}>
								<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::MarkAsRead, Some(actual_weak_markasred.clone()))))}> {"Mark as read"} </a>
								<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::Hide, Some(actual_weak_hide.clone()))))}> {"Hide"} </a>
								<a class="dropdown-item" onclick={&ontoggle_compact}> { if self.is_compact(ctx) { "Show expanded" } else { "Show compact" } } </a>
								<a
									class="dropdown-item"
									href={ actual_article.url() }
									target="_blank" rel="noopener noreferrer"
								>
									{ "External Link" }
								</a>
								{ dropdown_buttons }
								<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::LogData, Some(actual_weak_logdata.clone()))))}>{"Log Data"}</a>
								<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::LogJsonData, Some(actual_weak_logjsondata.clone()))))}>{"Log Json Data"}</a>
								<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::ParentCallback(ParentMsg::Action(Action::FetchData, Some(actual_weak_fetchdata.clone()))))}>{"Fetch Data"}</a>
							</Dropdown>
						}
					}else {
						html! {}
					} }
				</div>
			</nav>
		}
	}

	fn view_media(&self, ctx: &Context<Self>, actual_article: &ArticleBox) -> Html {
		//TODO Show thumbnail if queue_load_info
		let type_src_tuples: Vec<(MediaType, String)> = actual_article.media().iter().map(|m| (m.media_type, m.src.clone())).collect();
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
							[(MediaType::Image, src)] => self.view_image(ctx, actual_article, src.clone(), false),
							[(MediaType::Image, src_1), (MediaType::Image, src_2)] => html! {
								<>
									{ self.view_image(ctx, actual_article, src_1.clone(), false) }
									{ self.view_image(ctx, actual_article, src_2.clone(), false) }
								</>
							},
							[(MediaType::Image, src_1), (MediaType::Image, src_2), (MediaType::Image, src_3)] => html! {
								<>
									{ self.view_image(ctx, actual_article, src_1.clone(), false) }
									{ self.view_image(ctx, actual_article, src_2.clone(), false) }
									{ self.view_image(ctx, actual_article, src_3.clone(), true) }
								</>
							},
							[(MediaType::Image, src_1), (MediaType::Image, src_2), (MediaType::Image, src_3), (MediaType::Image, src_4), ..] => html! {
								<>
									{ self.view_image(ctx, actual_article, src_1.clone(), false) }
									{ self.view_image(ctx, actual_article, src_2.clone(), false) }
									{ self.view_image(ctx, actual_article, src_3.clone(), false) }
									{ self.view_image(ctx, actual_article, src_4.clone(), false) }
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

	fn view_image(&self, ctx: &Context<Self>, actual_article: &ArticleBox, image: String, is_large_third: bool) -> Html {
		let media_holder_classes = classes!(
			"mediaHolder",
			if self.is_compact(ctx) { Some("mediaHolderCompact") } else { None },
			if is_large_third { Some("thirdImage") } else { None },
		);

		html! {
			<div class={media_holder_classes}>
				<div class="is-hidden imgPlaceholder"/>
				<img alt={actual_article.id()} src={image} onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnMediaClick))}/>
			</div>
		}
	}

	fn view_quoted_post(&self, ctx: &Context<Self>, quoted: &ArticleBox, weak_quoted: ArticleWeak) -> Html {
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
				{ self.view_nav(ctx, quoted, weak_quoted, true) }
			</div>
		}
	}

	fn view_repost_label(&self, ctx: &Context<Self>) -> Html {
		let boxed = &ctx.props().article_struct.boxed;
		if boxed.actual_article_index().is_none() {
			return html! {}
		}

		let service = boxed.service();
		let username = boxed.author_username();
		let onclick = ctx.link().callback(move |e: MouseEvent| {
			e.prevent_default();
			Msg::AddUserTimeline(service.clone(), username.clone())
		});
		html! {
			<div class="repostLabel"
				href={boxed.url()}
				target="_blank">
				<a {onclick}>
					{ format!("{} reposted - {}", &boxed.author_name(), short_timestamp(&boxed.creation_time())) }
				</a>
			</div>
		}
	}

	fn view_reply_label(&self, ctx: &Context<Self>) -> Html {
		let boxed = &ctx.props().article_struct.boxed_actual_article();

		let username = if let Some(ArticleRefType::Reply(a)) = boxed.referenced_articles().iter().find(|article_ref| matches!(article_ref, ArticleRefType::Reply(_))) {
			Some(a.upgrade().unwrap().borrow().author_username())
		}else if let Some(username) = boxed.unfetched_references().iter().find_map(|article_ref| {
			if let UnfetchedArticleRef::ReplyToUser(username) = article_ref {
				Some(username)
			} else {
				None
			}
		}) {
			Some(username.clone())
		}else {
			None
		};

		if let Some(username) = username {
			if username == boxed.author_username() {
				html! {
					<div class="replyLabel">
						<a> { "in a thread" } </a>
					</div>
				}
			}else {
				html! {
					<div class="replyLabel">
						<a> { format!("Replying to @{}", username) } </a>
					</div>
				}
			}
		}else {
			html! {}
		}
	}

	fn view_avatar(&self, ctx: &Context<Self>) -> Html {
		let article = &ctx.props().article_struct.boxed;
		let repost = ctx.props().article_struct.boxed_refs.iter().find_map(|ref_article| {
			if let ArticleRefType::Reposted(a) | ArticleRefType::RepostedQuote(a, _) = ref_article {
				Some(a)
			}else {
				None
			}
		});
		match article.author_avatar_url().as_str() {
			"" => html! {},
			url => html! {
				<div class="media-left">
					{ if let Some(a) = repost {
						html! {
							<figure class="image is-64x64 sharedAvatar">
								<img src={a.author_avatar_url().as_str().to_owned()} alt={format!("{}'s avatar", &a.author_username())}/>
								<img src={url.to_owned()} alt={format!("{}'s avatar", &article.author_username())}/>
							</figure>
						}
					}else {
						html! {
							<figure class="image is-64x64">
								<img src={url.to_owned()} alt={format!("{}'s avatar", &article.author_username())}/>
							</figure>
						}
					} }
				</div>
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
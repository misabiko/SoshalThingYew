use yew::prelude::*;
use js_sys::Date;
use std::cell::Ref;
use wasm_bindgen::closure::Closure;
use yew_agent::{Dispatcher, Dispatched};

use crate::articles::{ArticleData, ArticleRefType, ArticleMedia};
use crate::articles::component::{ViewProps, Msg as ParentMsg};
use crate::dropdown::{Dropdown, DropdownLabel};
use crate::timeline::agent::{TimelineAgent, Request as TimelineAgentRequest};
use crate::error::log_warn;

pub struct SocialArticle {
	compact: Option<bool>,
	add_timeline_agent: Dispatcher<TimelineAgent>,
}

pub enum Msg {
	ParentCallback(ParentMsg),
	ToggleCompact,
	AddUserTimeline(String, String),
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
		let strong = ctx.props().article.upgrade().unwrap();
		let borrow = strong.borrow();

		let (actual_article, retweet_header, quoted_post) = match &borrow.referenced_article() {
			ArticleRefType::NoRef => (strong.clone(), html! {}, html! {}),
			ArticleRefType::Repost(a) => (
				a.upgrade().unwrap(),
				view_repost_label(&borrow),
				html! {}
			),
			ArticleRefType::Quote(a) => {
				let quote_article = a.upgrade().unwrap();
				let quote_borrow = quote_article.borrow();
				(strong.clone(), html! {}, self.view_quoted_post(ctx, &quote_borrow))
			}
			ArticleRefType::QuoteRepost(a, q) => {
				let reposted_article = a.upgrade().unwrap();

				let quoted_article = q.upgrade().unwrap();
				let quoted_borrow = quoted_article.borrow();
				(reposted_article.clone(), view_repost_label(&borrow), self.view_quoted_post(ctx, &quoted_borrow))
			}
		};
		let actual_borrow = actual_article.borrow();

		let actual_c = actual_article.clone();
		let on_username_click = ctx.link().callback(move |e: MouseEvent| {
			let borrow = actual_c.borrow();
			e.prevent_default();
			Msg::AddUserTimeline(borrow.service().to_owned(), borrow.author_username())
		});

		html! {
			<article class="article" articleId={borrow.id()} key={borrow.id()} style={ctx.props().style.clone()}>
				{ retweet_header }
				<div class="media">
					<figure class="media-left">
						<p class="image is-64x64">
							{ match actual_borrow.author_avatar_url().as_str() {
								"" => html! {},
								url => html! { <img src={url.to_owned()} alt={format!("{}'s avatar", &actual_borrow.author_username())}/> }
							} }
						</p>
					</figure>
					<div class="media-content">
						<div class="content">
							<div class="articleHeader">
								<a class="names" href={actual_borrow.author_url()} target="_blank" rel="noopener noreferrer" onclick={on_username_click}>
									<strong>{ actual_borrow.author_name() }</strong>
									<small>{ format!("@{}", actual_borrow.author_username()) }</small>
								</a>
								{ self.view_timestamp(&actual_borrow) }
							</div>
							{ match ctx.props().hide_text || self.is_filtered_out(&actual_borrow) {
								false => html! {<p class="articleParagraph">{ actual_borrow.text() }</p>},
								true => html! {},
							} }
						</div>
						{ quoted_post }
						{ self.view_nav(ctx, &actual_borrow) }
					</div>
				</div>
				{ match self.is_filtered_out(&actual_borrow) {
					false => self.view_media(ctx, &actual_borrow),
					true => html! {},
				} }
			</article>
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
							let _ = promise.catch(&Closure::once(Box::new(|err| log_warn(Some("Failed to play video"), err))));
						}
						Err(err) => log_warn(Some("Failed to try and play the video"), err)
					}
				},
				false => {
					video.set_muted(false);
					match video.pause() {
						Err(err) => log_warn(Some("Failed to try and pause the video"), err),
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

	fn is_filtered_out(&self, actual_article: &Ref<dyn ArticleData>) -> bool {
		actual_article.marked_as_read() || actual_article.hidden()
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
		let strong = ctx.props().article.upgrade().unwrap();
		let borrow = strong.borrow();
		let ontoggle_compact = ctx.link().callback(|_| Msg::ToggleCompact);
		let dropdown_buttons = match &borrow.referenced_article() {
			ArticleRefType::NoRef => html! {},
			ArticleRefType::Repost(_) | ArticleRefType::QuoteRepost(_, _) => html! {
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
					{ match self.is_filtered_out(&actual_borrow) {
						false => html! {
							<>
								<a
									class={classes!("level-item", "articleButton", "repostButton", if actual_borrow.reposted() { Some("repostedPostButton") } else { None })}
									onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::Repost))}
								>
									<span class="icon">
										<i class="fas fa-retweet"/>
									</span>
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
									<span class="icon">
										<i class={classes!("fa-heart", if actual_borrow.liked() { "fas" } else { "far" })}/>
									</span>
									{match actual_borrow.like_count() {
										0 => html! {},
										count => html! {
											<span>{ count }</span>
										}
									}}
								</a>
								{
									match &actual_borrow.media()[..] {
										[ArticleMedia::Image(_, _), ..] => html! {
											<a class="level-item articleButton" onclick={&ontoggle_compact}>
												<span class="icon">
													<i class={classes!("fas", if self.is_compact(ctx) { "fa-compress" } else { "fa-expand" })}/>
												</span>
											</a>
										},
										_ => html! {},
									}
								}
								{
									match ctx.props().in_modal {
										false => html! {
											<a class="level-item articleButton" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleInModal))}>
												<span class="icon">
													<i class="fas fa-expand-alt"/>
												</span>
											</a>
										},
										true => html! {},
									}
								}

							</>
						},
						true => html! {},
					} }
					<Dropdown current_label={DropdownLabel::Icon("fas fa-ellipsis-h".to_owned())} label_classes={classes!("articleButton")}>
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleMarkAsRead))}> {"Mark as read"} </div>
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::ToggleHide))}> {"Hide"} </div>
						<div class="dropdown-item" onclick={&ontoggle_compact}> { if self.is_compact(ctx) { "Show expanded" } else { "Show compact" } } </div>
						<a
							class="dropdown-item"
							href={ actual_borrow.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						{ dropdown_buttons }
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::LogData))}>{"Log Data"}</div>
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::FetchData))}>{"Fetch Data"}</div>
					</Dropdown>
				</div>
			</nav>
		}
	}

	fn view_media(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>) -> Html {
		match (&ctx.props().animated_as_gifs, &actual_borrow.media()[..]) {
			(_, [ArticleMedia::Image(_, _), ..]) => {
				let images_classes = classes!(
						"postMedia",
						"postImages",
						if self.is_compact(ctx) { Some("postImagesCompact") } else { None }
					);

				html! {
					<div class={images_classes.clone()}> {
						match &actual_borrow.media()[..] {
							[ArticleMedia::Image(image, _)] => self.view_image(ctx, actual_borrow, image.clone(), false),
							[ArticleMedia::Image(i1, _), ArticleMedia::Image(i2, _)] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
								</>
							},
							[ArticleMedia::Image(i1, _), ArticleMedia::Image(i2, _), ArticleMedia::Image(i3, _)] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i3.clone(), true) }
								</>
							},
							[ArticleMedia::Image(i1, _), ArticleMedia::Image(i2, _), ArticleMedia::Image(i3, _), ArticleMedia::Image(i4, _), ..] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i3.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i4.clone(), false) }
								</>
							},
							_ => html! {{"unexpected media format"}}
						}
					} </div>
				}
			}
			(false, [ArticleMedia::Video(video_src, _)]) => html! {
				<div class="postMedia postVideo">
					<video ref={ctx.props().video_ref.clone()} controls=true onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnImageClick))}>
						<source src={video_src.clone()} type="video/mp4"/>
					</video>
				</div>
			},
			(_, [ArticleMedia::VideoGif(gif_src, _)]) | (true, [ArticleMedia::Video(gif_src, _)]) => html! {
				<div class="postMedia postVideo">
					<video ref={ctx.props().video_ref.clone()} controls=true autoplay=true loop=true muted=true onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnImageClick))}>
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
				<img alt={actual_borrow.id()} src={image} onclick={ctx.link().callback(|_| Msg::ParentCallback(ParentMsg::OnImageClick))}/>
			</div>
		}
	}

	fn view_quoted_post(&self, ctx: &Context<SocialArticle>, quoted: &Ref<dyn ArticleData>) -> Html {
		html! {
			<div class="quotedPost">
				<div class="articleHeader">
					<a class="names" href={quoted.author_url()} target="_blank" rel="noopener noreferrer">
						<strong>{ quoted.author_name() }</strong>
						<small>{ format!("@{}", quoted.author_username()) }</small>
					</a>
					{ self.view_timestamp(&quoted) }
				</div>
				{ match self.is_filtered_out(&quoted) {
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
}

fn view_repost_label(repost: &Ref<dyn ArticleData>) -> Html {
	html! {
		<div class="repostLabel"
			href={repost.url()}
			target="_blank">
			<a>{ format!("{} reposted - {}", &repost.author_name(), short_timestamp(&repost.creation_time())) }</a>
		</div>
	}
}

fn short_timestamp(date: &Date) -> String {
	let time_since = Date::now() - date.get_time();

	if time_since < 1000.0 {
		"just now".to_owned()
	} else if time_since < 60000.0 {
		format!("{}s", (time_since / 1000.0).floor())
	} else if time_since < 3600000.0 {
		format!("{}m", (time_since / 60000.0).floor())
	} else if time_since < 86400000.0 {
		format!("{}h", (time_since / (3600000.0)).floor())
	} else if time_since < 604800000.0 {
		format!("{}d", (time_since / (86400000.0)).floor())
	} else {
		//format!("{} {}", monthAbbrevs[actualDate.getMonth()], actualDate.getDate())
		//TODO Parse month timestamp
		"long ago".to_owned()
	}
}
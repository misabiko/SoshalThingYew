use yew::prelude::*;
use js_sys::Date;
use wasm_bindgen::JsValue;
use web_sys::console;
use std::cell::Ref;
use yew_agent::{Bridge, Bridged};

use crate::articles::{ArticleData, ArticleRefType, Props, ArticleMedia};
use crate::dropdown::{Dropdown, DropdownLabel};
use crate::services::article_actions::{ArticleActionsAgent, Request as ArticleActionsRequest, Response as ArticleActionsResponse};

pub struct SocialArticle {
	compact: Option<bool>,
	article_actions: Box<dyn Bridge<ArticleActionsAgent>>,
}

pub enum Msg {
	ToggleCompact,
	OnImageClick,
	LogData,
	Like,
	Repost,
	ToggleMarkAsRead,
	ToggleHide,
	ActionsCallback(ArticleActionsResponse),
}

impl Component for SocialArticle {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			compact: None,
			article_actions: ArticleActionsAgent::bridge(ctx.link().callback(Msg::ActionsCallback)),
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
			}
			Msg::OnImageClick => {
				ctx.link().send_message(Msg::ToggleMarkAsRead);
				false
			}
			Msg::LogData => {
				let strong = ctx.props().data.upgrade().unwrap();
				console::dir_1(&JsValue::from_serde(&strong.borrow().json()).unwrap_or_default());
				false
			}
			Msg::Like => {
				let strong = ctx.props().data.upgrade().unwrap();
				let borrow = strong.borrow();

				self.article_actions.send(ArticleActionsRequest::Like(match borrow.referenced_article() {
					ArticleRefType::NoRef => ctx.props().data.clone(),
					ArticleRefType::Repost(a) => a,
					ArticleRefType::Quote(_) => ctx.props().data.clone(),
				}));
				false
			}
			Msg::Repost => {
				let strong = ctx.props().data.upgrade().unwrap();
				let borrow = strong.borrow();

				self.article_actions.send(ArticleActionsRequest::Repost(match borrow.referenced_article() {
					ArticleRefType::NoRef => ctx.props().data.clone(),
					ArticleRefType::Repost(a) => a,
					ArticleRefType::Quote(_) => ctx.props().data.clone(),
				}));
				false
			}
			Msg::ToggleMarkAsRead => {
				let strong = ctx.props().data.upgrade().unwrap();
				let mut borrow = strong.borrow_mut();

				match borrow.referenced_article() {
					ArticleRefType::NoRef => {
						let marked_as_read = borrow.marked_as_read();
						borrow.set_marked_as_read(!marked_as_read);
						self.article_actions.send(ArticleActionsRequest::MarkAsRead(ctx.props().data.clone(), !marked_as_read));
					},
					ArticleRefType::Repost(a) | ArticleRefType::Quote(a) => {
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
				let strong = ctx.props().data.upgrade().unwrap();
				let mut borrow = strong.borrow_mut();

				match borrow.referenced_article() {
					ArticleRefType::NoRef => {
						let hidden = borrow.hidden();
						borrow.set_hidden(!hidden);
					},
					ArticleRefType::Repost(a) | ArticleRefType::Quote(a) => {
						let strong = a.upgrade().unwrap();
						let mut borrow = strong.borrow_mut();

						let hidden = borrow.hidden();
						borrow.set_hidden(!hidden);
					}
				};

				true
			}
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

		let (actual_article, retweet_header, quoted_post) = match &borrow.referenced_article() {
			ArticleRefType::NoRef => (strong.clone(), html! {}, html! {}),
			ArticleRefType::Repost(a) => (
				a.upgrade().unwrap(),
				html! {
					<div class="repostLabel"
						href={borrow.url()}
						target="_blank">
						<a>{ format!("{} reposted", &borrow.author_name()) }</a>
					</div>
				},
				html! {}
			),
			ArticleRefType::Quote(a) => {
				let quote_article = a.upgrade().unwrap();
				let quote_borrow = quote_article.borrow();
				(strong.clone(), html! {}, html! {
					<div class="quotedPost">
						<div class="articleHeader">
							<a class="names" href={quote_borrow.author_url()} target="_blank" rel="noopener noreferrer">
								<strong>{ quote_borrow.author_name() }</strong>
								<small>{ format!("@{}", quote_borrow.author_username()) }</small>
							</a>
							{ self.view_timestamp(ctx, &quote_borrow) }
						</div>
						{ match self.is_filtered_out(&quote_borrow) {
							false => html! {
								<>
									<p class="refArticleParagraph">{quote_borrow.text()}</p>
									{ self.view_media(ctx, &quote_borrow) }
								</>
							},
							true => html! {},
						} }
					</div>
				})
			}
		};
		let actual_borrow = actual_article.borrow();

		html! {
			<article class="article" articleId={borrow.id()} style={ctx.props().style.clone()}>
				{ retweet_header }
				<div class="media">
					<figure class="media-left">
						<p class="image is-64x64">
							<img src={actual_borrow.author_avatar_url().clone()} alt={format!("{}'s avatar", &actual_borrow.author_username())}/>
						</p>
					</figure>
					<div class="media-content">
						<div class="content">
							<div class="articleHeader">
								<a class="names" href={actual_borrow.author_url()} target="_blank" rel="noopener noreferrer">
									<strong>{ actual_borrow.author_name() }</strong>
									<small>{ format!("@{}", actual_borrow.author_username()) }</small>
								</a>
								{ self.view_timestamp(ctx, &actual_borrow) }
							</div>
							{ match self.is_filtered_out(&actual_borrow) {
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

	fn view_timestamp(&self, _ctx: &Context<Self>, actual_article: &Ref<dyn ArticleData>) -> Html {
		let time_since = Date::now() - actual_article.creation_time().get_time();
		let label = if time_since < 1000.0 {
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
		};

		html! {
			<span class="timestamp">
				<small title={actual_article.creation_time().to_string().as_string()}>{ label }</small>
			</span>
		}
	}

	fn view_nav(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>) -> Html {
		let strong = ctx.props().data.upgrade().unwrap();
		let borrow = strong.borrow();
		let ontoggle_compact = ctx.link().callback(|_| Msg::ToggleCompact);
		let dropdown_buttons = match &borrow.referenced_article() {
			ArticleRefType::NoRef => html! {},
			ArticleRefType::Repost(_) => html! {
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
									onclick={ctx.link().callback(|_| Msg::Repost)}
								>
									<span class="icon">
										<i class="fas fa-retweet"/>
									</span>
									<span>{actual_borrow.repost_count()}</span>
								</a>
								<a
									class={classes!("level-item", "articleButton", "likeButton", if actual_borrow.liked() { Some("likedPostButton") } else { None })}
									onclick={ctx.link().callback(|_| Msg::Like)}
								>
									<span class="icon">
										<i class={classes!("fa-heart", if actual_borrow.liked() { "fas" } else { "far" })}/>
									</span>
									<span>{actual_borrow.like_count()}</span>
								</a>
								{
									match actual_borrow.media().len() {
										0 => html! {},
										_ => html! {
											<a class="level-item articleButton" onclick={&ontoggle_compact}>
												<span class="icon">
													<i class={classes!("fas", if self.is_compact(ctx) { "fa-compress" } else { "fa-expand" })}/>
												</span>
											</a>
										}
									}
								}
							</>
						},
						true => html! {},
					} }
					<Dropdown current_label={DropdownLabel::Icon("fas fa-ellipsis-h".to_owned())} label_classes={classes!("articleButton")}>
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ToggleMarkAsRead)}> {"Mark as read"} </div>
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ToggleHide)}> {"Hide"} </div>
						<div class="dropdown-item" onclick={&ontoggle_compact}> { if self.is_compact(ctx) { "Show expanded" } else { "Show compact" } } </div>
						<div class="dropdown-item"> {"Fetch Status"} </div>
						<div class="dropdown-item"> {"Expand"} </div>
						<a
							class="dropdown-item"
							href={ actual_borrow.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						{ dropdown_buttons }
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::LogData)}>{"Log Data"}</div>
					</Dropdown>
				</div>
			</nav>
		}
	}

	fn view_media(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>) -> Html {
		match &actual_borrow.media()[..] {
			[ArticleMedia::Image(_), ..] => {
				let images_classes = classes!(
						"postMedia",
						"postImages",
						if self.is_compact(ctx) { Some("postImagesCompact") } else { None }
					);

				html! {
					<div class={images_classes.clone()}> {
						match &actual_borrow.media()[..] {
							[ArticleMedia::Image(image)] => self.view_image(ctx, actual_borrow, image.clone(), false),
							[ArticleMedia::Image(i1), ArticleMedia::Image(i2)] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
								</>
							},
							[ArticleMedia::Image(i1), ArticleMedia::Image(i2), ArticleMedia::Image(i3)] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i3.clone(), true) }
								</>
							},
							[ArticleMedia::Image(i1), ArticleMedia::Image(i2), ArticleMedia::Image(i3), ArticleMedia::Image(i4), ..] => html! {
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
			[ArticleMedia::Video(video_src)] => html! {
				<div class="postMedia postVideo">
					<video controls=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
						<source src={video_src.clone()} type="video/mp4"/>
					</video>
				</div>
			},
			[ArticleMedia::Gif(gif_src)] => html! {
				<div class="postMedia postVideo">
					<video controls=true autoplay=true loop=true muted=true onclick={ctx.link().callback(|_| Msg::OnImageClick)}>
						<source src={gif_src.clone()} type="video/mp4"/>
					</video>
				</div>
			},
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
				<img alt={actual_borrow.id()} src={image} onclick={ctx.link().callback(|_| Msg::OnImageClick)}/>
			</div>
		}
	}
}
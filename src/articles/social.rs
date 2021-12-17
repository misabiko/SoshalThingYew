use yew::prelude::*;
use js_sys::Date;
use wasm_bindgen::JsValue;
use web_sys::console;
use std::rc::Rc;
use std::cell::Ref;
use yew_agent::{Bridge, Bridged};

use crate::articles::{ArticleData, Props};
use crate::dropdown::{Dropdown, DropdownLabel};
use crate::services::article_actions::{ArticleActionsAgent, Request as ArticleActionsRequest, Response as ArticleActionsResponse};

pub struct SocialArticle {
	compact: Option<bool>,
	article_actions: Box<dyn Bridge<ArticleActionsAgent>>
}

pub enum Msg {
	ToggleCompact,
	OnImageClick,
	LogData,
	Like,
	Repost,
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
			},
			Msg::OnImageClick => {
				ctx.link().send_message(Msg::ToggleCompact);
				false
			},
			Msg::LogData => {
				let strong = ctx.props().data.upgrade().unwrap();
				console::dir_1(&JsValue::from_serde(&strong.borrow().json()).unwrap_or_default());
				false
			},
			Msg::Like => {
				let strong = ctx.props().data.upgrade().unwrap();
				let actual_article = strong.borrow().referenced_article().and_then(|w| w.upgrade()).unwrap_or_else(|| strong.clone());
				self.article_actions.send(ArticleActionsRequest::Like(Rc::downgrade(&actual_article)));
				false
			}
			Msg::Repost => {
				let strong = ctx.props().data.upgrade().unwrap();
				let actual_article = strong.borrow().referenced_article().and_then(|w| w.upgrade()).unwrap_or_else(|| strong.clone());
				self.article_actions.send(ArticleActionsRequest::Repost(Rc::downgrade(&actual_article)));
				false
			}
			Msg::ActionsCallback(_) => true
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let strong = ctx.props().data.upgrade().unwrap();
		let borrow = strong.borrow();
		let actual_article = borrow.referenced_article().and_then(|w| w.upgrade()).unwrap_or_else(|| strong.clone());
		let actual_borrow = actual_article.borrow();

		let is_retweet = borrow.referenced_article().is_some();
		let retweet_header = match &is_retweet {
			true => html! {
				<div class="repostLabel"
					href={borrow.url()}
					target="_blank">
					<a>{ format!("{} reposted", &borrow.author_name()) }</a>
				</div>
			},
			false => html! {}
		};

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
							<p class="articleParagraph">{ actual_borrow.text() }</p>
						</div>
						{ self.view_nav(ctx, &actual_borrow) }
					</div>
				</div>
				{ self.view_media(ctx, &actual_borrow) }
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
		let is_retweet = borrow.referenced_article().is_some();

		html! {
			<nav class="level is-mobile">
				<div class="level-left">
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
					<Dropdown current_label={DropdownLabel::Icon("fas fa-ellipsis-h".to_owned())} label_classes={classes!("articleButton")}>
						<div class="dropdown-item"> {"Mark as red"} </div>
						<div class="dropdown-item"> {"Hide"} </div>
						<div class="dropdown-item" onclick={&ontoggle_compact}> { if self.is_compact(ctx) { "Show expanded" } else { "Show compact" } } </div>
						<div class="dropdown-item"> {"Log"} </div>
						<div class="dropdown-item"> {"Fetch Status"} </div>
						<div class="dropdown-item"> {"Expand"} </div>
						<a
							class="dropdown-item"
							href={ actual_borrow.url() }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
						{
							match &is_retweet {
								true => html! {
									<a
										class="dropdown-item"
										href={ borrow.url() }
										target="_blank" rel="noopener noreferrer"
									>
										{ "Repost's External Link" }
									</a>
								},
								false => html! {}
							}
						}
						<div class="dropdown-item" onclick={ctx.link().callback(|_| Msg::LogData)}>{"Log Data"}</div>
					</Dropdown>
				</div>
			</nav>
		}
	}

	fn view_media(&self, ctx: &Context<Self>, actual_borrow: &Ref<dyn ArticleData>) -> Html {
		let images_classes = classes!(
			"postMedia",
			"postImages",
			if self.is_compact(ctx) { Some("postImagesCompact") } else { None }
		);

		if actual_borrow.media().len() == 0 {
			html! {}
		} else {
			html! {
				<div>
					<div class={images_classes.clone()}> {
						match &actual_borrow.media()[..] {
							[image] => self.view_image(ctx, actual_borrow, image.clone(), false),
							[i1, i2] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
								</>
							},
							[i1, i2, i3] => html! {
								<>
									{ self.view_image(ctx, actual_borrow, i1.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i2.clone(), false) }
									{ self.view_image(ctx, actual_borrow, i3.clone(), true) }
								</>
							},
							_ => html! {
								<>
									{ self.view_image(ctx, actual_borrow, actual_borrow.media()[0].clone(), false) }
									{ self.view_image(ctx, actual_borrow, actual_borrow.media()[1].clone(), false) }
									{ self.view_image(ctx, actual_borrow, actual_borrow.media()[2].clone(), false) }
									{ self.view_image(ctx, actual_borrow, actual_borrow.media()[3].clone(), false) }
								</>
							}
						}
					} </div>
				</div>
			}
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
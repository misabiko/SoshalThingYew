use std::rc::Rc;
use yew::prelude::*;
use js_sys::Date;

pub struct SocialArticle {
	compact: Option<bool>,
	show_dropdown: bool,
}

#[derive(Properties, Clone)]
pub struct Props {
	#[prop_or_default]
	pub compact: bool,
	#[prop_or_default]
	pub style: Option<String>,
	pub data: Rc<dyn SocialArticleData>,
}

impl PartialEq<Props> for Props {
	fn eq(&self, other: &Props) -> bool {
		self.compact == other.compact &&
		self.style == other.style &&
		&self.data == &other.data
	}
}

pub enum Msg {
	ToggleCompact,
	ToggleDropdown,
	OnImageClick,
}

pub trait SocialArticleData {
	fn id(&self) -> String;
	fn creation_time(&self) -> Date;
	fn text(&self) -> String;
	fn author_username(&self) -> String;
	fn author_name(&self) -> String;
	fn author_avatar_url(&self) -> String;
	fn author_url(&self) -> String;
	fn like_count(&self) -> i64 { 0 }
	fn repost_count(&self) -> i64 { 0 }
	fn liked(&self) -> bool { false }
	fn reposted(&self) -> bool { false }

	fn media(&self) -> Vec<String>;
}

impl PartialEq<dyn SocialArticleData> for dyn SocialArticleData {
	fn eq(&self, other: &dyn SocialArticleData) -> bool {
		self.id() == other.id() &&
			self.text() == other.text() &&
			self.author_username() == other.author_username() &&
			self.author_name() == other.author_name() &&
			self.author_avatar_url() == other.author_avatar_url() &&
			self.author_url() == other.author_url() &&
			self.media() == other.media()
	}
}

impl SocialArticle {
	fn is_compact(&self, ctx: &Context<Self>) -> bool {
		match self.compact {
			Some(compact) => compact,
			None => ctx.props().compact,
		}
	}

	fn view_timestamp(&self, ctx: &Context<Self>) -> Html {
		let time_since = Date::now() - ctx.props().data.creation_time().get_time();
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
				<small title={ctx.props().data.creation_time().to_string().as_string()}>{ label }</small>
			</span>
		}
	}

	fn view_nav(&self, ctx: &Context<Self>) -> Html {
		let ontoggle_compact = ctx.link().callback(|_| Msg::ToggleCompact);

		html! {
			<nav class="level is-mobile">
				<div class="level-left">
					<a class={classes!("level-item", "articleButton", "repostButton", if ctx.props().data.reposted() { Some("repostedPostButton") } else { None })}>
						<span class="icon">
							<i class="fas fa-retweet"/>
						</span>
						<span>{ctx.props().data.repost_count()}</span>
					</a>
					<a class={classes!("level-item", "articleButton", "likeButton", if ctx.props().data.liked() { Some("likedPostButton") } else { None })}>
						<span class="icon">
							<i class={classes!("fa-heart", if ctx.props().data.liked() { "fas" } else { "far" })}/>
						</span>
						<span>{ctx.props().data.like_count()}</span>
					</a>
					{
						match ctx.props().data.media().len() {
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
					<div class={classes!("dropdown", if self.show_dropdown { Some("is-active") } else { None })}>
						<div class="dropdown-trigger">
							<a class="level-item articleButton" onclick={ctx.link().callback(|_| Msg::ToggleDropdown)}>
								<span class="icon">
									<i class="fas fa-ellipsis-h"/>
								</span>
							</a>
						</div>
						<div class="dropdown-menu">
							<div class="dropdown-content">
								<div class="dropdown-item"> {"Mark as red"} </div>
								<div class="dropdown-item"> {"Hide"} </div>
								<div class="dropdown-item" onclick={&ontoggle_compact}> { if self.is_compact(ctx) { "Show expanded" } else { "Show compact" } } </div>
								<div class="dropdown-item"> {"Log"} </div>
								<div class="dropdown-item"> {"Fetch Status"} </div>
								<div class="dropdown-item"> {"Expand"} </div>
								<a
									class="dropdown-item"
									href={ format!("https://twitter.com/{}/status/{}", &ctx.props().data.author_username(), &ctx.props().data.id()) }
									target="_blank" rel="noopener noreferrer"
								>
									{ "External Link" }
								</a>
							</div>
						</div>
					</div>
				</div>
			</nav>
		}
	}

	fn view_media(&self, ctx: &Context<Self>) -> Html {
		let images_classes = classes!(
			"postMedia",
			"postImages",
			if self.is_compact(ctx) { Some("postImagesCompact") } else { None }
		);

		if ctx.props().data.media().len() == 0 {
			html! {}
		} else {
			html! {
				<div>
					<div class={images_classes.clone()}> {
						match &ctx.props().data.media()[..] {
							[image] => self.view_image(ctx, image.clone(), false),
							[i1, i2] => html! {
								<>
									{ self.view_image(ctx, i1.clone(), false) }
									{ self.view_image(ctx, i2.clone(), false) }
								</>
							},
							[i1, i2, i3] => html! {
								<>
									{ self.view_image(ctx, i1.clone(), false) }
									{ self.view_image(ctx, i2.clone(), false) }
									{ self.view_image(ctx, i3.clone(), true) }
								</>
							},
							_ => html! {
								<>
									{ self.view_image(ctx, ctx.props().data.media()[0].clone(), false) }
									{ self.view_image(ctx, ctx.props().data.media()[1].clone(), false) }
									{ self.view_image(ctx, ctx.props().data.media()[2].clone(), false) }
									{ self.view_image(ctx, ctx.props().data.media()[3].clone(), false) }
								</>
							}
						}
					} </div>
				</div>
			}
		}
	}

	fn view_image(&self, ctx: &Context<Self>, image: String, is_large_third: bool) -> Html {
		let media_holder_classes = classes!(
			"mediaHolder",
			if self.is_compact(ctx) { Some("mediaHolderCompact") } else { None },
			if is_large_third { Some("thirdImage") } else { None },
		);

		html! {
			<div class={media_holder_classes}>
				<div class="is-hidden imgPlaceholder"/>
				<img alt={ctx.props().data.id()} src={image} onclick={ctx.link().callback(|_| Msg::OnImageClick)}/>
			</div>
		}
	}
}

impl Component for SocialArticle {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			compact: None,
			show_dropdown: false
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleCompact => match self.compact {
				Some(compact) => self.compact = Some(!compact),
				None => self.compact = Some(!ctx.props().compact),
			},
			Msg::ToggleDropdown => self.show_dropdown = !self.show_dropdown,
			Msg::OnImageClick => ctx.link().send_message(Msg::ToggleCompact)
		};

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<article class="article" articleId={ctx.props().data.id()} style={ctx.props().style.clone()}>
				<div class="media">
					<figure class="media-left">
						<p class="image is-64x64">
							<img src={ctx.props().data.author_avatar_url().clone()} alt={format!("{}'s avatar", &ctx.props().data.author_username())}/>
						</p>
					</figure>
					<div class="media-content">
						<div class="content">
							<div class="articleHeader">
								<a class="names" href={ctx.props().data.author_url()} target="_blank" rel="noopener noreferrer">
									<strong>{ ctx.props().data.author_name() }</strong>
									<small>{ format!("@{}", ctx.props().data.author_username()) }</small>
								</a>
								{ self.view_timestamp(ctx) }
							</div>
							<p class="articleParagraph">{ ctx.props().data.text() }</p>
						</div>
						{ self.view_nav(ctx) }
					</div>
				</div>
				{ self.view_media(ctx) }
			</article>
		}
	}
}

pub fn sort_by_id(a: &Rc<dyn SocialArticleData>, b: &Rc<dyn SocialArticleData>) -> std::cmp::Ordering {
	b.id().partial_cmp(&a.id()).unwrap()
}
use std::rc::Rc;
use yew::prelude::*;

pub struct SocialArticle {
	link: ComponentLink<Self>,
	props: Props,
	compact: Option<bool>,
}

#[derive(Properties, Clone)]
pub struct Props {
	#[prop_or_default]
	pub compact: bool,
	pub data: Rc<dyn SocialArticleData>,
}

pub enum Msg {
	ToggleCompact,
}

pub trait SocialArticleData {
	fn id(&self) -> String;
	fn text(&self) -> String;
	fn author_username(&self) -> String;
	fn author_name(&self) -> String;
	fn author_avatar_url(&self) -> String;
	fn author_url(&self) -> String;

	fn media(&self) -> Vec<String>;
}

impl SocialArticle {
	fn is_compact(&self) -> bool {
		match self.compact {
			Some(compact) => compact,
			None => self.props.compact,
		}
	}

	fn view_nav(&self) -> Html {
		let ontoggle_compact = self.link.callback(|_| Msg::ToggleCompact);

		html! {
			<nav class="level is-mobile">
				<div class="level-left">
					<a class="level-item articleButton repostButton">
						<span class="icon">
							<i class="fas fa-retweet"/>
						</span>
						<span>{"0"}</span>
					</a>
					<a class="level-item articleButton likeButton">
						<span class="icon">
							<i class="far fa-heart"/>
						</span>
						<span>{"0"}</span>
					</a>
					{
						match self.props.data.media().len() {
							0 => html! {},
							_ => html! {
								<a class="level-item articleButton" onclick=&ontoggle_compact>
									<span class="icon">
										<i class=classes!("fas", if self.is_compact() { "fa-compress" } else { "fa-expand" })/>
									</span>
								</a>
							}
						}
					}
					<ybc::Dropdown
						button_classes=classes!("level-item", "articleButton")
						button_html=html! {
							<span class="icon">
								<i class="fas fa-ellipsis-h"/>
							</span>
						}
					>
						<div class="dropdown-item"> {"Mark as red"} </div>
						<div class="dropdown-item"> {"Hide"} </div>
						<div class="dropdown-item" onclick=&ontoggle_compact> { if self.is_compact() { "Show expanded" } else { "Show compact" } } </div>
						<div class="dropdown-item"> {"Log"} </div>
						<div class="dropdown-item"> {"Fetch Status"} </div>
						<div class="dropdown-item"> {"Expand"} </div>
						<a
							class="dropdown-item"
							href={ format!("https://twitter.com/{}/status/{}", &self.props.data.author_username(), &self.props.data.id()) }
							target="_blank" rel="noopener noreferrer"
						>
							{ "External Link" }
						</a>
					</ybc::Dropdown>
				</div>
			</nav>
		}
	}

	fn view_media(&self) -> Html {
		let images_classes = classes!(
			"postMedia",
			"postImages",
			if self.is_compact() { Some("postImagesCompact") } else { None }
		);

		if self.props.data.media().len() == 0 {
			html! {}
		} else {
			html! {
				<div>
					<div class=images_classes.clone()> {
						match &self.props.data.media()[..] {
							[image] => self.view_image(image.clone(), false),
							[i1, i2] => html! {
								<>
									{ self.view_image(i1.clone(), false) }
									{ self.view_image(i2.clone(), false) }
								</>
							},
							[i1, i2, i3] => html! {
								<>
									{ self.view_image(i1.clone(), false) }
									{ self.view_image(i2.clone(), false) }
									{ self.view_image(i3.clone(), true) }
								</>
							},
							_ => html! {
								<>
									{ self.view_image(self.props.data.media()[0].clone(), false) }
									{ self.view_image(self.props.data.media()[1].clone(), false) }
									{ self.view_image(self.props.data.media()[2].clone(), false) }
									{ self.view_image(self.props.data.media()[3].clone(), false) }
								</>
							}
						}
					} </div>
				</div>
			}
		}
	}

	fn view_image(&self, image: String, is_large_third: bool) -> Html {
		let media_holder_classes = classes!(
			"mediaHolder",
			if self.is_compact() { Some("mediaHolderCompact") } else { None },
			if is_large_third { Some("thirdImage") } else { None },
		);

		html! {
			<div class=media_holder_classes>
				<div class="is-hidden imgPlaceholder"/>
				<img alt=self.props.data.id() src=image/>
			</div>
		}
	}
}

impl Component for SocialArticle {
	type Message = Msg;
	type Properties = Props;

	fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
		Self {
			link,
			props,
			compact: None,
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::ToggleCompact => match self.compact {
				Some(compact) => self.compact = Some(!compact),
				None => self.compact = Some(!self.props.compact),
			}
		};

		true
	}

	fn change(&mut self, props: Self::Properties) -> ShouldRender {
		if props.compact != self.props.compact {
			self.props.compact = props.compact;
			true
		}else {
			false
		}
	}

	fn view(&self) -> Html {
		html! {
			<article class="article">
				<div class="media">
					<figure class="media-left">
						<p class="image is-64x64">
							<img src=self.props.data.author_avatar_url().clone() alt=format!("{}'s avatar", &self.props.data.author_username())/>
						</p>
					</figure>
					<div class="media-content">
						<div class="content">
							<div class="articleHeader">
								<a class="names" href=self.props.data.author_url() target="_blank" rel="noopener noreferrer">
									<strong>{ self.props.data.author_name() }</strong>
									<small>{ format!("@{}", self.props.data.author_username()) }</small>
								</a>
								<span class="timestamp">
									<small title="'actualArticle.creationDate'">{ "just now" }</small>
								</span>
							</div>
							<p class="articleParagraph">{ self.props.data.text() }</p>
						</div>
						{ self.view_nav() }
					</div>
				</div>
				{ self.view_media() }
			</article>
		}
	}
}
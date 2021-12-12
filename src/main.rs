use std::rc::Rc;
use yew::prelude::*;
use yew::agent::{Dispatched, Dispatcher};

pub mod timeline;
pub mod articles;
pub mod endpoints;
pub mod twitter;
pub mod pixiv;
mod favviewer;

use crate::timeline::{Timeline};
use crate::articles::SocialArticleData;
use crate::endpoints::{EndpointId, TimelineEndpoints};
use crate::favviewer::FavViewer;
use crate::twitter::{TwitterAgent, Response as TwitterResponse};
use crate::pixiv::PixivAgent;

struct Sidebar;

impl Component for Sidebar {
	type Message = ();
	type Properties = ();

	fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
		Self {}
	}

	fn update(&mut self, _msg: Self::Message) -> ShouldRender {
		false
	}

	fn change(&mut self, _props: Self::Properties) -> ShouldRender {
		false
	}

	//TODO Fix top button click moving bottom ones
	fn view(&self) -> Html {
		html! {
			<nav id="sidebar">
				<div id="sidebarButtons">
					<div>
						<button title="Expand sidebar">
							<span class="icon">
								<i class="fas fa-angle-double-right fa-2x"/>
							</span>
						</button>
						<button title="Add new timeline">
							<span class="icon">
								<i class="fas fa-plus fa-2x"/>
							</span>
						</button>
					</div>
					<div/>
				</div>
			</nav>
		}
	}
}

struct Model {
	//link: ComponentLink<Self>,
	boot_articles: Option<Vec<Rc<dyn SocialArticleData>>>,
	twitter: Box<dyn Bridge<TwitterAgent>>,
	pixiv: Dispatcher<PixivAgent>,
	default_endpoint: Option<EndpointId>,
}

enum Msg {
	FetchedBootArticles(Vec<Rc<dyn SocialArticleData>>),
	FailedToFetch,
	TwitterResponse(TwitterResponse),
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
		Self {
			//link,
			boot_articles: None,
			twitter: TwitterAgent::bridge(link.callback(Msg::TwitterResponse)),
			pixiv: PixivAgent::dispatcher(),
			default_endpoint: None,
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::FetchedBootArticles(articles) => {
				match &mut self.boot_articles {
					Some(boot_articles) => boot_articles.extend(articles),
					None => {
						self.boot_articles = Some(articles)
					}
				};

				true
			},
			Msg::FailedToFetch => false,
			Msg::TwitterResponse(r) => {
				match r {
					TwitterResponse::DefaultEndpoint(e) => {
						self.default_endpoint = Some(e);
						true
					}
				}
			}
		}
	}

	fn change(&mut self, _props: Self::Properties) -> ShouldRender {
		false
	}

	fn view(&self) -> Html {
		let pathname_opt = match yew::web_sys::window().map(|w| w.location()) {
			Some(location) => match location.pathname() {
				Ok(pathname_opt) => Some(pathname_opt),
				Err(_) => None
			},
			None => None,
		};
		let boot_timeline = match &self.boot_articles {
			Some(articles) => html!{ <Timeline name="Boot Articles" articles=articles.clone()/>},
			None => {
				/*match pathname_opt.as_ref() {
					Some(pathname) => if let Some(id) = pathname.strip_prefix("/twitter/status/").map(str::to_owned) {
						self.link.send_future(async move {
							match fetch_tweet(&id).await {
								Ok(tweet) => Msg::FetchedBootArticles(vec!(tweet)),
								Err(err) => {
									log::error!("Failed to fetch \"{}\"\n{:?}", &id, err);
									Msg::FailedToFetch
								}
							}
						});
					} else if let Some(username) = pathname.strip_prefix("/twitter/user/").map(str::to_owned) {
						self.link.send_future(async move {
							let url = format!("/proxy/twitter/user/{}", &username);
							match fetch_tweets(&url).await {
								Ok(vec_tweets) => Msg::FetchedBootArticles(vec_tweets),
								Err(err) => {
									log::error!("Failed to fetch \"{}\"\n{:?}", &url, err);
									Msg::FailedToFetch
								}
							}
						});
					},
					None => {}
				};*/
				html!{}
			}
		};

		let home_timeline = match self.default_endpoint {
			Some(e) => html! { <Timeline name="Home" endpoints={TimelineEndpoints {
				start: vec![e],
				refresh: vec![e],
			}}/> },
			None => html! {},
		};

		html! {
			<>
				<Sidebar/>
				<div id="timelineContainer">
					{ boot_timeline }
					{ home_timeline }
				</div>
			</>
		}
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

	match yew::web_sys::window()
		.map(|w| w.location())
		.map(|l| l.href()) {
		Some(Ok(href)) => match href.as_str() {
			"https://www.pixiv.net/bookmark_new_illust.php" => {
				yew::initialize();
				let element = yew::utils::document()
					.query_selector("#root > div:last-child > div:nth-child(2)")
					.expect("can't get mount node for rendering")
					.expect("can't unwrap mount node");
				App::<FavViewer>::new().mount(element);
				yew::run_loop();
			}
			_ => yew::start_app::<Model>(),
		},
		None => log::error!("Failed to get location.href."),
		Some(Err(err)) => log::error!("Failed to get location.href.\n{}", &err.as_string().unwrap_or("Failed to parse the error.".to_string())),
	};
}

//TODO Merge branch
//TODO Choose endpoints
//TODO Update multiple timelines with the same endpoint
//TODO Update to 0.19.3
//TODO Reduce agent param names
//TODO Add image article
//TODO Filters
//TODO Rate limits
//TODO Twitter Auth
//TODO Pixiv articles
//TODO Masonry
//TODO Youtube articles
//TODO Social timestamps
//TODO Social expanded view

//TODO Show multiple article types in same timeline
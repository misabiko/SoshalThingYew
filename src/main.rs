use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub mod error;
pub mod timeline;
pub mod articles;
pub mod endpoints;
pub mod twitter;
pub mod pixiv;
mod sidebar;
mod favviewer;

use crate::sidebar::Sidebar;
use crate::timeline::{TimelineProps, Timeline};
use crate::endpoints::{EndpointAgent, EndpointId, TimelineEndpoints, Endpoint, Request as EndpointRequest};
use crate::favviewer::FavViewer;
use crate::twitter::{TwitterAgent, Response as TwitterResponse, fetch_tweet, UserTimelineEndpoint, HomeTimelineEndpoint};
use crate::pixiv::PixivAgent;

struct Model {
	endpoint_agent: Dispatcher<EndpointAgent>,
	timelines: Vec<TimelineProps>,
	#[allow(dead_code)]
	twitter: Box<dyn Bridge<TwitterAgent>>,
	#[allow(dead_code)]
	pixiv: Dispatcher<PixivAgent>,
	default_endpoint: Option<EndpointId>,
}

enum Msg {
	TwitterResponse(TwitterResponse),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	AddTimeline(EndpointId),
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(ctx: &Context<Self>) -> Self {
		let pathname_opt = match web_sys::window().map(|w| w.location()) {
			Some(location) => match location.pathname() {
				Ok(pathname_opt) => Some(pathname_opt),
				Err(_) => None
			},
			None => None,
		};

		match pathname_opt.as_ref() {
			Some(pathname) => if let Some(id) = pathname.strip_prefix("/twitter/status/").map(str::to_owned) {
				/*ctx.link().send_future(async move {
					match fetch_tweet(&id).await {
						Ok(tweet) => Msg::FetchedBootArticles(vec!(tweet)),
						Err(err) => {
							log::error!("Failed to fetch \"{}\"\n{:?}", &id, err);
							Msg::FailedToFetch
						}
					}
				});*/
			} else if let Some(username) = pathname.strip_prefix("/twitter/user/").map(str::to_owned) {
				let callback = ctx.link().callback(Msg::AddTimeline);
				log::debug!("Adding endpoint for {}", &username);
				ctx.link().send_message(
					Msg::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(UserTimelineEndpoint {
							id,
							username: username.clone(),
							agent: TwitterAgent::dispatcher(),
							articles: Vec::new()
						})
					}))
				);
			} else if pathname.starts_with("/twitter/home") {
				let callback = ctx.link().callback(Msg::AddTimeline);
				ctx.link().send_message(
					Msg::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(HomeTimelineEndpoint {
							id,
							agent: TwitterAgent::dispatcher(),
							articles: Vec::new()
						})
					}))
				);
			},
			None => {}
		};

		Self {
			timelines: Vec::new(),
			endpoint_agent: EndpointAgent::dispatcher(),
			twitter: TwitterAgent::bridge(ctx.link().callback(Msg::TwitterResponse)),
			pixiv: PixivAgent::dispatcher(),
			default_endpoint: None,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::TwitterResponse(r) => match r {
				TwitterResponse::DefaultEndpoint(e) => {
					self.default_endpoint = Some(e);
					true
				}
			},
			Msg::AddEndpoint(e) => {
				self.endpoint_agent.send(EndpointRequest::AddEndpoint(e));
				false
			}
			Msg::AddTimeline(id) => {
				log::debug!("Adding new timeline for {}", &id);
				self.timelines.push(yew::props! { TimelineProps {
					name: "Added Timeline",
					endpoints: TimelineEndpoints {
						start: vec![id],
						refresh: vec![id],
					}
				}});
				true
			}
		}
	}

	fn view(&self, _ctx: &Context<Self>) -> Html {
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
					{for self.timelines.iter().map(|props| html! {
						<Timeline ..props.clone()/>
					})}
					{ home_timeline }
				</div>
			</>
		}
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

	match web_sys::window()
		.map(|w| w.location())
		.map(|l| l.href()) {
		Some(Ok(href)) => match href.as_str() {
			"https://www.pixiv.net/bookmark_new_illust.php" => {
				let element = gloo_utils::document()
					.query_selector("#root > div:last-child > div:nth-child(2)")
					.expect("can't get mount node for rendering")
					.expect("can't unwrap mount node");
				yew::start_app_in_element::<FavViewer>(element);
			}
			_ => {
				yew::start_app::<Model>();
			},
		},
		None => log::error!("Failed to get location.href."),
		Some(Err(err)) => log::error!("Failed to get location.href.\n{}", &err.as_string().unwrap_or("Failed to parse the error.".to_string())),
	};
}

//TODO Add timeline to pixiv
//TODO Masonry
//TODO Choose endpoints
//TODO Add image article
//TODO Add timelines
//TODO Filters
//TODO Rate limits
//TODO Pixiv articles
//TODO Youtube articles
//TODO Social expanded view
//TODO HTTPS

//TODO Show multiple article types in same timeline
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};

pub mod error;
pub mod timeline;
pub mod containers;
pub mod articles;
pub mod services;
mod sidebar;
mod favviewer;

use crate::sidebar::Sidebar;
use crate::timeline::{Props as TimelineProps, Timeline};
use crate::services::{
	endpoints::{EndpointAgent, EndpointId, TimelineEndpoints, Endpoint, Request as EndpointRequest},
	twitter::{TwitterAgent, endpoints::{UserTimelineEndpoint, HomeTimelineEndpoint, SingleTweetEndpoint}},
	pixiv::{PixivAgent, FollowEndpoint},
};
use crate::favviewer::{PageInfo, pixiv::PixivPageInfo};

#[derive(PartialEq, Clone)]
enum DisplayMode {
	Single {
		column_count: u8
	},
	Default
}

impl Default for DisplayMode {
	fn default() -> Self {
		DisplayMode::Default
	}
}

struct Model {
	endpoint_agent: Dispatcher<EndpointAgent>,
	display_mode: DisplayMode,
	timelines: Vec<TimelineProps>,
	#[allow(dead_code)]
	twitter: Dispatcher<TwitterAgent>,
	#[allow(dead_code)]
	pixiv: Dispatcher<PixivAgent>,
	page_info: Option<Box<dyn PageInfo>>,
}

enum Msg {
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	AddTimeline(String, EndpointId),
	ToggleFavViewer,
}

#[derive(Properties, PartialEq, Default)]
struct Props {
	favviewer: bool,
	#[prop_or_default]
	display_mode: Option<DisplayMode>
}

impl Component for Model {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let (pathname_opt, search_opt) = match web_sys::window().map(|w| w.location()) {
			Some(location) => (match location.pathname() {
				Ok(pathname_opt) => Some(pathname_opt),
				Err(err) => {
					log::error!("Failed to get location.pathname.\n{:?}", err);
					None
				}
			}, match location.search().and_then(|s| web_sys::UrlSearchParams::new_with_str(&s)) {
				Ok(search_opt) => Some(search_opt),
				Err(err) => {
					log::error!("Failed to get location.search.\n{:?}", err);
					None
				}
			}),
			None => (None, None),
		};

		match pathname_opt.as_ref() {
			Some(pathname) => if let Some(tweet_id) = pathname.strip_prefix("/twitter/status/").and_then(|s| s.parse::<u64>().ok()) {
				let callback = ctx.link().callback(|id|Msg::AddTimeline("Tweet".to_owned(), id));
				log::debug!("Adding endpoint for {}", &tweet_id);
				ctx.link().send_message(
					Msg::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(SingleTweetEndpoint::new(id, tweet_id.clone()))
					}))
				);
			} else if let Some(username) = pathname.strip_prefix("/twitter/user/").map(str::to_owned) {
				let callback = ctx.link().callback(|id| Msg::AddTimeline("User".to_owned(), id));
				log::debug!("Adding endpoint for {}", &username);
				ctx.link().send_message(
					Msg::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(UserTimelineEndpoint::new(id, username.clone()))
					}))
				);
			} else if pathname.starts_with("/twitter/home") {
				let callback = ctx.link().callback( |id| Msg::AddTimeline("Home".to_owned(), id));
				ctx.link().send_message(
					Msg::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(HomeTimelineEndpoint::new(id))
					}))
				);
			} else {
				let callback = ctx.link().callback(|id| Msg::AddTimeline("Pixiv".to_owned(), id));
				ctx.link().send_message(
					Msg::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(FollowEndpoint::new(id))
					}))
				);
			},
			None => {}
		};

		let single_timeline_bool = search_opt.as_ref()
			.and_then(|s| s.get("single_timeline"))
			.and_then(|s| s.parse().ok())
			.unwrap_or_default();

		let display_mode = if let Some(display_mode) = &ctx.props().display_mode {
			(*display_mode).clone()
		} else if single_timeline_bool {
			DisplayMode::Single {
				column_count: search_opt.as_ref()
					.and_then(|s| s.get("column_count"))
					.and_then(|s| s.parse().ok())
					.unwrap_or(1),
			}
		}else {
			DisplayMode::Default
		};

		let page_info = Some(Box::new(PixivPageInfo::new(ctx.link().callback(|_| Msg::ToggleFavViewer))) as Box<dyn PageInfo>);

		Self {
			display_mode,
			timelines: Vec::new(),
			endpoint_agent: EndpointAgent::dispatcher(),
			twitter: TwitterAgent::dispatcher(),
			pixiv: PixivAgent::dispatcher(),
			page_info,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddEndpoint(e) => {
				self.endpoint_agent.send(EndpointRequest::AddEndpoint(e));
				false
			}
			Msg::AddTimeline(name, id) => {
				log::debug!("Adding new timeline for {}", &id);
				self.timelines.push(yew::props! { TimelineProps {
					name,
					endpoints: TimelineEndpoints {
						start: vec![id],
						refresh: vec![id],
					}
				}});
				true
			}
			Msg::ToggleFavViewer => {
				if let Some(page_info) = &mut self.page_info {
					page_info.toggle_hidden();
					true
				}else {
					false
				}
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<>
				{
					self.page_info
						.as_ref()
						.map(Box::as_ref)
						.map(PageInfo::view)
						.unwrap_or_default()
				}
				{ if ctx.props().favviewer { html! {} } else { html! {<Sidebar/>} }}
				<div id="timelineContainer">
					{
						match self.display_mode {
							DisplayMode::Default => html! {
								{for self.timelines.iter().map(|props| html! {
									<Timeline ..props.clone()/>
								})}
							},
							DisplayMode::Single {column_count} => if let Some(props) = self.timelines.first() {
								html! {
									<Timeline main_timeline=true {column_count} ..props.clone()>
										<button title="Toggle FavViewer" onclick={ctx.link().callback(|_| Msg::ToggleFavViewer)}>
											<span class="icon">
												<i class="fas fa-eye-slash fa-lg"/>
											</span>
										</button>
									</Timeline>
								}
							}else {
								html! {}
							}
						}
					}
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
		Some(Ok(href)) => {
			let href = href.as_str();
			if href.contains("pixiv.net/bookmark_new_illust") {
				let mount_point = gloo_utils::document().create_element("div").expect("to create empty div");
				mount_point.set_id("favviewer");

				gloo_utils::document()
					.query_selector("#root > div:last-child > div:nth-child(2)")
					.expect("can't get mount node for rendering")
					.expect("can't unwrap mount node")
					.append_with_node_1(&mount_point)
					.expect("can't append mount node");

				yew::start_app_with_props_in_element::<Model>(mount_point, yew::props! { Props {
					favviewer:true,
					display_mode: DisplayMode::Single {
						column_count: 5,
					}
				}});
			}else {
				yew::start_app::<Model>();
			}
		},
		None => log::error!("Failed to get location.href."),
		Some(Err(err)) => log::error!("Failed to get location.href.\n{}", &err.as_string().unwrap_or("Failed to parse the error.".to_string())),
	};
}

//TODO Sort
//TODO Save timeline data
//TODO Save fetched articles
//TODO Autoscroll
//TODO like/retweets
//TODO Retweets
//TODO Quotes
//TODO Fix container dropdown
//TODO Choose endpoints
//TODO Add image article
//TODO Add timelines
//TODO Filters
//TODO Rate limits
//TODO Youtube articles
//TODO Social expanded view
//TODO Avoid refreshing endpoint every watch update
//TODO HTTPS

//TODO Show multiple article types in same timeline
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub mod error;
pub mod timeline;
pub mod articles;
pub mod services;
pub mod modals;
pub mod dropdown;
mod sidebar;
mod favviewer;
pub mod choose_endpoints;

use crate::sidebar::Sidebar;
use crate::timeline::{Props as TimelineProps, Timeline, TimelineId, Container};
use crate::services::{
	Endpoint,
	endpoint_agent::{EndpointId, EndpointAgent, Request as EndpointRequest, Response as EndpointResponse, TimelineEndpoints},
	pixiv::{FollowEndpoint, PixivAgent},
	twitter::{endpoints::*, TwitterAgent},
};
use crate::favviewer::{PageInfo, PixivPageInfo};
use crate::modals::AddTimelineModal;
use crate::timeline::agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse};

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

pub type TimelinePropsClosure = Box<dyn FnOnce(TimelineId) -> TimelineProps>;
pub type TimelinePropsEndpointsClosure = Box<dyn FnOnce(TimelineId, TimelineEndpoints) -> TimelineProps>;

struct Model {
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	_timeline_agent: Box<dyn Bridge<TimelineAgent>>,
	display_mode: DisplayMode,
	timelines: Vec<TimelineProps>,
	page_info: Option<Box<dyn PageInfo>>,
	_twitter: Dispatcher<TwitterAgent>,
	_pixiv: Dispatcher<PixivAgent>,
	timeline_counter: TimelineId,
	main_timeline: TimelineId,
	last_display_single: DisplayMode,
}

enum Msg {
	AddEndpoint(Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>),
	AddTimeline(String, EndpointId),
	ToggleFavViewer,
	AddTimelineProps(TimelinePropsClosure),
	ToggleDisplayMode,
	TimelineAgentResponse(TimelineAgentResponse),
	EndpointResponse(EndpointResponse),
}

#[derive(Properties, PartialEq, Default)]
struct Props {
	favviewer: bool,
	#[prop_or_default]
	display_mode: Option<DisplayMode>,
}

impl Component for Model {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let (pathname_opt, search_opt) = parse_url();

		if let Some(pathname) = pathname_opt {
			parse_pathname(ctx, &pathname, &search_opt);
		}

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

		//TODO Detect current page
		let page_info = match ctx.props().favviewer {
			true => Some(Box::new(PixivPageInfo::new(ctx.link().callback(|_| Msg::ToggleFavViewer))) as Box<dyn PageInfo>),
			false => None
		};

		let _twitter = TwitterAgent::dispatcher();
		let _pixiv = PixivAgent::dispatcher();

		let mut _timeline_agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
		_timeline_agent.send(TimelineAgentRequest::RegisterTimelineContainer);
		_timeline_agent.send(TimelineAgentRequest::LoadStorageTimelines);

		let mut endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		endpoint_agent.send(EndpointRequest::RegisterTimelineContainer);

		Self {
			_timeline_agent,
			last_display_single: match display_mode {
				DisplayMode::Single{ .. } => display_mode.clone(),
				_ => DisplayMode::Single {
					column_count: 4
				},
			},
			display_mode,
			timelines: Vec::new(),
			endpoint_agent,
			page_info,
			_twitter,
			_pixiv,
			timeline_counter: TimelineId::MIN,
			main_timeline: TimelineId::MIN,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddEndpoint(e) => {
				self.endpoint_agent.send(EndpointRequest::AddEndpoint(e));
				false
			}
			Msg::AddTimeline(name, endpoint_id) => {
				let mut endpoints = Vec::new();
				let timeline_id = self.timeline_counter.clone();
				endpoints.push(endpoint_id.into());

				self.timelines.push(yew::props! { TimelineProps {
					name,
					id: timeline_id.clone(),
					endpoints: TimelineEndpoints {
						start: endpoints.clone(),
						refresh: endpoints,
					},
				}});
				self.timeline_counter += 1;

				true
			}
			Msg::AddTimelineProps(props) => {
				let id = self.timeline_counter.clone();
				self.timelines.push((props)(id.clone()));
				self.timeline_counter += 1;

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
			Msg::ToggleDisplayMode => {
				self.display_mode = match self.display_mode {
					DisplayMode::Default => self.last_display_single.clone(),
					DisplayMode::Single { .. } => DisplayMode::Default,
				};
				true
			},
			Msg::TimelineAgentResponse(response) => match response {
				TimelineAgentResponse::SetMainTimeline(id) => {
					log::debug!("Set main timeline! {}", &id);
					self.main_timeline = id;
					if let DisplayMode::Default = self.display_mode {
						self.display_mode = self.last_display_single.clone();
					};
					true
				}
				TimelineAgentResponse::RemoveTimeline(id) => {
					let index = self.timelines.iter().position(|t| t.id == id);
					if let Some(index) = index {
						let id = self.timelines[index].id.clone();
						self.timelines.remove(index);

						if id == self.main_timeline {
							self.main_timeline = match self.timelines.first() {
								Some(t) => t.id.clone(),
								None => self.timeline_counter.clone(),
							}
						}
					}
					true
				}
				TimelineAgentResponse::CreateTimelines(timelines) => {
					for props in timelines {
						self.timelines.push((props)(self.timeline_counter.clone()));
						self.timeline_counter += 1;
					}
					true
				}
				_ => false
			}
			Msg::EndpointResponse(response) => match response {
				EndpointResponse::BatchRequestResponse(timelines) => {
					for (endpoints, closure) in timelines {
						let id = self.timeline_counter.clone();
						self.timelines.push((closure)(id.clone(), endpoints));
						self.timeline_counter += 1;
					}

					true
				}
				_ => false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let (dm_title, dm_icon) = match self.display_mode {
			DisplayMode::Default => ("Single Timeline", "fa-expand-alt"),
			DisplayMode::Single { .. } => ("Multiple Timeline", "fa-columns"),
		};
		let display_mode_toggle = html! {
			<button onclick={ctx.link().callback(|_| Msg::ToggleDisplayMode)} title={dm_title}>
				<span class="icon">
					<i class={classes!("fas", "fa-2x", dm_icon)}/>
				</span>
			</button>
		};

		html! {
			<>
				<AddTimelineModal add_timeline_callback={ctx.link().callback(|props| Msg::AddTimelineProps(props))}/>
				{
					self.page_info
						.as_ref()
						.map(Box::as_ref)
						.map(PageInfo::view)
						.unwrap_or_default()
				}
				{
					match ctx.props().favviewer {
						false => html! {
							<Sidebar>
								{ display_mode_toggle }
							</Sidebar>
						},
						true => html! {},
					}
				}
				<div id="timelineContainer">
					{
						match self.display_mode {
							DisplayMode::Default => html! {
								{for self.timelines.iter().map(|props| html! {
									<Timeline key={props.id.clone()} ..props.clone()/>
								})}
							},
							DisplayMode::Single {column_count} => html! {
								{for self.timelines.iter().map(|props| if props.id == self.main_timeline {
									 html! {
										<Timeline key={props.id.clone()} main_timeline=true container={Container::Masonry} {column_count} ..props.clone()>
											{
												match ctx.props().favviewer {
													true => html! {
														<button title="Toggle FavViewer" onclick={ctx.link().callback(|_| Msg::ToggleFavViewer)}>
															<span class="icon">
																<i class="fas fa-eye-slash fa-lg"/>
															</span>
														</button>
													},
													false => html! {}
												}
											}
										</Timeline>
									}
								}else  {
									html! {
										<Timeline hide=true key={props.id.clone()} ..props.clone()/>
									}
								})}
							}
						}
					}
				</div>
			</>
		}
	}
}

fn parse_url() -> (Option<String>, Option<web_sys::UrlSearchParams>) {
	match web_sys::window().map(|w| w.location()) {
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
	}
}

fn parse_pathname(ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>) {
	if let Some(tweet_id) = pathname.strip_prefix("/twitter/status/").and_then(|s| s.parse::<u64>().ok()) {
		let callback = ctx.link().callback(|id|Msg::AddTimeline("Tweet".to_owned(), id));

		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(SingleTweetEndpoint::new(id, tweet_id.clone()))
			}))
		);
	} else if let Some(username) = pathname.strip_prefix("/twitter/user/").map(str::to_owned) {
		let (retweets, replies) = match search_opt {
			Some(search) => (
				search.get("rts")
					.and_then(|s| s.parse().ok())
					.unwrap_or_default(),
				search.get("replies")
				.and_then(|s| s.parse().ok())
				.unwrap_or_default()
			),
			None => (false, false)
		};
		let callback = ctx.link().callback(|id| Msg::AddTimeline("User".to_owned(), id));

		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(UserTimelineEndpoint::new(id, username.clone(), retweets, replies))
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
	} else if let Some(list_params) = pathname.strip_prefix("/twitter/list/").map(|s| s.split("/").collect::<Vec<&str>>()) {
		if let [username, slug] = list_params[..] {
			let callback = ctx.link().callback(|id| Msg::AddTimeline("List".to_owned(), id));
			let username = username.to_owned();
			let slug = slug.to_owned();

			ctx.link().send_message(
				Msg::AddEndpoint(Box::new(move |id| {
					callback.emit(id);
					Box::new(ListEndpoint::new(id, username, slug))
				}) /*as Box<dyn FnOnce(EndpointId) -> Box<ListEndpoint>>*/)
			);
		}
	} if ctx.props().favviewer {
		let callback = ctx.link().callback(|id| Msg::AddTimeline("Pixiv".to_owned(), id));
		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(FollowEndpoint::new(id))
			}))
		);
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
					favviewer: true,
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

//TODO Profile lag when redrawing articles
//TODO Youtube articles
	//TODO Have custom service setting view
	//TODO Show quato units for Youtube service
	//TODO Cache playlist id for each subscribed channel
//TODO Custom social buttons per article type
//TODO Notifications
//TODO Save timeline data
//TODO Display timeline errors
//TODO Prompt on not logged in
//TODO Save fetched articles
//TODO Avoid refreshing endpoint every watch update
//TODO Add "Open @myusername on soshalthing" context menu?

//TODO Show multiple article types in same timeline

//TODO Fix articles not redrawing when they redraw...
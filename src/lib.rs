use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};
use std::collections::HashMap;
use gloo_storage::Storage;
use serde::Deserialize;

pub mod articles;
pub mod choose_endpoints;
pub mod components;
pub mod error;
pub mod favviewer;
pub mod modals;
pub mod notifications;
pub mod services;
pub mod timeline;
mod sidebar;

use components::{FA, IconSize};
use error::Result;
use favviewer::PageInfo;
use modals::AddTimelineModal;
use notifications::{NotificationAgent, Request as NotificationRequest, Response as NotificationResponse};
use services::{
	Endpoint,
	endpoint_agent::{EndpointId, EndpointAgent, Request as EndpointRequest, Response as EndpointResponse, TimelineEndpoints, TimelineCreationRequest},
	pixiv::PixivAgent,
	twitter::{endpoints::*, TwitterAgent, Request as TwitterRequest, Response as TwitterResponse},
};
use sidebar::Sidebar;
use timeline::{Props as TimelineProps, Timeline, TimelineId, Container};
use timeline::agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse};

#[derive(Deserialize)]
struct ModelStorage {
	display_mode: DisplayMode,
}

#[derive(PartialEq, Clone, Copy, Deserialize)]
#[serde(tag = "type")]
pub enum DisplayMode {
	Single {
		container: Container,
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

pub enum TimelineCreationMode {
	NameEndpoints(String, TimelineEndpoints),
	Props(TimelinePropsClosure),
}

#[derive(serde::Deserialize)]
pub struct AuthInfo {
	twitter: Option<String>,
}

pub struct Model {
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	_timeline_agent: Box<dyn Bridge<TimelineAgent>>,
	display_mode: DisplayMode,
	timelines: Vec<TimelineProps>,
	page_info: Option<PageInfo>,
	twitter: Box<dyn Bridge<TwitterAgent>>,
	_pixiv: Dispatcher<PixivAgent>,
	timeline_counter: TimelineId,
	main_timeline: TimelineId,
	last_display_single: DisplayMode,
	services_sidebar: HashMap<String, Html>,
	_notification_agent: Box<dyn Bridge<NotificationAgent>>,
	notifications: Vec<Html>,
}

pub enum Msg {
	AddEndpoint(Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>),
	BatchAddEndpoints(Vec<Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>>, Vec<Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>>, TimelineCreationRequest),
	AddTimeline(TimelineCreationMode),
	ToggleFavViewer,
	ToggleDisplayMode,
	TimelineAgentResponse(TimelineAgentResponse),
	EndpointResponse(EndpointResponse),
	TwitterResponse(TwitterResponse),
	FetchedAuthInfo(Result<AuthInfo>),
	NotificationResponse(NotificationResponse),
}

#[derive(Properties, PartialEq, Default)]
pub struct Props {
	pub favviewer: bool,
	#[prop_or_default]
	pub display_mode: Option<DisplayMode>,
	#[prop_or_default]
	pub page_info: Option<PageInfo>,
	#[prop_or_default]
	pub services_sidebar: HashMap<String, Html>,
}

impl Component for Model {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut _notification_agent = NotificationAgent::bridge(ctx.link().callback(Msg::NotificationResponse));
		_notification_agent.send(NotificationRequest::RegisterTimelineContainer);

		let mut twitter = TwitterAgent::bridge(ctx.link().callback(Msg::TwitterResponse));
		twitter.send(TwitterRequest::Sidebar);
		let _pixiv = PixivAgent::dispatcher();

		let mut _timeline_agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
		_timeline_agent.send(TimelineAgentRequest::RegisterTimelineContainer);
		_timeline_agent.send(TimelineAgentRequest::LoadStorageTimelines);

		let mut endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		endpoint_agent.send(EndpointRequest::RegisterTimelineContainer);

		let (pathname_opt, search_opt) = parse_url();

		//TODO use memreplace Some(Setup) → None
		let page_info = match &ctx.props().page_info {
			Some(PageInfo::Setup { style_html, initial_style, make_activator, add_timelines }) => {
				(add_timelines)();

				Some(PageInfo::Ready {
					style_html: style_html.clone(),
					style: initial_style.clone(),
					favviewer_button: (make_activator)(ctx.link().callback(| _ | Msg::ToggleFavViewer))
				})
			}
			_ => None,
		};

		if let Some(pathname) = pathname_opt {
			parse_pathname(ctx, &pathname, &search_opt);
		}

		let single_timeline_bool = search_opt.as_ref()
			.and_then(|s| s.get("single_timeline"))
			.and_then(|s| s.parse().ok())
			.unwrap_or_default();

		let display_mode = if let Some(ModelStorage { display_mode }) = gloo_storage::LocalStorage::get("SoshalThingYew").ok() {
			display_mode
		}else if let Some(display_mode) = &ctx.props().display_mode {
			*display_mode
		} else if single_timeline_bool {
			DisplayMode::Single {
				container: search_opt.as_ref()
					.and_then(|s| s.get("container"))
					.as_ref().and_then(|s| Container::from(s).ok())
					.unwrap_or(Container::Masonry),
				column_count: search_opt.as_ref()
					.and_then(|s| s.get("column_count"))
					.and_then(|s| s.parse().ok())
					.unwrap_or(4),
			}
		}else {
			DisplayMode::Default
		};

		ctx.link().send_future(async {
			Msg::FetchedAuthInfo(fetch_auth_info().await)
		});

		Self {
			_timeline_agent,
			last_display_single: match display_mode {
				DisplayMode::Single{ .. } => display_mode,
				_ => DisplayMode::Single {
					container: Container::Masonry,
					column_count: 4,
				},
			},
			display_mode,
			timelines: Vec::new(),
			endpoint_agent,
			page_info,
			twitter,
			_pixiv,
			timeline_counter: TimelineId::MIN,
			main_timeline: TimelineId::MIN,
			services_sidebar: ctx.props().services_sidebar.clone(),
			_notification_agent,
			notifications: Vec::new(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddEndpoint(e) => {
				self.endpoint_agent.send(EndpointRequest::AddEndpoint(e));
				false
			}
			Msg::BatchAddEndpoints(start, refresh, creation_request) => {
				self.endpoint_agent.send(EndpointRequest::BatchAddEndpoints(start, refresh, creation_request));
				false
			}
			Msg::AddTimeline(creation_mode) => {
				let timeline_id = self.timeline_counter;
				match creation_mode {
					TimelineCreationMode::NameEndpoints(name, endpoints) => {
						self.timelines.push(yew::props! { TimelineProps {
							name,
							id: timeline_id,
							endpoints,
						}});
					}
					TimelineCreationMode::Props(props) => {
						self.timelines.push((props)(timeline_id));
					}
				}
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
					DisplayMode::Default => self.last_display_single,
					DisplayMode::Single { .. } => DisplayMode::Default,
				};
				true
			},
			Msg::TimelineAgentResponse(response) => match response {
				TimelineAgentResponse::SetMainTimeline(id) => {
					log::debug!("Set main timeline! {}", &id);
					self.main_timeline = id;
					if let DisplayMode::Default = self.display_mode {
						self.display_mode = self.last_display_single;
					};
					true
				}
				TimelineAgentResponse::RemoveTimeline(id) => {
					let index = self.timelines.iter().position(|t| t.id == id);
					if let Some(index) = index {
						let id = self.timelines[index].id;
						self.timelines.remove(index);

						if id == self.main_timeline {
							self.main_timeline = match self.timelines.first() {
								Some(t) => t.id,
								None => self.timeline_counter,
							}
						}
					}
					true
				}
				TimelineAgentResponse::CreateTimelines(timelines) => {
					for props in timelines {
						self.timelines.push((props)(self.timeline_counter));
						self.timeline_counter += 1;
					}
					true
				}
				_ => false
			}
			Msg::EndpointResponse(response) => match response {
				EndpointResponse::BatchRequestResponse(timelines) => {
					for (endpoints, closure) in timelines {
						let id = self.timeline_counter;
						self.timelines.push((closure)(id, endpoints));
						self.timeline_counter += 1;
					}

					true
				}
				EndpointResponse::AddTimeline(creation_mode) => {
					ctx.link().send_message(Msg::AddTimeline(creation_mode));
					false
				},
				_ => false
			}
			Msg::TwitterResponse(response) => {
				match response {
					TwitterResponse::Sidebar(html) => self.services_sidebar.insert("Twitter".to_owned(), html),
				};
				true
			}
			Msg::FetchedAuthInfo(response) => {
				match response {
					Ok(auth_info) => {
						self.twitter.send(TwitterRequest::Auth(auth_info.twitter));
					}
					Err(err) => log::error!("{}", err),
				};
				false
			}
			Msg::NotificationResponse(response) => {
				match response {
					NotificationResponse::DrawNotifications(notifs) => self.notifications = notifs,
				};
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let (dm_title, dm_icon) = match self.display_mode {
			DisplayMode::Default => ("Single Timeline", "expand-alt"),
			DisplayMode::Single { .. } => ("Multiple Timeline", "columns"),
		};
		let display_mode_toggle = html! {
			<button onclick={ctx.link().callback(|_| Msg::ToggleDisplayMode)} title={dm_title}>
				<FA icon={dm_icon} size={IconSize::X2}/>
			</button>
		};

		html! {
			<>
				<AddTimelineModal add_timeline_callback={ctx.link().callback(|props| Msg::AddTimeline(TimelineCreationMode::Props(props)))}/>
				<div id="soshal-notifications">
					{ for self.notifications.iter().cloned() }
				</div>
				{ self.page_info.as_ref().map(|p| p.view()).unwrap_or_default() }
				{
					match ctx.props().favviewer {
						false => html! {
							<Sidebar services={self.services_sidebar.values().cloned().collect::<Vec<Html>>()}>
								{ display_mode_toggle }
							</Sidebar>
						},
						true => html! {},
					}
				}
				<div id="timelineContainer">
					{
						match &self.display_mode {
							DisplayMode::Default => html! {
								{for self.timelines.iter().map(|props| html! {
									<Timeline key={props.id} ..props.clone()/>
								})}
							},
							DisplayMode::Single {container, column_count} => html! {
								{for self.timelines.iter().map(|props| if props.id == self.main_timeline {
									 html! {
										<Timeline key={props.id.clone()} main_timeline=true container={container.clone()} column_count={column_count.clone()} ..props.clone()>
											{
												match ctx.props().favviewer {
													true => html! {
														<button title="Toggle FavViewer" onclick={ctx.link().callback(|_| Msg::ToggleFavViewer)}>
															<FA icon="eye-slash" size={IconSize::Large}/>
														</button>
													},
													false => html! {}
												}
											}
										</Timeline>
									}
								}else  {
									html! {
										<Timeline hide=true key={props.id} ..props.clone()/>
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

pub fn parse_url() -> (Option<String>, Option<web_sys::UrlSearchParams>) {
	match web_sys::window().map(|w| w.location()) {
		Some(location) => (match location.pathname() {
			Ok(pathname_opt) => Some(pathname_opt),
			Err(err) => {
				log_error!("Failed to get location.pathname", err);
				None
			}
		}, match location.search().and_then(|s| web_sys::UrlSearchParams::new_with_str(&s)) {
			Ok(search_opt) => Some(search_opt),
			Err(err) => {
				log_error!("Failed to get location.search", err);
				None
			}
		}),
		None => (None, None),
	}
}

pub fn parse_pathname(ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>) {
	if let Some(tweet_id) = pathname.strip_prefix("/twitter/status/").and_then(|s| s.parse::<u64>().ok()) {
		let callback = ctx.link().callback(|id| Msg::AddTimeline(TimelineCreationMode::NameEndpoints("Tweet".to_owned(), TimelineEndpoints::new_with_endpoint_both(id))));

		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(SingleTweetEndpoint::new(id, tweet_id))
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
		let callback = ctx.link().callback(|id| Msg::AddTimeline(TimelineCreationMode::NameEndpoints("User".to_owned(), TimelineEndpoints::new_with_endpoint_both(id))));

		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(UserTimelineEndpoint::new(id, username.clone(), retweets, replies))
			}))
		);
	} else if pathname.starts_with("/twitter/home") {
		let callback = ctx.link().callback( |id| Msg::AddTimeline(TimelineCreationMode::NameEndpoints("Home".to_owned(), TimelineEndpoints::new_with_endpoint_both(id))));
		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(HomeTimelineEndpoint::new(id))
			}))
		);
	} else if let Some(list_params) = pathname.strip_prefix("/twitter/list/").map(|s| s.split("/").collect::<Vec<&str>>()) {
		if let [username, slug] = list_params[..] {
			let callback = ctx.link().callback(|id| Msg::AddTimeline(TimelineCreationMode::NameEndpoints("List".to_owned(), TimelineEndpoints::new_with_endpoint_both(id))));
			let username = username.to_owned();
			let slug = slug.to_owned();

			ctx.link().send_message(
				Msg::AddEndpoint(Box::new(move |id| {
					callback.emit(id);
					Box::new(ListEndpoint::new(id, username, slug))
				}) /*as Box<dyn FnOnce(EndpointId) -> Box<ListEndpoint>>*/)
			);
		}
	}
}

//Maybe move to a generic util module?
pub fn base_url() -> String {
	let location = web_sys::window().unwrap().location();
	let host = location.host().unwrap();
	let protocol = location.protocol().unwrap();

	format!("{}//{}", protocol, host)
}

async fn fetch_auth_info() -> Result<AuthInfo> {
	Ok(reqwest::Client::builder().build()?
		.get(format!("{}/proxy/auth_info", base_url()))
		.send().await?
		.json::<AuthInfo>().await?)
}
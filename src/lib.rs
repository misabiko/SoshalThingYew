use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub mod error;
pub mod timeline;
pub mod articles;
pub mod services;
pub mod modals;
pub mod dropdown;
mod sidebar;
pub mod favviewer;
pub mod choose_endpoints;

use crate::sidebar::Sidebar;
use crate::timeline::{Props as TimelineProps, Timeline, TimelineId, Container};
use crate::services::{
	Endpoint,
	endpoint_agent::{EndpointId, EndpointAgent, Request as EndpointRequest, Response as EndpointResponse, TimelineEndpoints},
	pixiv::{FollowPageEndpoint, PixivAgent},
	twitter::{endpoints::*, TwitterAgent},
};
use crate::favviewer::{FavViewerStyle, PageInfo};
use crate::modals::AddTimelineModal;
use crate::services::endpoint_agent::TimelineCreationRequest;
use crate::services::pixiv::FollowAPIEndpoint;
use crate::timeline::agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse};

#[derive(PartialEq, Clone)]
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

pub struct Model {
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	_timeline_agent: Box<dyn Bridge<TimelineAgent>>,
	display_mode: DisplayMode,
	timelines: Vec<TimelineProps>,
	page_info: Option<PageInfo>,
	_twitter: Dispatcher<TwitterAgent>,
	_pixiv: Dispatcher<PixivAgent>,
	timeline_counter: TimelineId,
	main_timeline: TimelineId,
	last_display_single: DisplayMode,
}

pub enum Msg {
	AddEndpoint(Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>),
	BatchAddEndpoints(Vec<Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>>, Vec<Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>>, TimelineCreationRequest),
	AddTimeline(TimelineCreationMode),
	ToggleFavViewer,
	ToggleDisplayMode,
	TimelineAgentResponse(TimelineAgentResponse),
	EndpointResponse(EndpointResponse),
}

#[derive(Properties, PartialEq, Default)]
pub struct Props {
	pub favviewer: bool,
	#[prop_or_default]
	pub display_mode: Option<DisplayMode>,
	#[prop_or_default]
	pub page_info: Option<PageInfo>,
}

impl Component for Model {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let _twitter = TwitterAgent::dispatcher();
		let _pixiv = PixivAgent::dispatcher();

		let mut _timeline_agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
		_timeline_agent.send(TimelineAgentRequest::RegisterTimelineContainer);
		_timeline_agent.send(TimelineAgentRequest::LoadStorageTimelines);

		let mut endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		endpoint_agent.send(EndpointRequest::RegisterTimelineContainer);

		let (pathname_opt, search_opt) = parse_url();

		//TODO use memreplace Some(Setup) → None
		let mut page_info = match &ctx.props().page_info {
			Some(PageInfo::Setup { style_html, make_activator, add_timelines }) => {
				(add_timelines)();

				Some(PageInfo::Ready {
					style_html: style_html.clone(),
					style: FavViewerStyle::Hidden,
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

		let display_mode = if let Some(display_mode) = &ctx.props().display_mode {
			(*display_mode).clone()
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

		Self {
			_timeline_agent,
			last_display_single: match display_mode {
				DisplayMode::Single{ .. } => display_mode.clone(),
				_ => DisplayMode::Single {
					container: Container::Masonry,
					column_count: 4,
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
				let timeline_id = self.timeline_counter.clone();
				match creation_mode {
					TimelineCreationMode::NameEndpoints(name, endpoints) => {
						self.timelines.push(yew::props! { TimelineProps {
							name,
							id: timeline_id.clone(),
							endpoints,
						}});
					}
					TimelineCreationMode::Props(props) => {
						self.timelines.push((props)(timeline_id.clone()));
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
				EndpointResponse::AddTimeline(creation_mode) => {
					ctx.link().send_message(Msg::AddTimeline(creation_mode));
					false
				},
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
				<AddTimelineModal add_timeline_callback={ctx.link().callback(|props| Msg::AddTimeline(TimelineCreationMode::Props(props)))}/>
				{ self.page_info.as_ref().map(|p| p.view()).unwrap_or_default() }
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
						match &self.display_mode {
							DisplayMode::Default => html! {
								{for self.timelines.iter().map(|props| html! {
									<Timeline key={props.id.clone()} ..props.clone()/>
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

pub fn parse_url() -> (Option<String>, Option<web_sys::UrlSearchParams>) {
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

pub fn parse_pathname(ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>) {
	if let Some(tweet_id) = pathname.strip_prefix("/twitter/status/").and_then(|s| s.parse::<u64>().ok()) {
		let callback = ctx.link().callback(|id| Msg::AddTimeline(TimelineCreationMode::NameEndpoints("Tweet".to_owned(), TimelineEndpoints::new_with_endpoint_both(id))));

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
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use serde::{Serialize, Deserialize};

use super::{
	Props as TimelineProps, Timeline, TimelineId, Container,
	agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse},
};
use crate::{AppSettings, TimelineContainerCallback};
use crate::services::{
	endpoint_agent::{EndpointAgent, TimelineEndpointWrapper, Request as EndpointRequest, Response as EndpointResponse},
	twitter::endpoints::*
};
use crate::components::{FA, IconSize};
use crate::modals::{
	add_timeline::AddTimelineModal,
	Modal,
};
use crate::timeline::filters::FilterCollection;

pub struct TimelineContainer {
	timelines: Vec<TimelineProps>,
	modal_timeline: Option<TimelineProps>,
	timeline_counter: TimelineId,
	main_timeline: TimelineId,
	_timeline_agent: Box<dyn Bridge<TimelineAgent>>,
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
}

pub enum TimelineContainerMsg {
	AddTimeline(TimelineCreationMode, bool),
	AddModalTimeline(TimelineCreationMode),
	RemoveModalTimeline,
	TimelineAgentResponse(TimelineAgentResponse),
	EndpointResponse(EndpointResponse),
}

type Msg = TimelineContainerMsg;

#[derive(Properties, PartialEq)]
pub struct TimelineContainerProps {
	pub parent_callback: Callback<TimelineContainerCallback>,
	pub app_settings: AppSettings,
	pub favviewer: bool,
	pub display_mode: DisplayMode,
}

impl Component for TimelineContainer {
	type Message = Msg;
	type Properties = TimelineContainerProps;

	fn create(ctx: &Context<Self>) -> Self {
		let mut _timeline_agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
		_timeline_agent.send(TimelineAgentRequest::RegisterTimelineContainer);
		_timeline_agent.send(TimelineAgentRequest::LoadStorageTimelines);

		let mut endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		endpoint_agent.send(EndpointRequest::RegisterTimelineContainer);

		parse_pathname(ctx, &mut endpoint_agent);

		Self {
			timelines: Vec::new(),
			modal_timeline: None,
			timeline_counter: TimelineId::MIN,
			main_timeline: TimelineId::MIN,
			_timeline_agent,
			endpoint_agent,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddTimeline(creation_mode, set_as_main_timeline) => {
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
				if set_as_main_timeline {
					self.main_timeline = timeline_id;
					if let DisplayMode::Default = ctx.props().display_mode {
						ctx.props().parent_callback.emit(TimelineContainerCallback::ToggleDisplayMode);
					};
				}

				self.timeline_counter += 1;
				true
			}
			Msg::AddModalTimeline(creation_mode) => {
				if let Some(modal_timeline) = &self.modal_timeline {
					self.endpoint_agent.send(EndpointRequest::RemoveTimeline(modal_timeline.id));
				}
				let id = self.timeline_counter;

				match creation_mode {
					TimelineCreationMode::NameEndpoints(name, endpoints) => {
						self.modal_timeline = Some(yew::props! { TimelineProps {
							name,
							id,
							endpoints,
						}});
					}
					TimelineCreationMode::Props(props) => {
						self.modal_timeline = Some((props)(id));
					}
				}

				self.timeline_counter += 1;
				true
			}
			Msg::RemoveModalTimeline => {
				if let Some(modal_timeline) = std::mem::take(&mut self.modal_timeline) {
					self.endpoint_agent.send(EndpointRequest::RemoveTimeline(modal_timeline.id));

					true
				}else {
					false
				}
			}
			Msg::TimelineAgentResponse(response) => match response {
				TimelineAgentResponse::SetMainTimeline(id) => {
					self.main_timeline = id;
					if let DisplayMode::Default = ctx.props().display_mode {
						ctx.props().parent_callback.emit(TimelineContainerCallback::ToggleDisplayMode);
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
				TimelineAgentResponse::AddQuickUserTimeline(service, username) => {
					let callback = {
						let username = username.clone();
						let /*mut*/ filters = FilterCollection::default();
						//filters.update(FilterMsg::AddFilter((Filter::Media, false)));
						//filters.update(FilterMsg::AddFilter((Filter::PlainTweet, false)));

						ctx.link().callback_once(|endpoint_id| Msg::AddModalTimeline(
							TimelineCreationMode::NameEndpoints(
								username,
								vec![TimelineEndpointWrapper {
									id: endpoint_id,
									on_start: true,
									on_refresh: true,
									filters,
								}],
							)
						))
					};

					self.endpoint_agent.send(EndpointRequest::AddUserEndpoint {
						service,
						username,
						shared: false,
						callback,
					});

					false
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
				EndpointResponse::AddTimeline(creation_mode, set_as_main_timeline) => {
					ctx.link().send_message(Msg::AddTimeline(creation_mode, set_as_main_timeline));
					false
				}
				_ => false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let add_timeline_callback = ctx.link().callback(|(props, set_as_main_timeline)|
			Msg::AddTimeline(TimelineCreationMode::Props(props), set_as_main_timeline)
		);

		html! {
			<>
				<AddTimelineModal {add_timeline_callback}/>
				<div id="timelineContainer">
					{ self.view_modal_timeline(ctx) }
					{ self.view_timelines(ctx) }
				</div>
			</>
		}
	}
}

impl TimelineContainer {
	fn view_timelines(&self, ctx: &Context<Self>) -> Html {
		let app_settings = &ctx.props().app_settings;

		match &ctx.props().display_mode {
			DisplayMode::Default => html! {
				<>
					{for self.timelines.iter().map(|props| html! {
						<Timeline key={props.id} app_settings={app_settings.clone()} ..props.clone()/>
					})}
				</>
			},
			DisplayMode::Single { container, column_count } => html! {
				<>
					{for self.timelines.iter().map(|props|
						if props.id == self.main_timeline {
							self.view_main_timeline(ctx, props, *container, *column_count)
						}else  {
							html! {
								<Timeline hide=true key={props.id} app_settings={app_settings.clone()} ..props.clone()/>
							}
						}
					)}
				</>
			}
		}
	}

	fn view_main_timeline(&self, ctx: &Context<Self>, props: &TimelineProps, container: Container, column_count: u8) -> Html {
		let toggle_favviewer_onclick = ctx.props().parent_callback
			.reform(|_: MouseEvent| TimelineContainerCallback::ToggleFavViewer);
		let toggle_sidebar_onclick = ctx.props().parent_callback
			.reform(|_: MouseEvent| TimelineContainerCallback::ToggleSidebarFavViewer);

		html! {
			<Timeline key={props.id} app_settings={ctx.props().app_settings} main_timeline=true {container} {column_count} ..props.clone()>
				{ match ctx.props().favviewer {
					true => html! {
						<>
							<button title="Toggle FavViewer" onclick={toggle_favviewer_onclick}>
								<FA icon="eye-slash" size={IconSize::Large}/>
							</button>
							<button title="Show Sidebar" onclick={toggle_sidebar_onclick}>
								<FA icon="ellipsis-v" size={IconSize::Large}/>
							</button>
						</>
					},
					false => html! {}
				} }
			</Timeline>
		}
	}

	fn view_modal_timeline(&self, ctx: &Context<Self>) -> Html {
		if let Some(modal_timeline) = &self.modal_timeline {
			let close_modal_callback = ctx.link().callback(|_| Msg::RemoveModalTimeline);
			let content_style = "width: unset".to_owned();

			html! {
				<Modal {content_style} {close_modal_callback}>
					<Timeline
						app_settings={ctx.props().app_settings.clone()}
						modal=true
						..modal_timeline.clone()
					/>
				</Modal>
			}
		}else {
			html! {}
		}
	}
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DisplayMode {
	Single {
		container: Container,
		column_count: u8,
	},
	Default,
}

impl Default for DisplayMode {
	fn default() -> Self {
		DisplayMode::Default
	}
}

pub type TimelinePropsClosure = Box<dyn FnOnce(TimelineId) -> TimelineProps>;
pub type TimelinePropsEndpointsClosure = Box<dyn FnOnce(TimelineId, Vec<TimelineEndpointWrapper>) -> TimelineProps>;

pub enum TimelineCreationMode {
	NameEndpoints(String, Vec<TimelineEndpointWrapper>),
	Props(TimelinePropsClosure),
}

fn parse_pathname(ctx: &Context<TimelineContainer>, endpoint_agent: &mut Box<dyn Bridge<EndpointAgent>>) {
	let location = web_sys::window().unwrap().location();
	let pathname = location.pathname().unwrap();
	let search = web_sys::UrlSearchParams::new_with_str(&location.search().unwrap()).unwrap();

	if let Some(tweet_id) = pathname.strip_prefix("/twitter/status/").and_then(|s| s.parse::<u64>().ok()) {
		let callback = ctx.link().callback(|id| Msg::AddTimeline(
			TimelineCreationMode::NameEndpoints("Tweet".to_owned(), vec![TimelineEndpointWrapper::new_both(id)]),
			false,
		));

		endpoint_agent.send(EndpointRequest::AddEndpoint {
			id_to_endpoint: Box::new(move |id| {
				callback.emit(id);
				Box::new(SingleTweetEndpoint::new(id, tweet_id))
			}),
			shared: false,
		});
	} else if let Some(username) = pathname.strip_prefix("/twitter/user/").map(str::to_owned) {
		let retweets = search.get("rts")
			.and_then(|s| s.parse().ok())
			.unwrap_or_default();
		let replies = search.get("replies")
			.and_then(|s| s.parse().ok())
			.unwrap_or_default();

		let callback = ctx.link().callback(|id| Msg::AddTimeline(
			TimelineCreationMode::NameEndpoints("User".to_owned(), vec![TimelineEndpointWrapper::new_both(id)]),
			false,
		));

		endpoint_agent.send(EndpointRequest::AddEndpoint {
			id_to_endpoint: Box::new(move |id| {
				callback.emit(id);
				Box::new(UserTimelineEndpoint::new(id, username.clone(), retweets, replies))
			}),
			shared: false,
		});
	} else if pathname.starts_with("/twitter/home") {
		let callback = ctx.link().callback(|id| Msg::AddTimeline(
			TimelineCreationMode::NameEndpoints("Home".to_owned(), vec![TimelineEndpointWrapper::new_both(id)]),
			false,
		));

		endpoint_agent.send(EndpointRequest::AddEndpoint {
			id_to_endpoint: Box::new(move |id| {
				callback.emit(id);
				Box::new(HomeTimelineEndpoint::new(id))
			}),
			shared: false,
		});
	} else if let Some(list_params) = pathname.strip_prefix("/twitter/list/").map(|s| s.split("/").collect::<Vec<&str>>()) {
		if let [username, slug] = list_params[..] {
			let callback = ctx.link().callback(|id| Msg::AddTimeline(
				TimelineCreationMode::NameEndpoints("List".to_owned(), vec![TimelineEndpointWrapper::new_both(id)]),
				false,
			));
			let username = username.to_owned();
			let slug = slug.to_owned();

			endpoint_agent.send(EndpointRequest::AddEndpoint {
				id_to_endpoint: Box::new(move |id| {
					callback.emit(id);
					Box::new(ListEndpoint::new(id, username, slug))
				}),
				shared: false,
			});
		}
	}
}
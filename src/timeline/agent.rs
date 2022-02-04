use gloo_storage::errors::StorageError;
use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext, Dispatcher, Dispatched};
use gloo_storage::Storage;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

use super::{TimelineId, Props as TimelineProps, Container};
use crate::services::EndpointSerialized;
use crate::services::endpoint_agent::{Request as EndpointRequest, EndpointAgent};
use crate::{TimelineEndpointWrapper, TimelinePropsClosure, TimelinePropsEndpointsClosure};
use crate::log_warn;
use crate::timeline::filters::FilterInstance;
use crate::timeline::sort_methods::SortMethod;

pub struct TimelineAgent {
	link: AgentLink<Self>,
	modal: Option<HandlerId>,
	choose_endpoints: Option<HandlerId>,
	timeline_container: Option<HandlerId>,
	endpoint_agent: Dispatcher<EndpointAgent>,
}

pub enum Request {
	RegisterModal,
	RegisterChooseEndpoints,
	RegisterTimelineContainer,
	AddTimeline,
	AddUserTimeline(&'static str, String),
	SetMainTimeline(TimelineId),
	SetMainContainer(Container),
	SetMainColumnCount(u8),
	RemoveTimeline(TimelineId),
	LoadStorageTimelines,
	LoadedStorageTimelines(Vec<Vec<TimelineEndpointWrapper>>),
}

pub enum Response {
	AddTimeline,
	AddBlankTimeline,
	AddUserTimeline(&'static str, String),
	SetMainTimeline(TimelineId),
	SetMainContainer(Container),
	SetMainColumnCount(u8),
	RemoveTimeline(TimelineId),
	CreateTimelines(Vec<TimelinePropsClosure>),
}

#[derive(Serialize, Deserialize)]
pub struct SoshalTimelineStorage {
	title: String,
	#[serde(default)]
	container: Container,
	#[serde(default)]
	endpoints: Vec<EndpointSerialized>,
	#[serde(default = "default_1")]
	column_count: u8,
	#[serde(default = "default_1")]
	width: u8,
	#[serde(default)]
	filters: Option<HashSet<FilterInstance>>,
	#[serde(default = "default_sort_method")]
	sort_method: Option<(SortMethod, bool)>,
	#[serde(default)]
	compact: bool,
	#[serde(default)]
	animated_as_gifs: bool,
	#[serde(default)]
	hide_text: bool,
}

impl Agent for TimelineAgent {
	type Reach = AgentContext<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			modal: None,
			choose_endpoints: None,
			timeline_container: None,
			endpoint_agent: EndpointAgent::dispatcher(),
			link,
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::RegisterModal => self.modal = Some(id),
			Request::RegisterChooseEndpoints => self.choose_endpoints = Some(id),
			Request::RegisterTimelineContainer => self.timeline_container = Some(id),
			//TODO Less confusing name AddTimeline
			Request::AddTimeline => {
				if let Some(choose_endpoints) = self.choose_endpoints {
					self.link.respond(choose_endpoints, Response::AddBlankTimeline)
				}
				if let Some(modal) = self.modal {
					self.link.respond(modal, Response::AddBlankTimeline)
				}
			}
			Request::AddUserTimeline(service, username) => {
				if let Some(choose_endpoints) = self.choose_endpoints {
					self.link.respond(choose_endpoints, Response::AddUserTimeline(service.clone(), username.clone()))
				}
				if let Some(modal) = self.modal {
					self.link.respond(modal, Response::AddUserTimeline(service, username))
				}
			}
			Request::SetMainTimeline(id) => {
				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::SetMainTimeline(id));
				}
			}
			Request::SetMainContainer(container) => {
				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::SetMainContainer(container));
				}
			}
			Request::SetMainColumnCount(count) => {
				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::SetMainColumnCount(count));
				}
			}
			Request::RemoveTimeline(id) => {
				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::RemoveTimeline(id));
				}
			}
			Request::LoadStorageTimelines => {
				if let Some(_timeline_container) = self.timeline_container {
					let storage: Vec<SoshalTimelineStorage> = match gloo_storage::LocalStorage::get("SoshalThingYew Timelines") {
						Ok(storage) => storage,
						Err(err) => {
							if let StorageError::SerdeError(_) | StorageError::JsError(_) =  err {
								log_warn!("Failed to parse timeline storage", err);
							}

							Vec::new()
						}
					};

					let callbacks = storage.into_iter().map(|t| {
						let name = t.title.clone();
						let width = t.width;
						let column_count = t.column_count;
						let container = t.container;
						let filters = t.filters;
						let sort_method = t.sort_method;
						let compact = t.compact;
						let animated_as_gifs = t.animated_as_gifs;
						let hide_text = t.hide_text;

						(
							t.endpoints,
							Box::new(move |id, endpoints|
								 yew::props! { TimelineProps {
									name,
									id,
									endpoints,
									container,
									width,
									column_count,
									filters,
									sort_method,
									compact,
									animated_as_gifs,
									hide_text,
								}}
							) as TimelinePropsEndpointsClosure,
						)
					}).collect();

					self.endpoint_agent.send(EndpointRequest::BatchNewEndpoints(callbacks));
				}
			}
			Request::LoadedStorageTimelines(timelines) => {
				log::debug!("Received endpoints for {} timelines", timelines.len());
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		if self.modal == Some(id) {
			self.modal = None
		}else if self.choose_endpoints == Some(id) {
			self.choose_endpoints = None
		}else if self.timeline_container == Some(id) {
			self.timeline_container = None
		}
	}
}

fn default_1() -> u8 {
	1
}

fn default_sort_method() -> Option<(SortMethod, bool)> {
	Some((SortMethod::default(), true))
}
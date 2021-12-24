use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext, Dispatcher, Dispatched};
use gloo_storage::Storage;
use serde::{Serialize, Deserialize};

use super::{TimelineId, Props as TimelineProps, Container};
use crate::services::endpoint_agent::{TimelineEndpoints, Request as EndpointRequest, EndpointAgent};
use crate::{TimelinePropsClosure, TimelinePropsEndpointsClosure};

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
	AddUserTimeline(String, String),
	SetMainTimeline(TimelineId),
	RemoveTimeline(TimelineId),
	LoadStorageTimelines,
	LoadedStorageTimelines(Vec<TimelineEndpoints>),
}

pub enum Response {
	AddTimeline,
	AddBlankTimeline,
	AddUserTimeline(String, String),
	SetMainTimeline(TimelineId),
	RemoveTimeline(TimelineId),
	CreateTimelines(Vec<TimelinePropsClosure>),
}

#[derive(Serialize, Deserialize)]
pub struct EndpointStorage {
	pub service: String,
	pub endpoint_type: usize,
	pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct TimelineEndpointsStorage {
	pub start: Vec<EndpointStorage>,
	pub refresh: Vec<EndpointStorage>,
}

#[derive(Serialize, Deserialize)]
pub struct SoshalTimelineStorage {
	title: String,
	container: String,
	endpoints: TimelineEndpointsStorage,
	column_count: u8,
	width: u8,
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
			Request::RemoveTimeline(id) => {
				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::RemoveTimeline(id));
				}
			}
			Request::LoadStorageTimelines => {
				if let Some(_timeline_container) = self.timeline_container {
					let storage: Vec<SoshalTimelineStorage> = gloo_storage::LocalStorage::get("SoshalThingYew Timelines").unwrap_or_default();

					let callbacks = storage.into_iter().map(|t| {
						let name = t.title.clone();
						let width = t.width.clone();
						let column_count = t.column_count.clone();
						let container = match Container::from(&t.container) {
							Ok(c) => c,
							Err(err) => {
								log::error!("{:?}", err);
								Container::Column
							}
						};
						/*match name {
							"Column" => Ok(Container::Column),
							"Row" => Ok(Container::Row),
							"Masonry" => Ok(Container::Masonry),
							_ => Err(format!("Couldn't parse container \"{}\".", name).into()),
						}*/
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
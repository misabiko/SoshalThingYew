use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext};
use gloo_storage::Storage;
use serde::{Serialize, Deserialize};

use super::TimelineId;
use crate::timeline::{Props as TimelineProps};
use crate::services::endpoints::TimelineEndpoints;
use crate::TimelinePropsClosure;

pub struct TimelineAgent {
	link: AgentLink<Self>,
	modal: Option<HandlerId>,
	choose_endpoints: Option<HandlerId>,
	timeline_container: Option<HandlerId>,
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
	service: String,
	endpoint_type: u8,
	params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct TimelineEndpointsStorage {
	start: Vec<EndpointStorage>,
	refresh: Vec<EndpointStorage>,
}

#[derive(Serialize, Deserialize)]
pub struct SoshalTimelineStorage {
	title: String,
	container: String,
}

impl Agent for TimelineAgent {
	type Reach = AgentContext<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			modal: None,
			choose_endpoints: None,
			timeline_container: None,
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
				if let Some(timeline_container) = self.timeline_container {
					let storage: Vec<SoshalTimelineStorage> = gloo_storage::LocalStorage::get("SoshalThingYew Timelines").unwrap_or_default();

					let props = storage.into_iter().map(|t| {
						let name = t.title.clone();
						Box::new(|id|
							yew::props! { TimelineProps {
								name,
								id,
								endpoints: TimelineEndpoints::default(),
							}}
						) as TimelinePropsClosure
					}).collect();

					self.link.respond(timeline_container, Response::CreateTimelines(props));
				}
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
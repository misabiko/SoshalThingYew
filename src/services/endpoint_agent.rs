use std::rc::{Rc, Weak};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;
use yew_agent::{Agent, Context as AgentContext, AgentLink, HandlerId};
use std::cell::RefCell;
use serde_json::json;
use gloo_timers::callback::Interval;

use super::{Endpoint, EndpointSerialized, RateLimit};
use crate::error::{Error, FetchResult};
use crate::articles::ArticleData;
use crate::timeline::agent::TimelineEndpointsSerialized;
use crate::timeline::filters::{Filter, deserialize_filters};
use crate::{TimelineId, TimelinePropsEndpointsClosure};

pub type EndpointId = i32;

#[derive(Clone)]
pub struct TimelineEndpointWrapper {
	pub id: EndpointId,
	pub filters: Vec<Filter>,
}

impl From<EndpointId> for TimelineEndpointWrapper {
	fn from(id: EndpointId) -> Self {
		Self { id, filters: Vec::new() }
	}
}

impl PartialEq for TimelineEndpointWrapper {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

//Maybe HashMap<RefreshTime, HashSet<EndpointId>> ?
#[derive(Clone, PartialEq, Default)]
pub struct TimelineEndpoints {
	pub start: Vec<TimelineEndpointWrapper>,
	pub refresh: Vec<TimelineEndpointWrapper>,
}

impl TimelineEndpoints {
	pub fn new_with_endpoint_both(endpoint_id: EndpointId) -> Self {
		let endpoints = vec![endpoint_id.into()];
		Self {
			start: endpoints.clone(),
			refresh: endpoints,
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum RefreshTime {
	Start,
	OnRefresh,
}

pub enum Msg {
	Refreshed(RefreshTime, EndpointId, (Vec<Rc<RefCell<dyn ArticleData>>>, Option<RateLimit>)),
	RefreshFail(Error),
	UpdatedState,
	AutoRefreshEndpoint(EndpointId),
	ResetAutoRefresh(EndpointId),
}

pub enum Request {
	InitTimeline(TimelineId, Rc<RefCell<TimelineEndpoints>>, Callback<Vec<Weak<RefCell<dyn ArticleData>>>>),
	RemoveTimeline(TimelineId),
	Refresh(Weak<RefCell<TimelineEndpoints>>),
	LoadBottom(Weak<RefCell<TimelineEndpoints>>),
	RefreshEndpoint(EndpointId, RefreshTime),
	EndpointFetchResponse(RefreshTime, EndpointId, FetchResult<Vec<Rc<RefCell<dyn ArticleData>>>>),
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<dyn ArticleData>>>),
	AddEndpoint(Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>),
	BatchAddEndpoints(Vec<Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>>, Vec<Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>>, Callback<TimelineEndpoints>),
	InitService(String, EndpointConstructors),
	UpdateRateLimit(EndpointId, RateLimit),
	BatchNewEndpoints(Vec<(TimelineEndpointsSerialized, TimelinePropsEndpointsClosure)>),
	RegisterTimelineContainer,
	GetState,
	StartAutoRefresh(EndpointId),
	StopAutoRefresh(EndpointId),
	SetAutoRefreshInterval(EndpointId, u32),
}

pub enum Response {
	UpdatedState(HashMap<String, EndpointConstructors>, Vec<EndpointView>),
	BatchRequestResponse(Vec<(TimelineEndpoints, TimelinePropsEndpointsClosure)>)
}

#[derive(Clone)]
pub struct EndpointConstructor {
	pub name: &'static str,
	pub param_template: Vec<(&'static str, serde_json::Value)>,
	pub callback: Rc<dyn Fn(EndpointId, serde_json::Value) -> Box<dyn Endpoint>>
}

impl EndpointConstructor {
	pub fn default_params(&self) -> serde_json::Value {
		let mut params = json!({});
		for (name, value) in &self.param_template {
			params[name] = value.clone();
		}
		params
	}
}

#[derive(Clone)]
pub struct EndpointConstructors {
	pub endpoint_types: Vec<EndpointConstructor>,
	pub user_endpoint: Option<usize>,
}

#[derive(Clone)]
pub struct EndpointView {
	pub id: EndpointId,
	pub name: String,
	pub ratelimit: Option<RateLimit>,
	pub is_autorefreshing: bool,
	pub autorefresh_interval: u32,
}

pub struct EndpointInfo {
	endpoint: Box<dyn Endpoint>,
	interval_id: Option<Interval>,
	interval: u32,
}

impl EndpointInfo {
	fn new(endpoint: Box<dyn Endpoint>) -> Self {
		Self {
			interval: endpoint.default_interval(),
			interval_id: None,
			endpoint,
		}
	}
}

pub struct EndpointAgent {
	link: AgentLink<Self>,
	endpoint_counter: EndpointId,
	pub endpoints: HashMap<EndpointId, EndpointInfo>,
	pub timelines: HashMap<TimelineId, (Weak<RefCell<TimelineEndpoints>>, Callback<Vec<Weak<RefCell<dyn ArticleData>>>>)>,
	pub services: HashMap<String, EndpointConstructors>,
	subscribers: HashSet<HandlerId>,
	timeline_container: Option<HandlerId>,
}

impl Agent for EndpointAgent {
	type Reach = AgentContext<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			endpoint_counter: i32::MIN,
			endpoints: HashMap::new(),
			timelines: HashMap::new(),
			services: HashMap::new(),
			subscribers: HashSet::new(),
			timeline_container: None,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::Refreshed(refresh_time, endpoint_id, response) => {
				log::debug!("{} articles for {}", &response.0.len(), self.endpoints[&endpoint_id].endpoint.name());
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				info.endpoint.add_articles(response.0.iter().map(|article| Rc::downgrade(&article)).collect());
				if let Some(ratelimit) = response.1 {
					info.endpoint.update_ratelimit(ratelimit);
				}

				for (_timeline_id, timeline) in &self.timelines {
					let timeline_strong = timeline.0.upgrade().unwrap();
					let borrow = timeline_strong.borrow();
					let endpoints = match &refresh_time {
						RefreshTime::OnRefresh => &borrow.refresh,
						RefreshTime::Start => &borrow.start,
					};

					if let Some(endpoint_wrapper) = endpoints.iter().find(|e| e.id == endpoint_id) {
						timeline.1.emit(response.0.iter()
							.map(|article| Rc::downgrade(&article))
							.filter(|article|
								endpoint_wrapper.filters.iter().all(|filter|
									filter.enabled && (filter.predicate)(article, &filter.inverted)
								)
							)
							.collect());
					}
				}

				self.link.send_message(Msg::UpdatedState);
			}
			Msg::RefreshFail(err) => {
				log::error!("Failed to fetch:\n{:?}", err);
			}
			Msg::UpdatedState => {
				for sub in &self.subscribers {
					if sub.is_respondable() {
						self.send_state(sub);
					}
				}
			}
			Msg::AutoRefreshEndpoint(endpoint_id) => self.endpoints.get_mut(&endpoint_id).unwrap().endpoint.refresh(RefreshTime::OnRefresh),
			Msg::ResetAutoRefresh(endpoint_id) => {
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				if info.interval_id.is_some() {
					let id_c = endpoint_id.clone();
					let callback = self.link.callback(move |_| Msg::AutoRefreshEndpoint(id_c));
					let new_interval = Interval::new(info.interval, move || {
						log::debug!("Refreshing {}", &id_c);
						callback.emit(());
					});

					Interval::cancel(std::mem::replace(&mut info.interval_id, Some(new_interval)).unwrap());
					self.link.send_message(Msg::UpdatedState);

				}
			}
		}
	}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::InitTimeline(timeline_id, endpoints, callback) => {
				self.timelines.insert(timeline_id, (Rc::downgrade(&endpoints), callback));

				for timeline_endpoint in &endpoints.borrow().start {
					let info = self.endpoints.get_mut(&timeline_endpoint.id).unwrap();
					if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						info.endpoint.refresh(RefreshTime::Start);
					}else {
						log::warn!("Can't refresh {}", &info.endpoint.name());
					}
				}
			},
			Request::RemoveTimeline(id) => {
				self.timelines.remove(&id);
			}
			Request::Refresh(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				for timeline_endpoint in endpoints.borrow().refresh.clone() {
					let info = self.endpoints.get_mut(&timeline_endpoint.id).unwrap();
					if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						info.endpoint.refresh(RefreshTime::OnRefresh);
						self.link.send_message(Msg::ResetAutoRefresh(*info.endpoint.id()));
					}else {
						log::warn!("Can't refresh {}", &info.endpoint.name());
					}
				}
			}
			Request::LoadBottom(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				for timeline_endpoint in endpoints.borrow().refresh.clone() {
					let info = self.endpoints.get_mut(&timeline_endpoint.id).unwrap();
					if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						info.endpoint.load_bottom(RefreshTime::OnRefresh);
						self.link.send_message(Msg::ResetAutoRefresh(*info.endpoint.id()));
					}else {
						log::warn!("Can't refresh {}", &info.endpoint.name());
					}
				}
			}
			Request::RefreshEndpoint(endpoint_id, refresh_time) => {
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
					info.endpoint.refresh(refresh_time);
					self.link.send_message(Msg::ResetAutoRefresh(*info.endpoint.id()));
				}else {
					log::warn!("Can't refresh {}", &info.endpoint.name());
				}
			}
			Request::EndpointFetchResponse(refresh_time, endpoint_id, response) => {
				match response {
					Ok(response) => self.link.send_message(Msg::Refreshed(refresh_time, endpoint_id, response)),
					Err(err) => self.link.send_message(Msg::RefreshFail(err)),
				};
			}
			Request::AddArticles(refresh_time, endpoint_id, articles) =>
				self.link.send_message(Msg::Refreshed(refresh_time, endpoint_id, (articles, None))),
			Request::AddEndpoint(endpoint_closure) => {
				self.endpoints.insert(self.endpoint_counter, EndpointInfo::new(endpoint_closure(self.endpoint_counter)));
				self.endpoint_counter += 1;

				self.link.send_message(Msg::UpdatedState);
			},
			Request::BatchAddEndpoints(start_closures, refresh_closures, callback) => {
				let start = start_closures.into_iter().map(|closure| {
					let id = self.endpoint_counter.clone();
					self.endpoints.insert(id.clone(), EndpointInfo::new((closure)(self.endpoint_counter)));
					self.endpoint_counter += 1;
					id.into()
				}).collect();
				let refresh = refresh_closures.into_iter().map(|closure| {
					let id = self.endpoint_counter.clone();
					self.endpoints.insert(id.clone(), EndpointInfo::new((closure)(self.endpoint_counter)));
					self.endpoint_counter += 1;
					id.into()
				}).collect();

				callback.emit(TimelineEndpoints { start, refresh });
				self.link.send_message(Msg::UpdatedState);
			},
			Request::InitService(name, endpoints) => {
				self.services.insert(name, endpoints);

				self.link.send_message(Msg::UpdatedState);
			},
			Request::UpdateRateLimit(endpoint_id, ratelimit) => {
				self.endpoints.get_mut(&endpoint_id).unwrap().endpoint.update_ratelimit(ratelimit)
			},
			Request::BatchNewEndpoints(timelines) => {
				let endpoints: Vec<(TimelineEndpoints, TimelinePropsEndpointsClosure)> = timelines.into_iter().map(|(constructor, callback)| {
					let start = constructor.start.iter()
						.map(|e| self.find_endpoint_or_create(e))
						.collect();
					let refresh = constructor.refresh.iter()
						.map(|e| self.find_endpoint_or_create(e))
						.collect();

					(TimelineEndpoints { start, refresh }, callback)
				}).collect();


				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::BatchRequestResponse(endpoints));
				}
			},
			Request::RegisterTimelineContainer => self.timeline_container = Some(id),
			Request::GetState => self.send_state(&id),
			Request::StartAutoRefresh(endpoint_id) => {
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				if let None = info.interval_id {
					let id_c = endpoint_id.clone();
					let id_c_2 = endpoint_id.clone();
					let callback = self.link.callback(move |_| Msg::AutoRefreshEndpoint(id_c));
					info.interval_id = Some(Interval::new(info.interval, move || {
						log::debug!("Refreshing {}", &id_c_2);
						callback.emit(());
					}));
					self.link.send_message(Msg::UpdatedState);
				}else {
					log::warn!("Auto refresh for {} is already on.", &endpoint_id);
				}
			},
			Request::StopAutoRefresh(endpoint_id) => {
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				if info.interval_id.is_some() {
					Interval::cancel(std::mem::replace(&mut info.interval_id, None).unwrap());
					self.link.send_message(Msg::UpdatedState);
				}else {
					log::warn!("Auto refresh for {} is not on.", &endpoint_id);
				}
			}
			Request::SetAutoRefreshInterval(endpoint_id, interval) => {
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				info.interval = interval;
				self.link.send_message(Msg::UpdatedState);
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);

		if self.timeline_container == Some(id) {
			self.timeline_container = None
		}
	}
}

impl EndpointAgent {
	fn endpoint_from_constructor(&self, storage: &EndpointSerialized) -> Option<EndpointId> {
		self.endpoints.iter().find_map(|(id, endpoint)| match endpoint.endpoint.eq_storage(storage) {
			true => Some(id.clone()),
			false => None
		})
	}

	fn find_endpoint_or_create(&mut self, storage: &EndpointSerialized) -> TimelineEndpointWrapper {
		let id = match self.endpoint_from_constructor(storage) {
			Some(id) => id,
			None => {
				let constructor = self.services[&storage.service].endpoint_types[storage.endpoint_type.clone()].clone();
				let params = storage.params.clone();

				let id = self.endpoint_counter.clone();
				self.endpoints.insert(self.endpoint_counter, EndpointInfo::new((constructor.callback)(id, params.clone())));
				self.endpoint_counter += 1;

				id
			}
		};

		let filters = deserialize_filters(&storage.filters);

		TimelineEndpointWrapper { id, filters }
	}

	fn send_state(&self, id: &HandlerId) {
		self.link.respond(*id, Response::UpdatedState(self.services.clone(), self.endpoints.iter().map(|(id, e)| EndpointView {
			id: id.clone(),
			name: e.endpoint.name(),
			ratelimit: e.endpoint.ratelimit().cloned(),
			is_autorefreshing: e.interval_id.is_some(),
			autorefresh_interval: e.interval.clone(),
		}).collect()));
	}
}
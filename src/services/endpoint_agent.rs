use std::rc::{Rc, Weak};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;
use yew_agent::{Agent, Context as AgentContext, AgentLink, HandlerId, Dispatcher, Dispatched};
use std::cell::RefCell;
use serde_json::json;
use gloo_timers::callback::Interval;

use super::{Endpoint, EndpointSerialized, RateLimit};
use crate::error::{Result, Error, RatelimitedResult};
use crate::articles::{ArticleRc, ArticleWeak};
use crate::choose_endpoints::EndpointForm;
use crate::timeline::{
	TimelineId,
	timeline_container::{TimelineCreationMode, TimelinePropsEndpointsClosure},
	filters::FilterCollection
};
use crate::notifications::{NotificationAgent, Request as NotificationRequest, Notification};

pub type EndpointId = i32;

//TODO Split agent stuff in separate module

#[derive(Clone, PartialEq)]
pub struct TimelineEndpointWrapper {
	pub id: EndpointId,
	pub on_start: bool,
	pub on_refresh: bool,
	pub filters: FilterCollection,
}

impl TimelineEndpointWrapper {
	pub fn new(id: EndpointId, on_start: bool, on_refresh: bool) -> Self {
		Self { id, on_start, on_refresh, filters: FilterCollection::default() }
	}

	pub fn new_both(id: EndpointId) -> Self {
		Self {
			id,
			on_start: true,
			on_refresh: true,
			filters: FilterCollection::default()
		}
	}
}

//TODO Replace with per_timeline boolean?
#[derive(Clone, Copy, PartialEq)]
pub enum RefreshTime {
	Start,
	OnRefresh,
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

/// The constructors for a service's endpoints
#[derive(Clone)]
pub struct EndpointConstructorCollection {
	pub constructors: Vec<EndpointConstructor>,
	/// Index of the endpoint used to query a user's articles
	pub user_endpoint_index: Option<usize>,
}

#[derive(Clone)]
pub struct EndpointView {
	pub id: EndpointId,
	pub name: String,
	pub ratelimit: Option<RateLimit>,
	pub is_autorefreshing: bool,
	pub autorefresh_interval: u32,
	pub shared: bool,
}

/// Additional data common to all endpoints
pub struct EndpointInfo {
	endpoint: Box<dyn Endpoint>,
	shared: bool,
	interval_id: Option<Interval>,
	interval: u32,
}

impl EndpointInfo {
	fn new(endpoint: Box<dyn Endpoint>, shared: bool) -> Self {
		Self {
			interval: endpoint.default_interval(),
			interval_id: None,
			endpoint,
			shared,
		}
	}
}

pub enum TimelineCreationRequest {
	NameEndpoints(String),
	Props(TimelinePropsEndpointsClosure)
}

pub struct BatchEndpointAddClosure {
	pub closure: Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>,
	pub on_start: bool,
	pub on_refresh: bool,
	pub shared: bool,
}

pub struct EndpointAgent {
	link: AgentLink<Self>,
	endpoint_counter: EndpointId,
	pub endpoints: HashMap<EndpointId, EndpointInfo>,
	pub timelines: HashMap<TimelineId, (Weak<RefCell<Vec<TimelineEndpointWrapper>>>, Callback<Vec<ArticleWeak>>)>,
	pub services: HashMap<&'static str, EndpointConstructorCollection>,
	subscribers: HashSet<HandlerId>,
	timeline_container: Option<HandlerId>,
	notification_agent: Dispatcher<NotificationAgent>,
}

pub enum Msg {
	Refreshed(RefreshTime, EndpointId, (Vec<ArticleRc>, Option<RateLimit>)),
	RefreshFail(EndpointId, Error),
	UpdatedState,
	AutoRefreshEndpoint(EndpointId),
	ResetAutoRefresh(EndpointId),
}

pub enum Request {
	InitTimeline(TimelineId, Rc<RefCell<Vec<TimelineEndpointWrapper>>>, Callback<Vec<ArticleWeak>>),
	RemoveTimeline(TimelineId),
	Refresh(Weak<RefCell<Vec<TimelineEndpointWrapper>>>),
	LoadBottom(Weak<RefCell<Vec<TimelineEndpointWrapper>>>),
	LoadTop(Weak<RefCell<Vec<TimelineEndpointWrapper>>>),
	RefreshEndpoint(EndpointId, RefreshTime),
	EndpointFetchResponse(RefreshTime, EndpointId, RatelimitedResult<Vec<ArticleRc>>),
	AddArticles(RefreshTime, EndpointId, Vec<ArticleRc>),
	AddEndpoint {
		id_to_endpoint: Box<dyn FnOnce(EndpointId) -> Box<dyn Endpoint>>,
		shared: bool,
	},
	AddUserEndpoint {
		service: &'static str,
		username: String,
		shared: bool,
		callback: Callback<EndpointId>,
	},
	AddEndpointFromForm(EndpointForm, Callback<EndpointId>),
	BatchAddEndpoints(Vec<BatchEndpointAddClosure>, TimelineCreationRequest),
	InitService(&'static str, EndpointConstructorCollection),
	UpdateRateLimit(EndpointId, RateLimit),
	BatchNewEndpoints(Vec<(Vec<EndpointSerialized>, TimelinePropsEndpointsClosure)>),
	RegisterTimelineContainer,
	GetState,
	StartAutoRefresh(EndpointId),
	StopAutoRefresh(EndpointId),
	SetAutoRefreshInterval(EndpointId, u32),
}

pub enum Response {
	UpdatedState(HashMap<&'static str, EndpointConstructorCollection>, Vec<EndpointView>),
	BatchRequestResponse(Vec<(Vec<TimelineEndpointWrapper>, TimelinePropsEndpointsClosure)>),
	AddTimeline(TimelineCreationMode, bool),
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
			notification_agent: NotificationAgent::dispatcher(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::Refreshed(refresh_time, endpoint_id, response) => {
				log::trace!("{} articles for {}", &response.0.len(), self.endpoints[&endpoint_id].endpoint.name());
				let info = self.endpoints.get_mut(&endpoint_id).unwrap();
				info.endpoint.add_articles(response.0.iter().map(|article| Rc::downgrade(&article)).collect());
				if let Some(ratelimit) = response.1 {
					info.endpoint.update_ratelimit(ratelimit);
				}

				for (_timeline_id, timeline) in &self.timelines {
					let timeline_strong = timeline.0.upgrade().unwrap();
					let borrow = timeline_strong.borrow();
					let endpoints: Vec<&TimelineEndpointWrapper> = borrow.iter().filter(|e| match refresh_time {
						RefreshTime::Start => e.on_start,
						RefreshTime::OnRefresh => e.on_refresh,
					}).collect();

					if let Some(endpoint_wrapper) = endpoints.iter().find(|e| e.id == endpoint_id) {
						timeline.1.emit(response.0.iter()
							.map(|article| Rc::downgrade(&article))
							.filter(|article|
								endpoint_wrapper.filters.iter().all(|instance|
									instance.enabled && {
										let strong = article.upgrade();
										if let Some(a) = strong {
											instance.filter.filter(&a.borrow()) != instance.inverted
										}else {
											false
										}
									}
								)
							)
							.collect());
					}
				}

				self.link.send_message(Msg::UpdatedState);
			}
			Msg::RefreshFail(endpoint_id, err) => {
				//TODO macrofy → log_error(err)
				log::error!("{}", &err);
				self.notification_agent.send(NotificationRequest::Notify(
					Some(format!("Endpoint{}RefreshFail", endpoint_id)),
					Notification::Error(err),
				));
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
					let id_c = endpoint_id;
					let callback = self.link.callback(move |_| Msg::AutoRefreshEndpoint(id_c));
					let new_interval = Interval::new(info.interval, move || {
						log::trace!("Refreshing {}", &id_c);
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

				for timeline_endpoint in endpoints.borrow().iter().filter(|e| e.on_start) {
					let info = self.endpoints.get_mut(&timeline_endpoint.id).unwrap();
					if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						info.endpoint.refresh(RefreshTime::Start);
					}else {
						log::warn!("Can't refresh {}", &info.endpoint.name());
					}
				}
			},
			Request::RemoveTimeline(id) => {
				let endpoints = self.timelines.get(&id).unwrap().0.upgrade().unwrap();
				let endpoints = endpoints.borrow();
				let mut should_update_state = false;

				for wrapper in endpoints.iter() {
					let shared = self.endpoints[&wrapper.id].shared;
					if !shared {
						self.endpoints.remove(&wrapper.id);
						should_update_state = true;
					}
				}

				self.timelines.remove(&id);

				if should_update_state {
					self.link.send_message(Msg::UpdatedState);
				}
			}
			Request::Refresh(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				for timeline_endpoint in endpoints.borrow().iter().filter(|e| e.on_refresh) {
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
				for timeline_endpoint in endpoints.borrow().iter().filter(|e| e.on_refresh) {
					let info = self.endpoints.get_mut(&timeline_endpoint.id).unwrap();
					if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						info.endpoint.load_bottom(RefreshTime::OnRefresh);
						self.link.send_message(Msg::ResetAutoRefresh(*info.endpoint.id()));
					}else {
						log::warn!("Can't refresh {}", &info.endpoint.name());
					}
				}
			}
			Request::LoadTop(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				for timeline_endpoint in endpoints.borrow().iter().filter(|e| e.on_refresh) {
					let info = self.endpoints.get_mut(&timeline_endpoint.id).unwrap();
					if info.endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						info.endpoint.load_top(RefreshTime::OnRefresh);
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
					Err(err) => self.link.send_message(Msg::RefreshFail(endpoint_id, err)),
				};
			}
			Request::AddArticles(refresh_time, endpoint_id, articles) =>
				self.link.send_message(Msg::Refreshed(refresh_time, endpoint_id, (articles, None))),
			Request::AddEndpoint { id_to_endpoint, shared } => {
				self.endpoints.insert(self.endpoint_counter, EndpointInfo::new(id_to_endpoint(self.endpoint_counter), shared));
				self.endpoint_counter += 1;

				self.link.send_message(Msg::UpdatedState);
			}
			Request::AddUserEndpoint { service, username, shared, callback } => {
				if let Some(endpoint_type) = self.services[&service].user_endpoint_index {
					//let EndpointForm { service, endpoint_type, refresh_time: _, params, mut filters, shared } = form;

					let constructor = self.services[&service].constructors[endpoint_type].clone();

					let params = json!({
						"username": username,
					});
					//"include_retweets": true,
					//"include_replies": true,

					self.link.send_input(Request::AddEndpoint {
						id_to_endpoint: Box::new(move |id| {
							callback.emit(id);
							(constructor.callback)(id, params.clone())
						}),
						shared,
					});
				} else {
					log::warn!("{} doesn't have a user endpoint.", service)
				}
			}
			//TODO Batch by default
			Request::AddEndpointFromForm(form, callback) => {
				let EndpointForm { service, endpoint_type, params, shared, .. } = form;

				let constructor = self.services[&service].constructors[endpoint_type].clone();

				let params = params.clone();
				self.link.send_input(Request::AddEndpoint {
					id_to_endpoint: Box::new(move |id| {
						callback.emit(id);
						(constructor.callback)(id, params.clone())
					}),
					shared,
				});
			}
			Request::BatchAddEndpoints(closures, timeline_creation_request) => {
				if let Some(timeline_container) = self.timeline_container {
					let endpoints = closures.into_iter().map(|BatchEndpointAddClosure {closure, on_start, on_refresh, shared}| {
						let id = self.endpoint_counter;
						self.endpoints.insert(id, EndpointInfo::new((closure)(id), shared));
						self.endpoint_counter += 1;
						TimelineEndpointWrapper::new(id, on_start, on_refresh)
					}).collect();

					self.link.respond(timeline_container.clone(), match timeline_creation_request {
						TimelineCreationRequest::NameEndpoints(name) => Response::AddTimeline(TimelineCreationMode::NameEndpoints(name, endpoints), false),
						TimelineCreationRequest::Props(props) => Response::AddTimeline(TimelineCreationMode::Props(Box::new(|timeline_id| (props)(timeline_id, endpoints))), false),
					});
					self.link.send_message(Msg::UpdatedState);
				}else {
					log::error!("BatchAddEndpoints: Model not yet registered to EndpointAgent");
				}
			},
			Request::InitService(name, endpoints) => {
				self.services.insert(name, endpoints);

				self.link.send_message(Msg::UpdatedState);
			},
			Request::UpdateRateLimit(endpoint_id, ratelimit) => {
				self.endpoints.get_mut(&endpoint_id).unwrap().endpoint.update_ratelimit(ratelimit)
			},
			Request::BatchNewEndpoints(endpoints) => {
				let endpoints: Vec<(Vec<TimelineEndpointWrapper>, TimelinePropsEndpointsClosure)> = endpoints.into_iter().map(|(constructor, callback)| {
					let endpoints = constructor.iter()
						.filter_map(|e|
							match self.find_endpoint_or_create(e, e.on_start, e.on_refresh) {
								Ok(e) => Some(e),
								Err(err) => {
									log::error!("{}", err);
									None
								}
							}
						)
						.collect();

					(endpoints, callback)
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
						log::trace!("Refreshing {}", &id_c_2);
						callback.emit(());
					}));
					self.link.send_message(Msg::UpdatedState);
				}else {
					log::trace!("Auto refresh for {} is already on.", &endpoint_id);
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

	fn find_endpoint_or_create(&mut self, serialized: &EndpointSerialized, on_start: bool, on_refresh: bool) -> Result<TimelineEndpointWrapper> {
		match self.endpoint_from_constructor(serialized) {
			Some(id) => Ok(id),
			None => {
				match self.services.get(&serialized.service.as_str()) {
					None => Err(format!("{} isn't registered as a service. Available: {:?}", &serialized.service, self.services.keys()).into()),
					Some(service) => {
						let constructor = service.constructors[serialized.endpoint_type].clone();
						let params = serialized.params.clone();

						let id = self.endpoint_counter;
						self.endpoints.insert(self.endpoint_counter, EndpointInfo::new((constructor.callback)(id, params.clone()), true));
						self.endpoint_counter += 1;

						Ok(id)
					}
				}
			}
		}
			.map(|id| {
				if serialized.auto_refresh {
					self.link.send_input(Request::StartAutoRefresh(id))
				}

				TimelineEndpointWrapper { id, on_start, on_refresh, filters: serialized.filters.clone() }
			})
	}

	fn send_state(&self, id: &HandlerId) {
		self.link.respond(*id, Response::UpdatedState(self.services.clone(), self.endpoints.iter().map(|(id, e)| EndpointView {
			id: id.clone(),
			name: e.endpoint.name(),
			ratelimit: e.endpoint.ratelimit().cloned(),
			is_autorefreshing: e.interval_id.is_some(),
			autorefresh_interval: e.interval,
			shared: e.shared,
		}).collect()));
	}
}
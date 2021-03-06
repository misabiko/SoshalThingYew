use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use std::rc::Weak;
use std::collections::HashMap;
use std::cell::RefCell;
use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use serde_json::Value;

use crate::services::endpoint_agent::{EndpointRequest, EndpointAgent, EndpointId, RefreshTime, EndpointConstructorCollection, EndpointResponse, EndpointView, TimelineEndpointWrapper};
use crate::components::{Dropdown, DropdownLabel};
use crate::timeline::{
	filters::{FiltersOptions, FilterCollection, FilterMsg},
	agent::{TimelineAgent, TimelineRequest, TimelineResponse},
};

pub struct EndpointForm {
	pub refresh_time: RefreshTime,
	pub service: &'static str,
	pub endpoint_type: usize,
	pub params: Value,
	pub filters: FilterCollection,
	pub shared: bool,
}

pub struct ChooseEndpoints {
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	_add_timeline_agent: Option<Box<dyn Bridge<TimelineAgent>>>,
	show_start_endpoint_dropdown: bool,
	show_refresh_endpoint_dropdown: bool,
	endpoint_views: HashMap<EndpointId, EndpointView>,
	endpoint_form: Option<EndpointForm>,
	services: HashMap<&'static str, EndpointConstructorCollection>,
}

pub enum ChooseEndpointsMsg {
	EndpointResponse(EndpointResponse),
	TimelineResponse(TimelineResponse),
	ToggleStartEndpointDropdown,
	ToggleRefreshEndpointDropdown,
	CreateNewEndpointForm(RefreshTime),
	SetFormService(&'static str),
	SetFormType(usize),
	SetFormParamValue((&'static str, Value), String),
	CreateEndpoint(bool),
	AddTimelineEndpoint(RefreshTime, EndpointId, FilterCollection),
	AddTimelineEndpointBoth(EndpointId, FilterCollection),
	RemoveTimelineEndpoint(RefreshTime, EndpointId),
	FormFilterMsg(FilterMsg),
	ExistingFilterMsg(usize, FilterMsg),
	ToggleFormShared,
}

#[derive(Properties, Clone)]
pub struct ChooseEndpointsProps {
	pub timeline_endpoints: Weak<RefCell<Vec<TimelineEndpointWrapper>>>,
	#[prop_or_default]
	pub inside_add_timeline: bool,
}

impl PartialEq for ChooseEndpointsProps {
	fn eq(&self, other: &Self) -> bool {
		self.timeline_endpoints.ptr_eq(&other.timeline_endpoints)
	}
}

type Msg = ChooseEndpointsMsg;
type Props = ChooseEndpointsProps;

impl Component for ChooseEndpoints {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let _add_timeline_agent = match ctx.props().inside_add_timeline {
			true => {
				let mut agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineResponse));
				agent.send(TimelineRequest::RegisterChooseEndpoints);
				Some(agent)
			}
			false => None,
		};

		Self {
			endpoint_agent: EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse)),
			_add_timeline_agent,
			show_start_endpoint_dropdown: false,
			show_refresh_endpoint_dropdown: false,
			endpoint_views: HashMap::new(),
			endpoint_form: None,
			services: HashMap::new(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::EndpointResponse(response) => match response {
				EndpointResponse::UpdatedState(services, endpoints) => {
					self.endpoint_views.clear();
					for endpoint in endpoints {
						self.endpoint_views.insert(endpoint.id, endpoint);
					}

					self.services = services;

					true
				}
				_ => false
			}
			Msg::TimelineResponse(response) => match response {
				TimelineResponse::AddBlankTimeline => {
					self.endpoint_form = None;
					true
				}
				TimelineResponse::AddUserTimeline(service, username) => {
					//TODO Default user timeline filters in options
					if let Some(endpoint_type) = self.services[&service].user_endpoint_index {
						self.endpoint_form = Some(EndpointForm {
							refresh_time: RefreshTime::Start,
							service,
							endpoint_type,
							params: json!({
								"username": username,
								"include_retweets": true,
								"include_replies": true,
							}),
							filters: FilterCollection::new(),
							shared: false,
						});
						true
					}else {
						log::warn!("{} doesn't have a user endpoint.", service);
						false
					}
				}
				_ => false
			}
			Msg::ToggleStartEndpointDropdown => {
				self.show_start_endpoint_dropdown = !self.show_start_endpoint_dropdown;
				true
			}
			Msg::ToggleRefreshEndpointDropdown => {
				self.show_refresh_endpoint_dropdown = !self.show_refresh_endpoint_dropdown;
				true
			}
			Msg::CreateNewEndpointForm(refresh_time) => {
				//TODO Abstract default service
				self.endpoint_form = Some(EndpointForm {
					refresh_time,
					service: "Twitter",
					endpoint_type: 0,
					params: self.services["Twitter"].constructors[0].default_params(),
					filters: FilterCollection::new(),
					shared: false,
				});
				true
			}
			Msg::SetFormService(name) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.service != name {
						form.params = self.services[&name].constructors[0].default_params();
						form.service = name;
						form.endpoint_type = 0;
						true
					} else {
						false
					}
				} else {
					false
				}
			}
			Msg::SetFormType(endpoint_type) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.endpoint_type != endpoint_type {
						form.endpoint_type = endpoint_type;
						form.params = self.services[&form.service].constructors[endpoint_type].default_params();
						true
					} else {
						false
					}
				} else {
					false
				}
			}
			Msg::SetFormParamValue((param, param_type), value) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.params[&param] != value {
						if let Value::Bool(_) = param_type {
							if let Value::Bool(prev) = form.params[&param] {
								form.params[&param] = Value::Bool(!prev);
							} else {
								form.params[&param] = Value::Bool(value != "on" && value != "true");
							}
						} else {
							form.params[&param] = Value::String(value);
						}
						true
					} else {
						false
					}
				} else {
					false
				}
			}
			Msg::CreateEndpoint(both) => {
				let mut created = false;
				if let Some(EndpointForm { service, endpoint_type, refresh_time, params, mut filters, shared }) = std::mem::take(&mut self.endpoint_form) {
					let constructor = self.services[&service].constructors[endpoint_type].clone();
					let refresh_time_c = refresh_time;
					let filters = std::mem::replace(&mut filters, FilterCollection::new());

					let callback = if both {
						ctx.link().callback_once(move |id| Msg::AddTimelineEndpointBoth(id, filters))
					} else {
						ctx.link().callback_once(move |id| Msg::AddTimelineEndpoint(refresh_time_c, id, filters))
					};
					let params = params.clone();
					self.endpoint_agent.send(EndpointRequest::AddEndpoint {
						id_to_endpoint: Box::new(move |id| {
							callback.emit(id);
							(constructor.callback)(id, params.clone())
						}),
						shared,
					});

					created = true;
				}

				self.endpoint_form = None;
				created
			}
			Msg::AddTimelineEndpoint(refresh_time, id, filters) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				timeline_endpoints.borrow_mut().push(match refresh_time {
					RefreshTime::Start => TimelineEndpointWrapper {
						id,
						on_start: true,
						on_refresh: false,
						filters,
					},
					RefreshTime::OnRefresh => TimelineEndpointWrapper {
						id,
						on_start: false,
						on_refresh: true,
						filters,
					},
				});
				true
			}
			Msg::AddTimelineEndpointBoth(id, filters) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				timeline_endpoints.borrow_mut().push(TimelineEndpointWrapper {
					id,
					on_start: true,
					on_refresh: true,
					filters,
				});
				true
			}
			Msg::RemoveTimelineEndpoint(refresh_time, id) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				let mut borrow = timeline_endpoints.borrow_mut();
				if let Some(index) = borrow.iter().position(|e| e.id == id) {
					match refresh_time {
						RefreshTime::Start => {
							if borrow[index].on_refresh {
								borrow[index].on_start = false;
							} else {
								borrow.remove(index);
							}
						}
						RefreshTime::OnRefresh => {
							if borrow[index].on_start {
								borrow[index].on_refresh = false;
							} else {
								borrow.remove(index);
							}
						}
					};
				}
				true
			}
			Msg::FormFilterMsg(msg) => self.endpoint_form.as_mut().unwrap().filters.update(msg),
			Msg::ExistingFilterMsg(endpoint_index, msg) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				let mut timeline_endpoints = timeline_endpoints.borrow_mut();
				timeline_endpoints[endpoint_index]
					.filters.update(msg)
			}
			Msg::ToggleFormShared => {
				if let Some(form) = &mut self.endpoint_form {
					form.shared = !form.shared;
					true
				} else {
					false
				}
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		//TODO Merge on_start and on_refresh lists
		html! {
			<>
				{ self.refresh_time_endpoints(ctx, RefreshTime::Start) }
				{ self.refresh_time_endpoints(ctx, RefreshTime::OnRefresh) }
			</>
		}
	}
}

impl ChooseEndpoints {
	fn refresh_time_endpoints(&self, ctx: &Context<Self>, refresh_time: RefreshTime) -> Html {
		let strong_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
		let endpoints_borrowed = strong_endpoints.borrow();
		let (label, endpoints): (String, Vec<&TimelineEndpointWrapper>) = match refresh_time {
			RefreshTime::Start => ("Start".to_owned(), endpoints_borrowed.iter().filter(|e| e.on_start).collect()),
			RefreshTime::OnRefresh => ("Refresh".to_owned(), endpoints_borrowed.iter().filter(|e| e.on_refresh).collect()),
		};

		let existing_dropdown = if !self.endpoint_views.is_empty() {
			html! {
				<Dropdown current_label={DropdownLabel::Text("Existing Endpoint".to_owned())}>
					{ for self.endpoint_views.iter().map(|(id, view)| {
						let id_c = id.clone();
						let refresh_time_c = refresh_time.clone();
						html! {
							<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::AddTimelineEndpoint(refresh_time_c.clone(), id_c.clone(), FilterCollection::new()))}>
								{ view.name.clone() }
							</a>
					}}) }
				</Dropdown>
			}
		} else {
			html! {}
		};

		//TODO Move to separate method
		let new_endpoint = match &self.endpoint_form {
			Some(form) => if form.refresh_time == refresh_time {
				let services = self.services.clone();
				let params = form.params.clone();

				let shared_button = {
					let (label, class) = if form.shared {
						("Shared", Some("is-success"))
					} else {
						("Just this timeline", None)
					};

					let onclick = ctx.link().callback(move |_| Msg::ToggleFormShared);

					html! {
						<div class="control">
							<button class={classes!("button", class)} {onclick}>
								{label}
							</button>
						</div>
					}
				};

				html! {
					<div class="control">
						<label class="label">{ "Service" }</label>
						<Dropdown current_label={DropdownLabel::Text(form.service.to_owned())}>
							{ for services.iter().map(|service| {
								let service_name = service.0.clone();
								let service_name_2 = service.0.clone();
								html! {
									<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetFormService(service_name.clone()))}>
										{ service_name_2.clone() }
									</a>
							}}) }
						</Dropdown>
						<label class="label">{ "Type" }</label>
						<Dropdown current_label={DropdownLabel::Text(services[&form.service].constructors[form.endpoint_type].name.clone().to_string())}>
							{ for services[&form.service].constructors.iter().enumerate().map(|(i, endpoint_con)| {
								html! {
									<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetFormType(i))}>
										{ endpoint_con.name.clone() }
									</a>
							}}) }
						</Dropdown>
						{ for services[&form.service].constructors[form.endpoint_type].param_template.iter().map(move |(param, param_type)| {
							let param_c = param.clone();
							let param_type_c = param_type.clone();
							let oninput = ctx.link().batch_callback(move |e: InputEvent|
								e.target()
									.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
									.map(|i| Msg::SetFormParamValue((param_c.clone(), param_type_c.clone()), i.value()))
							);

							match param_type {
								Value::String(default) => {
									let value = params[&param.to_string()].as_str().map(|s| s.to_owned()).unwrap_or(default.clone());
									html! {
										<div class="field is-horizontal">
											<div class="field-label is-normal">
												<label class="label">{param.clone()}</label>
											</div>
											<div class="field-body">
												<div class="block control">
													<input type="text" class="input" {oninput} {value}/>
												</div>
											</div>
										</div>
									}
								}
								Value::Bool(default) => {
									let checked = params[&param.to_string()].as_bool().unwrap_or(default.clone());
									html! {
										<div class="field">
											<label class="checkbox">
												<input type="checkbox" {checked} {oninput}/>
												{ format!(" {}", &param) }
											</label>
										</div>
									}
								}
								Value::Number(default) => {
									let value = params[&param.to_string()].as_str().map(|s| s.to_owned()).unwrap_or(default.to_string());
									html! {
										<div class="field is-horizontal">
											<div class="field-label is-normal">
												<label class="label">{param.clone()}</label>
											</div>
											<div class="field-body">
												<div class="block control">
													<input type="number" class="input" {oninput} {value}/>
												</div>
											</div>
										</div>
									}
								}
								other_type => {
									log::warn!("Non implemented endpoint param type: {:?}", other_type);
									html! {
										<div class="field is-horizontal">
											<div class="field-label is-normal">
												<label class="label">{param.clone()}</label>
											</div>
											<div class="field-body">
												<div class="block control">
													<input type="text" class="input" {oninput}/>
												</div>
											</div>
										</div>
									}
								}
							}
						})}
						{ shared_button }
						{ self.view_form_filters(ctx) }
						<div class="field has-addons">
							<div class="control">
								<button class="button" onclick={ctx.link().callback(|_| Msg::CreateEndpoint(false))}>{"Create"}</button>
							</div>
							<div class="control">
								<button class="button" onclick={ctx.link().callback(|_| Msg::CreateEndpoint(true))}>{"Create for both"}</button>
							</div>
						</div>
					</div>
				}
			} else { html! {} },
			None => {
				let on_new_endpoint_callback = match refresh_time {
					RefreshTime::Start => ctx.link().callback(move |_| Msg::CreateNewEndpointForm(RefreshTime::Start)),
					RefreshTime::OnRefresh => ctx.link().callback(move |_| Msg::CreateNewEndpointForm(RefreshTime::OnRefresh)),
				};
				html! {
					<button class="button" onclick={on_new_endpoint_callback}>{"New Endpoint"}</button>
				}
			}
		};

		html! {
			<div class="field">
				<label class="label">{label}</label>
				{ for endpoints.into_iter().enumerate().map(|(i, e)| self.view_endpoint(ctx, refresh_time, e.id, i)) }
				<div class="control">
					{ existing_dropdown }
				</div>
				<div class="control">
					{ new_endpoint }
				</div>
			</div>
		}
	}

	fn view_endpoint(&self, ctx: &Context<Self>, refresh_time: RefreshTime, endpoint_id: EndpointId, index: usize) -> Html {
		//TODO endpoint_views shouldn't be empty
		if let Some(endpoint_view) = self.endpoint_views.get(&endpoint_id) {
			let name = endpoint_view.name.clone();

			html! {
				<div class="block">
					{ name }
					{ self.view_existing_filters(ctx, index) }
					<div class="control">
						<button
							class="button"
							onclick={ctx.link().callback(move |_| Msg::RemoveTimelineEndpoint(refresh_time.clone(), endpoint_id.clone()))}
						>
							{"Remove"}
						</button>
					</div>
				</div>
			}
		} else {
			html! {}
		}
	}

	fn view_existing_filters(&self, ctx: &Context<Self>, endpoint_index: usize) -> Html {
		html! {
			<FiltersOptions
				filters={ctx.props().timeline_endpoints.upgrade().unwrap().borrow()[endpoint_index].filters.clone()}
				callback={ctx.link().callback(move |msg| Msg::ExistingFilterMsg(endpoint_index, msg))}
			/>
		}
	}

	fn view_form_filters(&self, ctx: &Context<Self>) -> Html {
		html! {
			<FiltersOptions
				filters={self.endpoint_form.as_ref().unwrap().filters.clone()}
				callback={ctx.link().callback(Msg::FormFilterMsg)}
			/>
		}
	}
}
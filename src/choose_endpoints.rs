use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use std::rc::Weak;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use serde_json::Value;

use crate::services::endpoint_agent::{Request as EndpointRequest, EndpointAgent, EndpointId, RefreshTime, EndpointConstructors, Response as EndpointResponse, EndpointView, TimelineEndpointWrapper};
use crate::components::{Dropdown, DropdownLabel};
use crate::timeline::{
	filters::{Filter, FilterInstance, FiltersOptions},
	agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse},
};

struct EndpointForm {
	refresh_time: RefreshTime,
	service: &'static str,
	endpoint_type: usize,
	params: Value,
	filters: HashSet<FilterInstance>,
}

pub struct ChooseEndpoints {
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	_add_timeline_agent: Option<Box<dyn Bridge<TimelineAgent>>>,
	show_start_endpoint_dropdown: bool,
	show_refresh_endpoint_dropdown: bool,
	endpoint_views: HashMap<EndpointId, EndpointView>,
	endpoint_form: Option<EndpointForm>,
	services: HashMap<&'static str, EndpointConstructors>,
}

pub enum Msg {
	EndpointResponse(EndpointResponse),
	TimelineAgentResponse(TimelineAgentResponse),
	ToggleStartEndpointDropdown,
	ToggleRefreshEndpointDropdown,
	CreateNewEndpointForm(RefreshTime),
	SetFormService(&'static str),
	SetFormType(usize),
	SetFormParamValue((&'static str, Value), String),
	CreateEndpoint(bool),
	AddTimelineEndpoint(RefreshTime, EndpointId, HashSet<FilterInstance>),
	AddTimelineEndpointBoth(EndpointId, HashSet<FilterInstance>),
	RemoveTimelineEndpoint(RefreshTime, EndpointId),
	ToggleFormFilterEnabled(FilterInstance),
	ToggleFormFilterInverted(FilterInstance),
	RemoveFormFilter(FilterInstance),
	AddFormFilter(Filter, bool),
	ToggleExistingFilterEnabled(usize, FilterInstance),
	ToggleExistingFilterInverted(usize, FilterInstance),
	RemoveExistingFilter(usize, FilterInstance),
	AddExistingFilter(usize, Filter, bool),
}

#[derive(Properties, Clone)]
pub struct Props {
	pub timeline_endpoints: Weak<RefCell<Vec<TimelineEndpointWrapper>>>,
	#[prop_or_default]
	pub inside_add_timeline: bool,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.timeline_endpoints.ptr_eq(&other.timeline_endpoints)
	}
}

impl Component for ChooseEndpoints {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let _add_timeline_agent = match ctx.props().inside_add_timeline {
			true => {
				let mut agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
				agent.send(TimelineAgentRequest::RegisterChooseEndpoints);
				Some(agent)
			},
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
			Msg::TimelineAgentResponse(response) => match response {
				TimelineAgentResponse::AddBlankTimeline => {
					self.endpoint_form = None;
					true
				}
				TimelineAgentResponse::AddUserTimeline(service, username) => {
					//TODO Default user timeline filters in options
					if let Some(endpoint_type) = self.services[&service].user_endpoint {
						self.endpoint_form = Some(EndpointForm {
							refresh_time: RefreshTime::Start,
							service,
							endpoint_type,
							params: json!({
								"username": username,
								"include_retweets": true,
								"include_replies": true,
							}),
							filters: HashSet::new(),
						});
						true
					}else { false }
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
					params: self.services["Twitter"].endpoint_types[0].default_params(),
					filters: HashSet::new(),
				});
				true
			}
			Msg::SetFormService(name) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.service != name {
						form.params = self.services[&name].endpoint_types[0].default_params();
						form.service = name;
						form.endpoint_type = 0;
						true
					}else {
						false
					}
				}else {
					false
				}
			}
			Msg::SetFormType(endpoint_type) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.endpoint_type != endpoint_type {
						form.endpoint_type = endpoint_type;
						form.params = self.services[&form.service].endpoint_types[endpoint_type].default_params();
						true
					}else {
						false
					}
				}else {
					false
				}
			}
			Msg::SetFormParamValue((param, param_type), value) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.params[&param] != value {
						if let Value::Bool(_) = param_type {
							if let Value::Bool(prev) = form.params[&param] {
								form.params[&param] = Value::Bool(!prev);
							}else {
								form.params[&param] = Value::Bool(value != "on" && value != "true");
							}
						}else {
							form.params[&param] = Value::String(value);
						}
						true
					}else {
						false
					}
				}else {
					false
				}
			}
			Msg::CreateEndpoint(both) => {
				let mut created = false;
				if let Some(EndpointForm { service, endpoint_type, refresh_time, params, filters }) = std::mem::take(&mut self.endpoint_form) {
					let constructor = self.services[&service].endpoint_types[endpoint_type].clone();
					let refresh_time_c = refresh_time;
					let callback = if both {
						ctx.link().callback(move |(id, filters)| Msg::AddTimelineEndpointBoth(id, filters))
					}else {
						ctx.link().callback(move |(id, filters)| Msg::AddTimelineEndpoint(refresh_time_c, id, filters))
					};
					let params = params.clone();
					self.endpoint_agent.send(EndpointRequest::AddEndpoint(Box::new(move |id| {
						callback.emit((id, filters));
						(constructor.callback)(id, params.clone())
					})));

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
				let mut borrow = timeline_endpoints.borrow_mut();
				borrow.push(TimelineEndpointWrapper {
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
							}else {
								borrow.remove(index);
							}
						},
						RefreshTime::OnRefresh => {
							if borrow[index].on_start {
								borrow[index].on_refresh = false;
							}else {
								borrow.remove(index);
							}
						},
					};
				}
				true
			}
			Msg::ToggleFormFilterEnabled(filter_instance) => {
				let filters = &mut self.endpoint_form.as_mut().unwrap().filters;
				filters.remove(&filter_instance);
				filters.insert(FilterInstance {
					enabled: !filter_instance.enabled,
					..filter_instance
				});
				true
			}
			Msg::ToggleFormFilterInverted(filter_instance) => {
				let filters = &mut self.endpoint_form.as_mut().unwrap().filters;
				filters.remove(&filter_instance);
				filters.insert(FilterInstance {
				   inverted: !filter_instance.inverted,
					..filter_instance
				});
				true
			}
			Msg::RemoveFormFilter(filter_instance) => {
				self.endpoint_form.as_mut().unwrap().filters.remove(&filter_instance);
				true
			}
			Msg::AddFormFilter(filter, inverted) => {
				self.endpoint_form.as_mut().unwrap().filters.insert(FilterInstance {
					filter,
					inverted,
					enabled: true
				});
				true
			}
			Msg::ToggleExistingFilterEnabled(endpoint_index, filter_instance) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				let mut borrow = timeline_endpoints.borrow_mut();
				let filters = &mut borrow[endpoint_index].filters;
				filters.remove(&filter_instance);
				filters.insert(FilterInstance {
					enabled: !filter_instance.enabled,
					..filter_instance
				});
				true
			}
			Msg::ToggleExistingFilterInverted(endpoint_index, filter_instance) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				let mut borrow = timeline_endpoints.borrow_mut();
				let filters = &mut borrow[endpoint_index].filters;
				filters.remove(&filter_instance);
				filters.insert(FilterInstance {
					inverted: !filter_instance.inverted,
					..filter_instance
				});
				true
			}
			Msg::RemoveExistingFilter(endpoint_index, filter_instance) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				let mut borrow = timeline_endpoints.borrow_mut();
				borrow[endpoint_index].filters.remove(&filter_instance);
				true
			}
			Msg::AddExistingFilter(endpoint_index, filter, inverted) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				let mut borrow = timeline_endpoints.borrow_mut();
				borrow[endpoint_index].filters.insert(FilterInstance {
					filter,
					inverted,
					enabled: true
				});
				true
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
							<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::AddTimelineEndpoint(refresh_time_c.clone(), id_c.clone(), HashSet::new()))}>
								{ view.name.clone() }
							</a>
					}}) }
				</Dropdown>
			}
		}else {
			html! {}
		};

		let new_endpoint = match &self.endpoint_form {
			Some(form) => if form.refresh_time == refresh_time {
				let services = self.services.clone();
				let params = form.params.clone();
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
						<Dropdown current_label={DropdownLabel::Text(services[&form.service].endpoint_types[form.endpoint_type].name.clone().to_string())}>
							{ for services[&form.service].endpoint_types.iter().enumerate().map(|(i, endpoint_con)| {
								html! {
									<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetFormType(i))}>
										{ endpoint_con.name.clone() }
									</a>
							}}) }
						</Dropdown>
						{ for services[&form.service].endpoint_types[form.endpoint_type].param_template.iter().map(move |(param, param_type)| {
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
			}else { html! {} },
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
		html! {
			<div class="block">
				{ self.endpoint_views.get(&endpoint_id).map(|e| e.name.clone()).unwrap_or_default() }
				{ self.view_existing_filters(ctx, index) }
				<button
					class="button"
					onclick={ctx.link().callback(move |_| Msg::RemoveTimelineEndpoint(refresh_time.clone(), endpoint_id.clone()))}
				>{"Remove"}</button>
			</div>
		}
	}

	fn view_existing_filters(&self, ctx: &Context<Self>, endpoint_index: usize) -> Html {
		html! {
			<FiltersOptions
				filters={ctx.props().timeline_endpoints.upgrade().unwrap().borrow()[endpoint_index].filters.clone()}
				toggle_enabled_callback={ctx.link().callback(move |filter_instance| Msg::ToggleExistingFilterEnabled(endpoint_index, filter_instance))}
				toggle_inverted_callback={ctx.link().callback(move |filter_instance| Msg::ToggleExistingFilterInverted(endpoint_index, filter_instance))}
				remove_callback={ctx.link().callback(move |filter_instance| Msg::RemoveExistingFilter(endpoint_index, filter_instance))}
				add_callback={ctx.link().callback(move |(filter, inverted)| Msg::AddExistingFilter(endpoint_index, filter, inverted))}
			/>
		}
	}

	fn view_form_filters(&self, ctx: &Context<Self>) -> Html {
		html! {
			<FiltersOptions
				filters={self.endpoint_form.as_ref().unwrap().filters.clone()}
				toggle_enabled_callback={ctx.link().callback(Msg::ToggleFormFilterEnabled)}
				toggle_inverted_callback={ctx.link().callback(Msg::ToggleFormFilterInverted)}
				remove_callback={ctx.link().callback(Msg::RemoveFormFilter)}
				add_callback={ctx.link().callback(|(filter, inverted)| Msg::AddFormFilter(filter, inverted))}
			/>
		}
	}
}
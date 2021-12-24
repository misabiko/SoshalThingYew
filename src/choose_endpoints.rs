use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use std::rc::Weak;
use std::collections::HashMap;
use std::cell::RefCell;
use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::services::endpoint_agent::{Request as EndpointRequest, EndpointAgent, TimelineEndpoints, EndpointId, RefreshTime, EndpointConstructors, Response as EndpointResponse, EndpointView};
use crate::dropdown::{Dropdown, DropdownLabel};
use crate::timeline::agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse};

struct EndpointForm {
	refresh_time: RefreshTime,
	service: String,
	endpoint_type: usize,
	params: serde_json::Value,
}

pub struct ChooseEndpoints {
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	_add_timeline_agent: Option<Box<dyn Bridge<TimelineAgent>>>,
	show_start_endpoint_dropdown: bool,
	show_refresh_endpoint_dropdown: bool,
	endpoint_views: HashMap<EndpointId, EndpointView>,
	endpoint_form: Option<EndpointForm>,
	services: HashMap<String, EndpointConstructors>,
}

pub enum Msg {
	EndpointResponse(EndpointResponse),
	TimelineAgentResponse(TimelineAgentResponse),
	ToggleStartEndpointDropdown,
	ToggleRefreshEndpointDropdown,
	NewEndpoint(RefreshTime),
	SetFormService(String),
	SetFormType(usize),
	SetFormParamValue(String, String),
	CreateEndpoint,
	AddTimelineEndpoint(RefreshTime, EndpointId),
	RemoveTimelineEndpoint(RefreshTime, EndpointId),
}

#[derive(Properties, Clone)]
pub struct Props {
	pub timeline_endpoints: Weak<RefCell<TimelineEndpoints>>,
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
					if let Some(endpoint_type) = self.services[&service].user_endpoint.clone() {
						self.endpoint_form = Some(EndpointForm {
							refresh_time: RefreshTime::Start,
							service,
							endpoint_type,
							params: json!({
								"username": username,
							})
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
			Msg::NewEndpoint(refresh_time) => {
				self.endpoint_form = Some(EndpointForm {
					refresh_time,
					service: "Twitter".to_owned(),
					endpoint_type: 0,
					params: serde_json::Value::default()
				});
				true
			}
			Msg::SetFormService(name) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.service != name {
						form.service = name;
						form.endpoint_type = 0;
						form.params = serde_json::Value::default();
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
						form.params = serde_json::Value::default();
						true
					}else {
						false
					}
				}else {
					false
				}
			}
			Msg::SetFormParamValue(param, value) => {
				if let Some(form) = &mut self.endpoint_form {
					if form.params[&param] != value {
						form.params[&param] = json!(value);
						true
					}else {
						false
					}
				}else {
					false
				}
			}
			Msg::CreateEndpoint => {
				let mut created = false;
				if let Some(form) = &mut self.endpoint_form {
					let constructor = self.services[&form.service].endpoint_types[form.endpoint_type.clone()].clone();
					let refresh_time_c = form.refresh_time.clone();
					let callback = ctx.link().callback(move |id| Msg::AddTimelineEndpoint(refresh_time_c.clone(), id));
					let params = form.params.clone();
					self.endpoint_agent.send(EndpointRequest::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						(constructor.callback)(id, params.clone())
					})));

					created = true;
				}

				self.endpoint_form = None;
				created
			}
			Msg::AddTimelineEndpoint(refresh_time, id) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				match refresh_time {
					RefreshTime::Start => timeline_endpoints.borrow_mut().start.insert(id.clone()),
					RefreshTime::OnRefresh => timeline_endpoints.borrow_mut().refresh.insert(id.clone()),
				};
				true
			}
			Msg::RemoveTimelineEndpoint(refresh_time, id) => {
				let timeline_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
				match refresh_time {
					RefreshTime::Start => timeline_endpoints.borrow_mut().start.remove(&id),
					RefreshTime::OnRefresh => timeline_endpoints.borrow_mut().refresh.remove(&id),
				};
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<>
				{ self.view_refresh_time_endpoints(ctx, RefreshTime::Start) }
				{ self.view_refresh_time_endpoints(ctx, RefreshTime::OnRefresh) }
			</>
		}
	}
}

impl ChooseEndpoints {
	fn view_refresh_time_endpoints(&self, ctx: &Context<Self>, refresh_time: RefreshTime) -> Html {
		let strong_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();
		let endpoints_borrowed = strong_endpoints.borrow();
		let (label, endpoints_iter) = match refresh_time {
			RefreshTime::Start => ("Start".to_owned(), endpoints_borrowed.start.iter()),
			RefreshTime::OnRefresh => ("Refresh".to_owned(), endpoints_borrowed.refresh.iter()),
		};

		let new_endpoint = match &self.endpoint_form {
			Some(form) => if form.refresh_time == refresh_time {
				let services = self.services.clone();
				let params = form.params.clone();
				html! {
					<div class="control">
						<label class="label">{ "Service" }</label>
						<Dropdown current_label={DropdownLabel::Text(form.service.clone())}>
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
						<Dropdown current_label={DropdownLabel::Text(services[&form.service].endpoint_types[form.endpoint_type.clone()].name.clone().to_string())}>
							{ for services[&form.service].endpoint_types.iter().enumerate().map(|(i, endpoint_con)| {
								html! {
									<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetFormType(i))}>
										{ endpoint_con.name.clone() }
									</a>
							}}) }
						</Dropdown>
						{ for services[&form.service].endpoint_types[form.endpoint_type.clone()].param_template.iter().map(move |param| {
							let param_c = param.to_string();
							let value: String = params[&param_c].as_str().unwrap().to_owned();
							let oninput = ctx.link().batch_callback(move |e: InputEvent|
								e.target()
									.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
									.map(|i| Msg::SetFormParamValue(param_c.clone(), i.value()))
							);

							html! {
								<div class="field">
									<label class="label">{param.clone()}</label>
									<div class="control">
										<input type="text" class="input" {oninput} {value}/>
									</div>
								</div>
							}
						})}
						<button class="button" onclick={ctx.link().callback(|_| Msg::CreateEndpoint)}>{"Create"}</button>
					</div>
				}
			}else { html! {} },
			None => {
				let on_new_endpoint_callback = match refresh_time {
					RefreshTime::Start => ctx.link().callback(move |_| Msg::NewEndpoint(RefreshTime::Start)),
					RefreshTime::OnRefresh => ctx.link().callback(move |_| Msg::NewEndpoint(RefreshTime::OnRefresh)),
				};
				html! {
					<button class="button" onclick={on_new_endpoint_callback}>{"New Endpoint"}</button>
				}
			}
		};

		let existing_dropdown = if !self.endpoint_views.is_empty() {
			html! {
				<Dropdown current_label={DropdownLabel::Text("Existing Endpoint".to_owned())}>
					{ for self.endpoint_views.iter().map(|(id, view)| {
						let id_c = id.clone();
						let refresh_time_c = refresh_time.clone();
						html! {
							<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::AddTimelineEndpoint(refresh_time_c.clone(), id_c.clone()))}>
								{ view.name.clone() }
							</a>
					}}) }
				</Dropdown>
			}
		}else {
			html! {}
		};

		html! {
			<div class="field">
				<label class="label">{label}</label>
				{ for endpoints_iter.map(|id| self.view_endpoint(ctx, &refresh_time, id)) }
				<div class="control">
					{ existing_dropdown }
				</div>
				{ new_endpoint.clone() }
			</div>
		}
	}

	fn view_endpoint(&self, ctx: &Context<Self>, refresh_time: &RefreshTime, endpoint_id: &EndpointId) -> Html {
		let refresh_time_c = refresh_time.clone();
		let endpoint_id_c = endpoint_id.clone();
		html! {
			<>
				{ self.endpoint_views.get(endpoint_id).map(|e| e.name.clone()).unwrap_or_default() }
				<button
					class="button"
					onclick={ctx.link().callback(move |_| Msg::RemoveTimelineEndpoint(refresh_time_c.clone(), endpoint_id_c.clone()))}
				>{"Remove"}</button>
			</>
		}
	}
}
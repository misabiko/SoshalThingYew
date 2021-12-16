use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher, Bridge};
use yew_agent::utils::store::{StoreWrapper, ReadOnly, Bridgeable};
use std::rc::Weak;
use std::collections::HashMap;

use crate::timeline::{Props as TimelineProps};
use crate::services::endpoints::{EndpointStore, TimelineEndpoints, EndpointId, RefreshTime};

struct EndpointView {
	name: String
}

struct EndpointForm {
	refresh_time: RefreshTime,
	service: String,
	endpoint_type: String,
}

pub struct ChooseEndpointModal {
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	show_start_endpoint_dropdown: bool,
	show_refresh_endpoint_dropdown: bool,
	endpoint_views: HashMap<EndpointId, EndpointView>,
	endpoint_form: Option<EndpointForm>,
}

pub enum Msg {
	ChooseEndpoint,
	EndpointStoreResponse(ReadOnly<EndpointStore>),
	ToggleStartEndpointDropdown,
	ToggleRefreshEndpointDropdown,
	NewEndpoint(RefreshTime),
}

#[derive(Properties, Clone)]
pub struct Props {
	pub timeline_endpoints: Weak<TimelineEndpoints>,
	pub close_modal_callback: Callback<MouseEvent>,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.timeline_endpoints.ptr_eq(&other.timeline_endpoints) &&
			self.close_modal_callback == other.close_modal_callback
	}
}

impl Component for ChooseEndpointModal {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			endpoint_store: EndpointStore::bridge(ctx.link().callback(Msg::EndpointStoreResponse)),
			show_start_endpoint_dropdown: false,
			show_refresh_endpoint_dropdown: false,
			endpoint_views: HashMap::new(),
			endpoint_form: None,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::EndpointStoreResponse(state) => {
				let state = state.borrow();
				log::info!("Endpoints: {}", state.endpoints.len());
				self.endpoint_views.clear();
				for (endpoint_id, endpoint) in &state.endpoints {
					self.endpoint_views.insert(endpoint_id.clone(), EndpointView {
						name: endpoint.name()
					});
				}
				false
			}
			Msg::ChooseEndpoint => {
				log::info!("Choosing endpoint");
				false
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
					endpoint_type: "Home".to_owned(),
				});
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="modal is-active">
				<div class="modal-background"/>
				<div class="modal-content">
					<div class="card">
						<header class="card-header">
							<p class="card-header-title">{"Choose Endpoint"}</p>
							<button class="card-header-icon">
								<span class="icon">
									<i class="fas fa-times"/>
								</span>
							</button>
						</header>
						<div class="card-content">
							{ self.view_refresh_time_endpoints(ctx, RefreshTime::Start) }
							{ self.view_refresh_time_endpoints(ctx, RefreshTime::OnRefresh) }
						</div>
					</div>
				</div>
			</div>
		}
	}
}

impl ChooseEndpointModal {
	fn view_refresh_time_endpoints(&self, ctx: &Context<Self>, refresh_time: RefreshTime) -> Html {
		let strong_endpoints = ctx.props().timeline_endpoints.upgrade().unwrap();

		let label = match refresh_time {
			RefreshTime::Start => "Start".to_owned(),
			RefreshTime::OnRefresh => "Refresh".to_owned(),
		};

		let new_endpoint = match &self.endpoint_form {
			Some(form) => if form.refresh_time == refresh_time {
				html! {
					<div class="control">
						<label class="label">{ "Service" }</label>
						{ form.service.clone() }
						<label class="label">{ "Type" }</label>
						{ form.endpoint_type.clone() }
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

		html! {
			<div class="field">
				<label class="label">{label}</label>
				{ for strong_endpoints.start.iter().map(|id| self.view_endpoint(id)) }
				<div class="control">
					<div class={classes!("dropdown", if self.show_start_endpoint_dropdown { Some("is-active") } else { None })}>
						<div class="dropdown-trigger">
							<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleStartEndpointDropdown)}>
								<span>{"Existing Endpoint"}</span>
								<span class="icon is-small">
									<i class="fas fa-angle-down"/>
								</span>
							</button>
						</div>
						<div class="dropdown-menu">
							<div class="dropdown-content">
							</div>
						</div>
					</div>
				</div>
				{ new_endpoint }
			</div>
		}
	}

	fn view_endpoint(&self, endpoint_id: &EndpointId) -> Html {
		html! {
			self.endpoint_views.get(endpoint_id).map(|e| e.name.clone()).unwrap_or_default()
		}
	}
}
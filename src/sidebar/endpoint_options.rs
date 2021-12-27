use std::collections::HashMap;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use wasm_bindgen::JsCast;

use crate::services::endpoint_agent::{EndpointId, EndpointAgent, Request as EndpointRequest, Response as EndpointResponse, EndpointView};

pub struct EndpointOptions {
	pub endpoints: HashMap<EndpointId, EndpointView>,
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
}

pub enum Msg {
	EndpointResponse(EndpointResponse),
	StartAutoRefresh(EndpointId),
	StopAutoRefresh(EndpointId),
	SetAutoRefreshInterval(EndpointId, u32)
}

impl Component for EndpointOptions {
	type Message = Msg;
	type Properties = ();

	fn create(ctx: &Context<Self>) -> Self {
		let mut _endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		_endpoint_agent.send(EndpointRequest::GetState);

		Self {
			endpoints: HashMap::new(),
			endpoint_agent: _endpoint_agent,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::EndpointResponse(response) => match response {
				EndpointResponse::UpdatedState(_services, endpoints) => {
					self.endpoints.clear();
					for endpoint in endpoints {
						self.endpoints.insert(endpoint.id.clone(), endpoint.clone());
					}

					true
				}
				_ => false
			}
			Msg::StartAutoRefresh(endpoint_id) => {
				self.endpoint_agent.send(EndpointRequest::StartAutoRefresh(endpoint_id));
				false
			}
			Msg::StopAutoRefresh(endpoint_id) => {
				self.endpoint_agent.send(EndpointRequest::StopAutoRefresh(endpoint_id));
				false
			}
			Msg::SetAutoRefreshInterval(endpoint_id, interval) => {
				self.endpoint_agent.send(EndpointRequest::SetAutoRefreshInterval(endpoint_id, interval));
				false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			{ for self.endpoints.values().map(|endpoint| {
				html! {
					<div class="block">
						{ endpoint.name.clone() }
						{ self.view_ratelimit(&endpoint) }
						{ self.view_autorefresh(ctx, &endpoint) }
					</div>
				}
			}) }
		}
	}
}

impl EndpointOptions {
	fn view_ratelimit(&self, endpoint: &EndpointView) -> Html {
		match &endpoint.ratelimit {
			Some(ratelimit) => {
				let time_left = (((ratelimit.reset as f64 * 1000.0) - js_sys::Date::now()) / 60000.0).ceil();
				html! {
					<>
						<progress class="progress" value={ratelimit.remaining.to_string()} max={ratelimit.limit.to_string()}>
							{ format!("{}%", (ratelimit.remaining as f64 / ratelimit.limit as f64 * 1000.0).round() / 10.0) }
						</progress>
						{ format!("{} minutes until reset", &time_left)}
					</>
				}
			},
			None => html! {}
		}
	}

	fn view_autorefresh(&self, ctx: &Context<Self>, endpoint: &EndpointView) -> Html {
		let id_c = endpoint.id.clone();
		let interval = endpoint.autorefresh_interval.to_string();

		html! {
			<div class="field has-addons">
				{ match endpoint.is_autorefreshing {
					false => {
						let id_c_2 = endpoint.id.clone();
						let oninput = ctx.link().batch_callback(move |e: InputEvent|
							e.target()
								.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
								.and_then(|i| i.value().parse::<u32>().ok())
								.map(|v| Msg::SetAutoRefreshInterval(id_c_2.clone(), v))
						);
						html! {
							<>
								<div class="control">
									<button class="button" onclick={ctx.link().callback(move |_| Msg::StartAutoRefresh(id_c))}>
										{"Auto refresh"}
									</button>
								</div>
								<div class="control">
									<input class="input" type="number" value={interval} {oninput}/>
								</div>
								<div class="control">
									<a class="button is-static">{"ms"}</a>
								</div>
							</>
						}
					},
					true => html! {
							<>
								<div class="control">
									<button class="button" onclick={ctx.link().callback(move |_| Msg::StopAutoRefresh(id_c))}>
										{"Stop refreshing"}
									</button>
								</div>
								<div class="control">
									<input class="input" type="number" value={interval} disabled=true/>
								</div>
								<div class="control">
									<a class="button is-static">{"ms"}</a>
								</div>
							</>
						},
				} }
			</div>
		}
	}
}
use std::collections::HashMap;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::RateLimit;
use crate::services::endpoint_agent::{EndpointAgent, Request as EndpointRequest, Response as EndpointResponse};

pub struct RateLimitView {
	pub ratelimits: HashMap<String, RateLimit>,
	_endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
}

pub enum Msg {
	EndpointResponse(EndpointResponse),
}

impl Component for RateLimitView {
	type Message = Msg;
	type Properties = ();

	fn create(ctx: &Context<Self>) -> Self {
		let mut _endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		_endpoint_agent.send(EndpointRequest::GetState);

		Self {
			ratelimits: HashMap::new(),
			_endpoint_agent,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::EndpointResponse(response) => match response {
				EndpointResponse::UpdatedState(_services, endpoints) => {
					self.ratelimits.clear();
					for endpoint in endpoints {
						if let Some(ratelimit) = endpoint.ratelimit {
							self.ratelimits.insert(endpoint.name, ratelimit.clone());
						}
					}

					true
				}
				_ => false
			}
		}
	}

	fn view(&self, _ctx: &Context<Self>) -> Html {
		html! {
			{ for self.ratelimits.iter().map(|(endpoint, ratelimit)| {
				let time_left = (((ratelimit.reset as f64 * 1000.0) - js_sys::Date::now()) / 60000.0).ceil();
				html! {
				<div>
					{ endpoint.clone() }
					<progress class="progress" value={ratelimit.remaining.to_string()} max={ratelimit.limit.to_string()}>
						{ format!("{}%", (ratelimit.remaining as f64 / ratelimit.limit as f64 * 1000.0).round() / 10.0) }
					</progress>
					{ format!("{} minutes until reset", &time_left)}
				</div>
				}
			}) }
		}
	}
}
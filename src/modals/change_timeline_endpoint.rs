use yew::prelude::*;
use std::rc::Weak;
use std::cell::RefCell;

use crate::services::endpoints::TimelineEndpoints;
use crate::choose_endpoints::ChooseEndpoints;

pub struct ChooseTimelineEndpointModal;

pub enum Msg {}

#[derive(Properties, Clone)]
pub struct Props {
	pub timeline_endpoints: Weak<RefCell<TimelineEndpoints>>,
	pub close_modal_callback: Callback<MouseEvent>,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.timeline_endpoints.ptr_eq(&other.timeline_endpoints) &&
			self.close_modal_callback == other.close_modal_callback
	}
}

impl Component for ChooseTimelineEndpointModal {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {}
	}

	fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
		false
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="modal is-active">
				<div class="modal-background"/>
				<div class="modal-content">
					<div class="card">
						<header class="card-header">
							<p class="card-header-title">{"Choose Endpoint"}</p>
							<button class="card-header-icon" onclick={ctx.props().close_modal_callback.clone()}>
								<span class="icon">
									<i class="fas fa-times"/>
								</span>
							</button>
						</header>
						<div class="card-content">
							<ChooseEndpoints timeline_endpoints={ctx.props().timeline_endpoints.clone()}/>
						</div>
					</div>
				</div>
			</div>
		}
	}
}
use yew::prelude::*;
use web_sys::HtmlInputElement;
use std::rc::Rc;
use std::cell::RefCell;
use yew_agent::{Bridge, Bridged};

use super::Modal;
use crate::timeline::{Props as TimelineProps, TimelineId};
use crate::timeline::agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse};
use crate::services::endpoints::TimelineEndpoints;
use crate::choose_endpoints::ChooseEndpoints;

pub struct AddTimelineModal {
	enabled: bool,
	title_ref: NodeRef,
	endpoints: Rc<RefCell<TimelineEndpoints>>,
	_agent: Box<dyn Bridge<TimelineAgent>>,
}

pub enum Msg {
	AddTimeline,
	AgentResponse(TimelineAgentResponse),
	SetEnabled(bool),
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub add_timeline_callback: Callback<Box<dyn FnOnce(TimelineId) -> TimelineProps>>,
}

impl Component for AddTimelineModal {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut _agent = TimelineAgent::bridge(ctx.link().callback(Msg::AgentResponse));
		_agent.send(TimelineAgentRequest::RegisterModal);

		Self {
			enabled: false,
			title_ref: NodeRef::default(),
			endpoints: Rc::new(RefCell::new(TimelineEndpoints::default())),
			_agent,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddTimeline => {
				let name = match self.title_ref.cast::<HtmlInputElement>() {
					Some(input) => input.value(),
					None => "Timeline".to_owned(),
				};
				let endpoints = self.endpoints.borrow().clone();
				ctx.props().add_timeline_callback.emit(Box::new(|id| {
					yew::props! { TimelineProps {
						name,
						id,
						endpoints,
					}}
				}));

				self.endpoints.replace(TimelineEndpoints::default());
				self.enabled = false;
				true
			}
			Msg::AgentResponse(response) => match response {
				TimelineAgentResponse::AddTimeline => {
					ctx.link().send_message(Msg::SetEnabled(true));
					false
				}
				TimelineAgentResponse::AddBlankTimeline => {
					if let Some(title) = self.title_ref.cast::<HtmlInputElement>() {
						title.set_value("Timeline");
					}
					let mut borrow = self.endpoints.borrow_mut();
					borrow.start.clear();
					borrow.refresh.clear();

					self.enabled = true;
					true
				},
				TimelineAgentResponse::AddUserTimeline(_service, username) => {
					if let Some(title) = self.title_ref.cast::<HtmlInputElement>() {
						title.set_value(&username);
					}
					let mut borrow = self.endpoints.borrow_mut();
					borrow.start.clear();
					borrow.refresh.clear();

					self.enabled = true;
					true
				},
				_ => false,
			}
			Msg::SetEnabled(value) => {
				self.enabled = value;
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let footer = html! {
			<>
				<button
					class="button card-footer-item"
					onclick={ctx.link().callback(|_| Msg::AddTimeline)}
				>{"Add"}</button>
				<button class="button card-footer-item" onclick={ctx.link().callback(|_| Msg::SetEnabled(false))}>{"Cancel"}</button>
			</>
		};

		html! {
			<Modal enabled={self.enabled.clone()} modal_title="Add Timeline" close_modal_callback={ctx.link().callback(|_| Msg::SetEnabled(false))} {footer}>
				<div class="field">
					<label class="label">{"Title"}</label>
					<div class="control">
						<input type="text" class="input" ref={self.title_ref.clone()}/>
					</div>
				</div>
				<ChooseEndpoints inside_add_timeline=true timeline_endpoints={Rc::downgrade(&self.endpoints)}/>
			</Modal>
		}
	}
}
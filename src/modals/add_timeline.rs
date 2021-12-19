use yew::prelude::*;
use web_sys::HtmlInputElement;
use std::rc::Rc;
use std::cell::RefCell;
use yew_agent::{Agent, AgentLink, Bridge, Bridged, HandlerId, Context as AgentContext};

use super::Modal;
use crate::timeline::{Props as TimelineProps};
use crate::services::endpoints::TimelineEndpoints;
use crate::choose_endpoints::ChooseEndpoints;

pub struct AddTimelineModal {
	enabled: bool,
	title_ref: NodeRef,
	endpoints: Rc<RefCell<TimelineEndpoints>>,
	_agent: Box<dyn Bridge<AddTimelineAgent>>,
}

pub enum Msg {
	AddTimeline,
	AgentResponse(Response),
	SetEnabled(bool),
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub add_timeline_callback: Callback<Box<dyn FnOnce(i16) -> TimelineProps>>,
}

impl Component for AddTimelineModal {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut _agent = AddTimelineAgent::bridge(ctx.link().callback(Msg::AgentResponse));
		_agent.send(Request::RegisterModal);

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
				Response::AddTimeline => {
					ctx.link().send_message(Msg::SetEnabled(true));
					false
				}
				Response::AddBlankTimeline => {
					if let Some(title) = self.title_ref.cast::<HtmlInputElement>() {
						title.set_value("Timeline");
					}
					let mut borrow = self.endpoints.borrow_mut();
					borrow.start.clear();
					borrow.refresh.clear();

					self.enabled = true;
					true
				},
				Response::AddUserTimeline(_service, username) => {
					if let Some(title) = self.title_ref.cast::<HtmlInputElement>() {
						title.set_value(&username);
					}
					let mut borrow = self.endpoints.borrow_mut();
					borrow.start.clear();
					borrow.refresh.clear();

					self.enabled = true;
					true
				},
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

pub struct AddTimelineAgent {
	link: AgentLink<Self>,
	modal: Option<HandlerId>,
	choose_endpoints: Option<HandlerId>,
}

pub enum Request {
	RegisterModal,
	RegisterChooseEndpoints,
	AddTimeline,
	AddUserTimeline(String, String),
}

pub enum Response {
	AddTimeline,
	AddBlankTimeline,
	AddUserTimeline(String, String),
}

impl Agent for AddTimelineAgent {
	type Reach = AgentContext<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			modal: None,
			choose_endpoints: None,
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::RegisterModal => self.modal = Some(id),
			Request::RegisterChooseEndpoints => self.choose_endpoints = Some(id),
			Request::AddTimeline => {
				if let Some(choose_endpoints) = self.choose_endpoints {
					self.link.respond(choose_endpoints, Response::AddBlankTimeline)
				}
				if let Some(modal) = self.modal {
					self.link.respond(modal, Response::AddBlankTimeline)
				}
			}
			Request::AddUserTimeline(service, username) => {
				if let Some(choose_endpoints) = self.choose_endpoints {
					self.link.respond(choose_endpoints, Response::AddUserTimeline(service.clone(), username.clone()))
				}
				if let Some(modal) = self.modal {
					self.link.respond(modal, Response::AddUserTimeline(service, username))
				}
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		if self.modal == Some(id) {
			self.modal = None
		}else if self.choose_endpoints == Some(id) {
			self.choose_endpoints = None
		}
	}
}
use yew::prelude::*;
use web_sys::HtmlInputElement;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;
use yew_agent::{Bridge, Bridged};

use super::ModalCard;
use crate::timeline::{
	Props as TimelineProps,
	agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse},
	filters::{Filter, FilterInstance, FiltersOptions, DEFAULT_FILTERS},
};
use crate::choose_endpoints::ChooseEndpoints;
use crate::{TimelineEndpointWrapper, TimelinePropsClosure};

pub struct AddTimelineModal {
	enabled: bool,
	title_ref: NodeRef,
	endpoints: Rc<RefCell<Vec<TimelineEndpointWrapper>>>,
	_agent: Box<dyn Bridge<TimelineAgent>>,
	filters: HashSet<FilterInstance>,
	set_as_main_timeline: bool,
}

pub enum Msg {
	AddTimeline,
	AgentResponse(TimelineAgentResponse),
	SetEnabled(bool),
	ToggleFilterEnabled(FilterInstance),
	ToggleFilterInverted(FilterInstance),
	AddFilter((Filter, bool)),
	RemoveFilter(FilterInstance),
	ToggleSetMainTimeline,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub add_timeline_callback: Callback<(TimelinePropsClosure, bool)>,
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
			endpoints: Rc::new(RefCell::new(Vec::new())),
			filters: DEFAULT_FILTERS.into(),
			_agent,
			set_as_main_timeline: false,
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
				let filters = self.filters.clone();
				let set_as_main_timeline = self.set_as_main_timeline;
				ctx.props().add_timeline_callback.emit((Box::new(|id| {
					yew::props! { TimelineProps {
						name,
						id,
						endpoints,
						filters,
					}}
				}), set_as_main_timeline));

				self.endpoints.replace(Vec::new());
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
					self.endpoints.borrow_mut().clear();
					self.filters = DEFAULT_FILTERS.into();

					self.enabled = true;
					true
				},
				TimelineAgentResponse::AddUserTimeline(_service, username) => {
					if let Some(title) = self.title_ref.cast::<HtmlInputElement>() {
						title.set_value(&username);
					}
					self.endpoints.borrow_mut().clear();
					self.filters = DEFAULT_FILTERS.into();

					self.enabled = true;
					true
				},
				_ => false,
			}
			Msg::SetEnabled(value) => {
				self.enabled = value;
				true
			}
			Msg::ToggleFilterEnabled(filter_instance) => {
				self.filters.remove(&filter_instance);
				self.filters.insert(FilterInstance {
					enabled: !filter_instance.enabled,
					..filter_instance
				});
				true
			}
			Msg::ToggleFilterInverted(filter_instance) => {
				self.filters.remove(&filter_instance);
				self.filters.insert(FilterInstance {
					inverted: !filter_instance.inverted,
					..filter_instance
				});
				true
			}
			Msg::AddFilter((filter, inverted)) => {
				self.filters.insert(FilterInstance {filter, inverted, enabled: true});
				true
			}
			Msg::RemoveFilter(filter_instance) => {
				self.filters.remove(&filter_instance);
				true
			}
			Msg::ToggleSetMainTimeline => {
				self.set_as_main_timeline = !self.set_as_main_timeline;
				false
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
			<ModalCard enabled={self.enabled} modal_title="Add Timeline" close_modal_callback={ctx.link().callback(|_| Msg::SetEnabled(false))} {footer}>
				<div class="field">
					<label class="label">{"Title"}</label>
					<div class="control">
						<input type="text" class="input" ref={self.title_ref.clone()}/>
					</div>
				</div>
				<ChooseEndpoints inside_add_timeline=true timeline_endpoints={Rc::downgrade(&self.endpoints)}/>
				<FiltersOptions
					filters={self.filters.clone()}
					toggle_enabled_callback={ctx.link().callback(Msg::ToggleFilterEnabled)}
					toggle_inverted_callback={ctx.link().callback(Msg::ToggleFilterInverted)}
					remove_callback={ctx.link().callback(Msg::RemoveFilter)}
					add_callback={ctx.link().callback(Msg::AddFilter)}
				/>
				<div class="field">
  					<div class="control">
						<label class="checkbox">
							<input type="checkbox" checked={self.set_as_main_timeline} onclick={ctx.link().callback(|_| Msg::ToggleSetMainTimeline)}/>
							{ " Set as main timeline" }
						</label>
  					</div>
				</div>
			</ModalCard>
		}
	}
}
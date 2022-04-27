use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatcher, Dispatched};

use super::ModalCard;
use crate::components::{Dropdown, DropdownLabel};
use crate::modals::modal_agent::{ModalAgent, ModalRequest, ModalType};
use crate::services::article_actions::Action;
use crate::timeline::{agent::{TimelineAgent, Request as TimelineRequest}, filters::{FilterCollection, FilterMsg, FiltersOptions}, TimelineId};

pub struct BatchActionModal {
	enabled: bool,
	action: Action,
	timeline_idx: Option<usize>,
	filters: FilterCollection,
	_modal_agent: Box<dyn Bridge<ModalAgent>>,
	timeline_agent: Dispatcher<TimelineAgent>,
}

pub enum BatchActionMsg {
	SetEnabled(bool),
	SetAction(Action),
	SetTimeline(Option<usize>),
	Apply,
	FilterMsg(FilterMsg),
}

type Msg = BatchActionMsg;

#[derive(Properties, PartialEq)]
pub struct BatchActionProps {
	pub timeline_ids: Vec<(TimelineId, String)>,
}

impl Component for BatchActionModal {
	type Message = BatchActionMsg;
	type Properties = BatchActionProps;

	fn create(ctx: &Context<Self>) -> Self {
		let mut _modal_agent = ModalAgent::bridge(ctx.link().callback(|_| Msg::SetEnabled(true)));
		_modal_agent.send(ModalRequest::Register(ModalType::BatchAction));

		Self {
			enabled: false,
			action: Action::MarkAsRead,
			timeline_idx: None,
			filters: FilterCollection::new(),
			_modal_agent,
			timeline_agent: TimelineAgent::dispatcher(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::SetEnabled(enabled) => {
				self.enabled = enabled;
				true
			}
			Msg::SetAction(action) => {
				self.action = action;
				true
			}
			Msg::SetTimeline(index) => {
				self.timeline_idx = index;
				true
			}
			Msg::Apply => {
				let timeline_ids = &ctx.props().timeline_ids;
				let timelines = self.timeline_idx.map(|index| vec![timeline_ids[index].0]).unwrap_or_else(|| Vec::new());
				self.timeline_agent.send(TimelineRequest::BatchAction(self.action, timelines, self.filters.clone()));

				self.enabled = false;
				true
			}
			Msg::FilterMsg(filter_msg) => self.filters.update(filter_msg),
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let footer = html! {
			<>
				<button
					class="button card-footer-item"
					onclick={ctx.link().callback(|_| Msg::Apply)}
				>
					{"Apply"}
				</button>
				<button
					class="button card-footer-item"
					onclick={ctx.link().callback(|_| Msg::SetEnabled(false))}
				>
					{"Cancel"}
				</button>
			</>
		};

		html! {
			<ModalCard enabled={self.enabled} modal_title="Batch Action" close_modal_callback={ctx.link().callback(|_| Msg::SetEnabled(false))} {footer}>
				<div class="field">
					<label class="label">{"Action"}</label>
					<div class="control">
						<Dropdown current_label={DropdownLabel::Text(self.action.to_string())}>
							{ for Action::iter().map(|action| html! {
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::SetAction(*action))}>
									{ action.to_string() }
								</a>
							}) }
						</Dropdown>
					</div>
				</div>

				{ self.view_timelines(ctx) }

				<FiltersOptions
					filters={self.filters.clone()}
					callback={ctx.link().callback(Msg::FilterMsg)}
				/>
			</ModalCard>
		}
	}
}

impl BatchActionModal {
	fn view_timelines(&self, ctx: &Context<Self>) -> Html {
		let timeline_ids = &ctx.props().timeline_ids;
		let current_label = self.timeline_idx
			.map(|index| timeline_ids[index].1.clone())
			.unwrap_or_else(|| "None".to_owned());
		let current_label = DropdownLabel::Text(current_label);

		html! {
			<div class="field">
				<label class="label">{"Timeline"}</label>
				<div class="control">
					<Dropdown {current_label}>
						<a key={-1} class="dropdown-item" onclick={ctx.link().callback(|_| Msg::SetTimeline(None))}>
							{ "None" }
						</a>
						{ for timeline_ids.iter().enumerate().map(|(i, (_, name))| html! {

							<a key={i} class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetTimeline(Some(i)))}>
								{ name.clone() }
							</a>
						}) }
					</Dropdown>
				</div>
			</div>
		}
	}
}
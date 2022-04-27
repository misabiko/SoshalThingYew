use std::fmt::{Display, Formatter};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use super::ModalCard;
use crate::components::{Dropdown, DropdownLabel, FA, IconSize};
use crate::modals::modal_agent::{ModalAgent, ModalRequest, ModalType};
use crate::timeline::filters::{FilterCollection, FilterMsg, FiltersOptions};

pub struct BatchActionModal {
	enabled: bool,
	action: Action,
	filters: FilterCollection,
	modal_agent: Box<dyn Bridge<ModalAgent>>,
}

pub enum BatchActionMsg {
	SetEnabled(bool),
	SetAction(Action),
	Apply,
	FilterMsg(FilterMsg),
}

type Msg = BatchActionMsg;

#[derive(Properties, PartialEq)]
pub struct BatchActionProps {

}

impl Component for BatchActionModal {
	type Message = BatchActionMsg;
	type Properties = BatchActionProps;

	fn create(ctx: &Context<Self>) -> Self {
		let mut modal_agent = ModalAgent::bridge(ctx.link().callback(|_| Msg::SetEnabled(true)));
		modal_agent.send(ModalRequest::Register(ModalType::BatchAction));

		Self {
			enabled: false,
			action: Action::MarkAsRead,
			filters: FilterCollection::default(),
			modal_agent,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::SetEnabled(enabled) => {
				self.enabled = enabled;
				true
			}
			Msg::SetAction(action) => {
				self.action = action;
				true
			}
			Msg::Apply => {
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
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::SetAction(Action::MarkAsRead))}> {Action::MarkAsRead.to_string()} </a>
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::SetAction(Action::Hide))}> {Action::Hide.to_string()} </a>
						</Dropdown>
					</div>
				</div>

				<FiltersOptions
					filters={self.filters.clone()}
					callback={ctx.link().callback(Msg::FilterMsg)}
				/>
			</ModalCard>
		}
	}
}

//TODO Use from article_actions
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Action {
	MarkAsRead,
	Hide,
}

impl Display for Action {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::MarkAsRead => write!(f, "Mark As Read"),
			Action::Hide => write!(f, "Hide"),
		}
	}
}
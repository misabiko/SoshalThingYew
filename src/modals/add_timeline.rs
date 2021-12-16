use yew::prelude::*;
use web_sys::HtmlInputElement;
use std::rc::Rc;
use std::cell::RefCell;

use crate::modals::Modal;
use crate::timeline::{Props as TimelineProps};
use crate::services::endpoints::TimelineEndpoints;
use crate::choose_endpoints::ChooseEndpoints;

pub struct AddTimelineModal {
	title_ref: NodeRef,
	endpoints: Rc<RefCell<TimelineEndpoints>>,
}

pub enum Msg {
	AddTimeline,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub add_timeline_callback: Callback<TimelineProps>,
	pub close_modal_callback: Callback<MouseEvent>,
}

impl Component for AddTimelineModal {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			title_ref: NodeRef::default(),
			endpoints: Rc::new(RefCell::new(TimelineEndpoints::default()))
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddTimeline => {
				let name = match self.title_ref.cast::<HtmlInputElement>() {
					Some(input) => input.value(),
					None => "Timeline".to_owned(),
				};
				ctx.props().add_timeline_callback.emit(yew::props! { TimelineProps {
					name,
					endpoints: self.endpoints.borrow().clone(),
				}});

				self.endpoints.replace(TimelineEndpoints::default());
			}
		};

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let footer = html! {
			<>
				<button
					class="button card-footer-item"
					onclick={ctx.link().callback(|_| Msg::AddTimeline)}
				>{"Add"}</button>
				<button class="button card-footer-item" onclick={ctx.props().close_modal_callback.clone()}>{"Cancel"}</button>
			</>
		};
		html! {
			<Modal modal_title="Add Timeline" close_modal_callback={ctx.props().close_modal_callback.clone()} {footer}>
				<div class="field">
					<label class="label">{"Title"}</label>
					<div class="control">
						<input type="text" class="input" ref={self.title_ref.clone()} value="Timeline"/>
					</div>
				</div>
				<ChooseEndpoints timeline_endpoints={Rc::downgrade(&self.endpoints)}/>
			</Modal>
		}
	}
}
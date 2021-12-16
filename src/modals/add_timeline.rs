use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::timeline::{Props as TimelineProps};
use crate::services::endpoints::TimelineEndpoints;

pub struct AddTimelineModal {
	title_ref: NodeRef,
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
					endpoints: TimelineEndpoints {
						start: Vec::new(),
						refresh: Vec::new(),
					}
				}})
			}
		};

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="modal is-active">
				<div class="modal-background"/>
				<div class="modal-content">
					<div class="card">
						<header class="card-header">
							<p class="card-header-title">{"Add Timeline"}</p>
							<button class="card-header-icon">
								<span class="icon">
									<i class="fas fa-times"/>
								</span>
							</button>
						</header>
						<div class="card-content">
							<div class="field">
								<label class="label">{"Title"}</label>
								<div class="control">
									<input type="text" class="input" ref={self.title_ref.clone()} value="Timeline"/>
								</div>
							</div>
						</div>
						<footer class="card-footer">
							<button
								class="button card-footer-item"
								onclick={ctx.link().callback(|_| Msg::AddTimeline)}
							>{"Add"}</button>
							<button class="button card-footer-item" onclick={ctx.props().close_modal_callback.clone()}>{"Cancel"}</button>
						</footer>
					</div>
				</div>
			</div>
		}
	}
}
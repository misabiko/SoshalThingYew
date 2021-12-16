use yew::prelude::*;

pub mod add_timeline;

pub use add_timeline::AddTimelineModal;

pub struct Modal;

pub enum Msg {}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub modal_title: String,
	pub children: Children,
	pub close_modal_callback: Callback<MouseEvent>,
}

impl Component for Modal {
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
							{ for ctx.props().children.iter() }
						</div>
					</div>
				</div>
			</div>
		}
	}
}
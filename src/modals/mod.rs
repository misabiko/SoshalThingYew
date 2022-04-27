use yew::prelude::*;

pub mod modal_agent;
pub mod add_timeline;
pub mod batch_action;

pub use add_timeline::AddTimelineModal;
use crate::components::FA;

#[derive(Properties, PartialEq, Clone)]
pub struct ModalCardProps {
	pub modal_title: String,
	pub children: Children,
	pub close_modal_callback: Callback<MouseEvent>,
	#[prop_or_default]
	pub footer: Html,
	#[prop_or(true)]
	pub enabled: bool,
}

#[function_component(ModalCard)]
pub fn modal_card(props: &ModalCardProps) -> Html {
	html! {
		<div class={classes!("modal", if props.enabled { Some("is-active") } else { None })}>
			<div class="modal-background"/>
			<div class="modal-content">
				<div class="card">
					<header class="card-header">
						<p class="card-header-title">{props.modal_title.clone()}</p>
						<button class="card-header-icon" onclick={props.close_modal_callback.clone()}>
							<FA icon="times"/>
						</button>
					</header>
					<div class="card-content">
						{ for props.children.iter() }
					</div>
					<footer class="card-footer">
						{ props.footer.clone() }
					</footer>
				</div>
			</div>
		</div>
	}
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub children: Children,
	pub close_modal_callback: Callback<MouseEvent>,
	#[prop_or(true)]
	pub enabled: bool,
	#[prop_or_default]
	pub content_style: Option<String>
}

#[function_component(Modal)]
pub fn modal(props: &Props) -> Html {
	//TODO Fix close button not white
	html! {
		<div class={classes!("modal", if props.enabled { Some("is-active") } else { None })}>
			<div class="modal-background"/>
			<div class="modal-content" style={props.content_style.clone()}>
				{ for props.children.iter() }
			</div>
			<button class="modal-close is-large" aria-label="close" onclick={props.close_modal_callback.clone()}>
				<FA icon="times"/>
			</button>
		</div>
	}
}

//TODO Close when clicking outside
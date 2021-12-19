use yew::prelude::*;

pub mod add_timeline;

pub use add_timeline::AddTimelineModal;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub modal_title: String,
	pub children: Children,
	pub close_modal_callback: Callback<MouseEvent>,
	#[prop_or_default]
	pub footer: Html,
	#[prop_or_default]
	pub enabled: bool,
}

#[function_component(Modal)]
pub fn modal(props: &Props) -> Html {
	html! {
		<div class={classes!("modal", if props.enabled { Some("is-active") } else { None })}>
			<div class="modal-background"/>
			<div class="modal-content">
				<div class="card">
					<header class="card-header">
						<p class="card-header-title">{"Choose Endpoint"}</p>
						<button class="card-header-icon" onclick={props.close_modal_callback.clone()}>
							<span class="icon">
								<i class="fas fa-times"/>
							</span>
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

//TODO Close when clicking outside
use yew::prelude::*;

pub struct Dropdown {
	expanded: bool,
}

pub enum Msg {
	ToggleExpanded,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
	pub current_label: String,
	pub children: Children,
}

impl Component for Dropdown {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			expanded: false,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleExpanded => {
				self.expanded = !self.expanded;
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class={classes!("dropdown", if self.expanded { Some("is-active") } else { None })}>
				<div class="dropdown-trigger">
					<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleExpanded)}>
						<span>{ ctx.props().current_label.clone() }</span>
						<span class="icon is-small">
							<i class="fas fa-angle-down"/>
						</span>
					</button>
				</div>
				<div class="dropdown-menu">
					<div class="dropdown-content">
						<div class="timelineButtons">
							{ for ctx.props().children.iter() }
						</div>
					</div>
				</div>
			</div>
		}
	}
}
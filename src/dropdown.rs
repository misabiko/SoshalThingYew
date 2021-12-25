use yew::prelude::*;
use wasm_bindgen::JsCast;

pub struct Dropdown {
	expanded: bool,
}

pub enum Msg {
	ToggleExpanded,
}

#[derive(Clone, PartialEq)]
pub enum DropdownLabel {
	Text(String),
	Icon(String),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
	pub current_label: DropdownLabel,
	#[prop_or_default]
	pub label_classes: Option<Classes>,
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
		let on_content_click = ctx.link().batch_callback(|e: MouseEvent| {
			e.target()
				.map(|t| t.has_type::<web_sys::HtmlDivElement>() || t.has_type::<web_sys::HtmlAnchorElement>())
				.and_then(|b| if b { Some(Msg::ToggleExpanded) } else { None })
		});

		html! {
			<div class={classes!("dropdown", if self.expanded { Some("is-active") } else { None })}>
				<div class="dropdown-trigger">
					<button class={classes!("button", ctx.props().label_classes.clone())} onclick={ctx.link().callback(|_| Msg::ToggleExpanded)}>
						{ match &ctx.props().current_label {
							DropdownLabel::Text(text) => html! {
								<>
									<span>{ text.clone() }</span>
									<span class="icon is-small">
										<i class="fas fa-angle-down"/>
									</span>
								</>
							},
							DropdownLabel::Icon(classes) => html! {
								<span class="icon is-small">
									<i class={classes.clone()}/>
								</span>
							},
						} }
					</button>
				</div>
				<div class="dropdown-menu">
					<div class="dropdown-content" onclick={on_content_click}>
						{ for ctx.props().children.iter() }
					</div>
				</div>
			</div>
		}
	}
}
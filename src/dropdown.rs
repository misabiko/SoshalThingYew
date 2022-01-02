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
	#[prop_or_default]
	pub is_right: bool,
	#[prop_or_default]
	pub on_expanded_change: Option<Callback<bool>>,
}

impl Component for Dropdown {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			expanded: false,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleExpanded => {
				self.expanded = !self.expanded;
				if let Some(on_expanded_change) = &ctx.props().on_expanded_change {
					on_expanded_change.emit(self.expanded);
				}
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let on_content_click = ctx.link().batch_callback(|e: MouseEvent| {
			e.target()
				.and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
				.map(|el| el.class_list().contains("dropdown-item"))
				.and_then(|b| if b { Some(Msg::ToggleExpanded) } else { None })
		});

		html! {
			<div class={classes!("dropdown", if self.expanded { Some("is-active") } else { None }, if ctx.props().is_right { Some("is-right") } else { None })}>
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
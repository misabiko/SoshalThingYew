use yew::prelude::*;

pub struct Sidebar {
	expanded: bool
}

pub enum Msg {
	ToggleExpanded,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub add_timeline_callback: Callback<MouseEvent>,
}

impl Component for Sidebar {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self { expanded: false}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleExpanded => {
				self.expanded = !self.expanded;
				true
			}
		}
	}

	//TODO Fix top button click moving bottom ones
	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<nav id="sidebar">
				{if self.expanded { html! {
					<div class="sidebarMenu">
						<div class="box">
							<div class="block">
								{"Twitter"}
							</div>
							<div class="block">
								<a class="button" href="/proxy/twitter/login">{"Login"}</a>
							</div>
						</div>
					</div>
				}} else { html!{} }}
				<div id="sidebarButtons">
					<div>
						<button title="Expand sidebar" onclick={ctx.link().callback(|_| Msg::ToggleExpanded)}>
							<span class="icon">
								<i class="fas fa-angle-double-right fa-2x"/>
							</span>
						</button>
						<button onclick={ctx.props().add_timeline_callback.clone()} title="Add new timeline">
							<span class="icon">
								<i class="fas fa-plus fa-2x"/>
							</span>
						</button>
					</div>
					<div/>
				</div>
			</nav>
		}
	}
}
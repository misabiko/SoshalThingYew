use yew::prelude::*;

pub struct Sidebar {
	expanded: bool
}

pub enum Msg {
	ToggleExpanded,
}

impl Component for Sidebar {
	type Message = Msg;
	type Properties = ();

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
							{"Twitter"}
							<a class="button" href="/proxy/twitter/login">{"Login"}</a>
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
						<button title="Add new timeline">
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
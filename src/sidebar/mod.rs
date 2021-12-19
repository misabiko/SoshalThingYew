use yew::prelude::*;
use yew_agent::{Dispatcher, Dispatched};

mod ratelimits;

use ratelimits::RateLimitView;
use crate::modals::add_timeline::{TimelineAgent, Request as TimelineAgentRequest};

pub struct Sidebar {
	expanded: bool,
	add_timeline_agent: Dispatcher<TimelineAgent>,
}

pub enum Msg {
	ToggleExpanded,
	AddTimeline,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub children: Children,
}

impl Component for Sidebar {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			expanded: false,
			add_timeline_agent: TimelineAgent::dispatcher(),
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleExpanded => {
				self.expanded = !self.expanded;
				true
			}
			Msg::AddTimeline => {
				self.add_timeline_agent.send(TimelineAgentRequest::AddTimeline);
				false
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
						<div class="box">
							<RateLimitView/>
						</div>
					</div>
				}} else { html!{} }}
				<div id="sidebarButtons">
					<div>
						<button title="Expand sidebar" onclick={ctx.link().callback(|_| Msg::ToggleExpanded)}>
							<span class="icon">
								<i class={classes!("fas", "fa-2x", if self.expanded { "fa-angle-double-left" } else { "fa-angle-double-right" })}/>
							</span>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::AddTimeline)} title="Add new timeline">
							<span class="icon">
								<i class="fas fa-plus fa-2x"/>
							</span>
						</button>
						{ for ctx.props().children.iter() }
					</div>
					<div title="Github">
						<a href="https://github.com/misabiko/SoshalThingYew">
							<button>
								<span class="icon">
									<i class="fab fa-github fa-2x"/>
								</span>
							</button>
						</a>
					</div>
				</div>
			</nav>
		}
	}
}
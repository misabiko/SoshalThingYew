use yew::prelude::*;
use yew_agent::{Dispatcher, Dispatched};

mod endpoint_options;

use endpoint_options::EndpointOptions;
use crate::timeline::agent::{TimelineAgent, TimelineRequest};
use crate::settings::{SettingsAgent, SettingsRequest};
use crate::components::{FA, IconSize, IconType};
use crate::modals::modal_agent::{ModalAgent, ModalRequest, ModalType};

pub struct Sidebar {
	expanded: bool,
	add_timeline_agent: Dispatcher<TimelineAgent>,
	settings_agent: Dispatcher<SettingsAgent>,
	modal_agent: Dispatcher<ModalAgent>,
}

pub enum SidebarMsg {
	ToggleExpanded,
	AddTimeline,
	ShowSettings,
	BatchAction,
}

#[derive(Properties, PartialEq, Clone)]
pub struct SidebarProps {
	pub services: Vec<Html>,
	pub children: Children,
	//pub parent_callback: Callback<SidebarCallback>,
}

type Msg = SidebarMsg;
type Props = SidebarProps;

impl Component for Sidebar {
	type Message = Msg;
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		let mut settings_agent = SettingsAgent::dispatcher();
		settings_agent.send(SettingsRequest::RegisterSidebar);

		Self {
			expanded: false,
			add_timeline_agent: TimelineAgent::dispatcher(),
			settings_agent,
			modal_agent: ModalAgent::dispatcher(),
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::ToggleExpanded => {
				self.expanded = !self.expanded;
				true
			}
			Msg::AddTimeline => {
				self.add_timeline_agent.send(TimelineRequest::AddTimeline);
				false
			}
			Msg::ShowSettings => {
				self.settings_agent.send(SettingsRequest::ShowModal);
				false
			}
			Msg::BatchAction => {
				self.modal_agent.send(ModalRequest::ActivateModal(ModalType::BatchAction));
				false
			}
		}
	}

	//TODO Fix top button click moving bottom ones
	fn view(&self, ctx: &Context<Self>) -> Html {
		let batch_callback = {
			let onclick = ctx.link().callback(|_| Msg::BatchAction);

			html! {
				<button {onclick} title="Batch action">
					<FA icon="a" size={IconSize::X2}/>
				</button>
			}
		};

		html! {
			<nav id="sidebar">
				{if self.expanded { html! {
					<div class="sidebarMenu">
						{ for ctx.props().services.iter().cloned() }
						<div class="box">
							<EndpointOptions/>
						</div>
					</div>
				}} else { html!{} }}
				<div id="sidebarButtons">
					<div>
						<button title="Expand sidebar" onclick={ctx.link().callback(|_| Msg::ToggleExpanded)}>
							<FA icon={if self.expanded { "angle-double-left" } else { "angle-double-right" }} size={IconSize::X2}/>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::AddTimeline)} title="Add new timeline">
							<FA icon="plus" size={IconSize::X2}/>
						</button>
						{ batch_callback }
						{ for ctx.props().children.iter() }
					</div>
					<div>
						<button onclick={ctx.link().callback(|_| Msg::ShowSettings)} title="Settings">
							<FA icon="cog" size={IconSize::X2}/>
						</button>
						<a href="https://github.com/misabiko/SoshalThingYew" title="Github">
							<button>
								<FA icon="github" icon_type={IconType::Brand} size={IconSize::X2}/>
							</button>
						</a>
					</div>
				</div>
			</nav>
		}
	}
}
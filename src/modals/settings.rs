use yew::prelude::*;
use web_sys::HtmlInputElement;
use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext, Bridge, Bridged};
use wasm_bindgen::JsCast;

use super::ModalCard;
use crate::{AppSettings, Container, DisplayMode, OnMediaClick};
use crate::components::{Dropdown, DropdownLabel};
use crate::services::storages::update_favviewer_settings;

pub struct SettingsModal {
	enabled: bool,
	settings_agent: Box<dyn Bridge<SettingsAgent>>,
	favviewer_settings: DisplayMode,
}

pub enum Msg {
	SetEnabled(bool),
	ChangeColumnCount(u8),
	ChangeContainer(Container),
	SettingsResponse(Response),
	ToggleFavViewerSettings,
	ChangeOnMediaClick(OnMediaClick),
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
	pub app_settings: AppSettings,
}

impl Component for SettingsModal {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut settings_agent = SettingsAgent::bridge(ctx.link().callback(Msg::SettingsResponse));
		settings_agent.send(Request::RegisterModal);

		Self {
			enabled: false,
			settings_agent,
			favviewer_settings: DisplayMode::default(),
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::SetEnabled(value) => {
				self.enabled = value;
				true
			}
			Msg::ChangeContainer(c) => {
				if let DisplayMode::Single {container, ..} = &mut self.favviewer_settings {
					*container = c;
				}
				self.settings_agent.send(Request::UpdateFavViewer(self.favviewer_settings));
				true
			}
			Msg::ChangeColumnCount(new_column_count) => {
				if let DisplayMode::Single {column_count, ..} = &mut self.favviewer_settings {
					*column_count = new_column_count;
				}
				self.settings_agent.send(Request::UpdateFavViewer(self.favviewer_settings));
				true
			}
			Msg::SettingsResponse(response) => match response {
				Response::ShowModal => {
					self.enabled = true;
					true
				}
				Response::UpdateFavViewerSettings(settings) => {
					self.favviewer_settings = settings;
					true
				}
				Response::ChangeOnMediaClick(_) => false
			}
			Msg::ToggleFavViewerSettings => {
				self.favviewer_settings = match self.favviewer_settings {
					DisplayMode::Single {..} => DisplayMode::Default,
					//TODO Cache default settings
					DisplayMode::Default => DisplayMode::Single {
						column_count: 3,
						container: Container::Masonry,
					}
				};
				self.settings_agent.send(Request::UpdateFavViewer(self.favviewer_settings));
				true
			}
			Msg::ChangeOnMediaClick(on_media_click) => {
				self.settings_agent.send(Request::ChangeOnMediaClick(on_media_click));
				false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let on_column_count_input = ctx.link().batch_callback(|e: InputEvent|
			e.target()
				.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
				.and_then(|i| i.value().parse::<u8>().ok())
				.map(|v| Msg::ChangeColumnCount(v))
		);

		html! {
			<ModalCard enabled={self.enabled} modal_title="Settings" close_modal_callback={ctx.link().callback(|_| Msg::SetEnabled(false))}>
				<div class="block control">
					<label class="label">{"On Media Click"}</label>
					<Dropdown current_label={DropdownLabel::Text(ctx.props().app_settings.on_media_click.name().to_owned())}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeOnMediaClick(OnMediaClick::Like))}> {OnMediaClick::Like.name()} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeOnMediaClick(OnMediaClick::Expand))}> {OnMediaClick::Expand.name()} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeOnMediaClick(OnMediaClick::MarkAsRead))}> {OnMediaClick::MarkAsRead.name()} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeOnMediaClick(OnMediaClick::Hide))}> {OnMediaClick::Hide.name()} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeOnMediaClick(OnMediaClick::Nothing))}> {OnMediaClick::Nothing.name()} </a>
					</Dropdown>
				</div>
				<div class="field">
  					<div class="control">
						<label class="checkbox">
							<input type="checkbox" checked={matches!(self.favviewer_settings, DisplayMode::Single {..})} onclick={ctx.link().callback(|_| Msg::ToggleFavViewerSettings)}/>
							{ " Single Timeline" }
						</label>
  					</div>
				</div>
				{ if let DisplayMode::Single {container, column_count} = self.favviewer_settings {
					html! {
						<>
							{ match container {
								Container::Column => html! {},
								_ => html! {
									<div class="block control">
										<label class="label">{"Column Count"}</label>
										<input class="input" type="number" value={column_count.to_string()} min=1 oninput={on_column_count_input}/>
									</div>
								},
							} }
							<div class="block control">
								<Dropdown current_label={DropdownLabel::Text(container.name().to_string())}>
									<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Column))}> {"Column"} </a>
									<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Row))}> {"Row"} </a>
									<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Masonry))}> {"Masonry"} </a>
								</Dropdown>
							</div>
						</>
					}
				}else {html! {}} }
			</ModalCard>
		}
	}
}

pub struct SettingsAgent {
	link: AgentLink<Self>,
	favviewer_settings: Option<DisplayMode>,
	modal: Option<HandlerId>,
	sidebar: Option<HandlerId>,
	model: Option<HandlerId>,
}

pub enum Request {
	ShowModal,
	InitFavViewerSettings(DisplayMode),
	UpdateFavViewer(DisplayMode),
	RegisterModal,
	RegisterSidebar,
	RegisterModel,
	ChangeOnMediaClick(OnMediaClick),
}

pub enum Response {
	ShowModal,
	UpdateFavViewerSettings(DisplayMode),
	ChangeOnMediaClick(OnMediaClick),
}

impl Agent for SettingsAgent {
	type Reach = AgentContext<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			favviewer_settings: None,
			modal: None,
			sidebar: None,
			model: None,
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::ShowModal => {
				if let Some(modal) = self.modal {
					self.link.respond(modal, Response::ShowModal);
				}
			}
			Request::InitFavViewerSettings(settings) => {
				self.favviewer_settings = Some(settings);
			}
			Request::UpdateFavViewer(settings) => {
				self.favviewer_settings = Some(settings);
				update_favviewer_settings(settings);
				if let Some(modal) = self.modal {
					self.link.respond(modal, Response::UpdateFavViewerSettings(settings))
				}
			}
			Request::RegisterModal => {
				self.modal = Some(id);
				if let Some(settings) = self.favviewer_settings {
					self.link.respond(id, Response::UpdateFavViewerSettings(settings));
				}
			}
			Request::RegisterSidebar => {
				self.sidebar = Some(id);
			}
			Request::RegisterModel => {
				self.model = Some(id);
			}
			Request::ChangeOnMediaClick(on_media_click) => {
				if let Some(model) = self.model {
					self.link.respond(model, Response::ChangeOnMediaClick(on_media_click));
				}
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		if self.modal == Some(id) {
			self.modal = None
		}else if self.sidebar == Some(id) {
			self.sidebar = None
		}else if self.model == Some(id) {
			self.model = None
		}
	}
}
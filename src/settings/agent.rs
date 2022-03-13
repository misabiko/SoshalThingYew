use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext};

use crate::DisplayMode;
use crate::services::storages::update_favviewer_settings;
use crate::settings::ChangeSettingMsg;

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
	ChangeSetting(ChangeSettingMsg),
}

pub enum Response {
	ShowModal,
	UpdateFavViewerSettings(DisplayMode),
	ChangeSetting(ChangeSettingMsg),
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
			Request::ChangeSetting(change_msg) => {
				if let Some(model) = self.model {
					self.link.respond(model, Response::ChangeSetting(change_msg));
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
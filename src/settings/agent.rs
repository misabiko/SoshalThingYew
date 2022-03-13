use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext};

use super::{OnMediaClick, ArticleFilteredMode};
use crate::DisplayMode;
use crate::services::storages::update_favviewer_settings;

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
	//TODO Make subrequest enum
	ChangeOnMediaClick(OnMediaClick),
	ChangeSocialFilteredMode(ArticleFilteredMode),
}

pub enum Response {
	ShowModal,
	UpdateFavViewerSettings(DisplayMode),
	ChangeOnMediaClick(OnMediaClick),
	ChangeSocialFilteredMode(ArticleFilteredMode),
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
			Request::ChangeSocialFilteredMode(article_filtered_mode) => {
				if let Some(model) = self.model {
					self.link.respond(model, Response::ChangeSocialFilteredMode(article_filtered_mode));
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
pub mod component;

pub use component::{SettingsModal, SettingsAgent, Request as SettingsRequest, Response as SettingsResponse};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AppSettings {
	pub on_media_click: OnMediaClick,
	//social filtered out effect {nothing, minimized, transparent}
}

impl AppSettings {
	pub fn override_settings(&self, settings_override: &AppSettingsOverride) -> Self {
		Self {
			on_media_click: settings_override.on_media_click.unwrap_or(self.on_media_click)
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct AppSettingsOverride {
	pub on_media_click: Option<OnMediaClick>,
}

//TODO Have a ArticleAction enum
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OnMediaClick {
	Like,
	Expand,
	MarkAsRead,
	Hide,
	Nothing,
}

impl OnMediaClick {
	pub fn name(&self) -> &'static str {
		match self {
			OnMediaClick::Like => "Like",
			OnMediaClick::Expand => "Expand",
			OnMediaClick::MarkAsRead => "Mark As Read",
			OnMediaClick::Hide => "Hide",
			OnMediaClick::Nothing => "Nothing",
		}
	}
}
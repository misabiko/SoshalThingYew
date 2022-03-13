mod agent;
mod component;

use std::fmt::{Display, Formatter};
pub use component::{
	SettingsModal,
	view_on_media_click_setting,
	view_article_filtered_mode_setting,
	view_keep_column_count_setting,
	view_masonry_independent_columns_setting
};
pub use agent::{SettingsAgent, Request as SettingsRequest, Response as SettingsResponse};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AppSettings {
	pub on_media_click: OnMediaClick,
	pub article_filtered_mode: ArticleFilteredMode,
	pub keep_column_count: bool,
	pub masonry_independent_columns: bool,
}

impl AppSettings {
	pub fn override_settings(&self, settings_override: &AppSettingsOverride) -> Self {
		Self {
			on_media_click: settings_override.on_media_click.unwrap_or(self.on_media_click),
			article_filtered_mode: settings_override.article_filtered_mode.unwrap_or(self.article_filtered_mode),
			keep_column_count: settings_override.keep_column_count.unwrap_or(self.keep_column_count),
			masonry_independent_columns: settings_override.masonry_independent_columns.unwrap_or(self.masonry_independent_columns),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct AppSettingsOverride {
	pub on_media_click: Option<OnMediaClick>,
	pub article_filtered_mode: Option<ArticleFilteredMode>,
	pub keep_column_count: Option<bool>,
	pub masonry_independent_columns: Option<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChangeSettingMsg {
	OnMediaClick(OnMediaClick),
	ArticleFilteredMode(ArticleFilteredMode),
	KeepColumnCount(bool),
	MasonryIndependentColumns(bool),
}

//TODO Have a ArticleAction enum
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OnMediaClick {
	Like,
	Repost,
	Expand,
	MarkAsRead,
	Hide,
	Nothing,
}

const ALL_ONMEDIACLICK: [OnMediaClick; 6] = [
	OnMediaClick::Like,
	OnMediaClick::Repost,
	OnMediaClick::Expand,
	OnMediaClick::MarkAsRead,
	OnMediaClick::Hide,
	OnMediaClick::Nothing,
];

//TODO Make macro to iter enums
impl OnMediaClick {
	pub fn iter() -> impl ExactSizeIterator<Item = &'static OnMediaClick> { ALL_ONMEDIACLICK.iter() }
}

impl Display for OnMediaClick {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			OnMediaClick::MarkAsRead => f.write_str("Mark As Read"),
			_ => f.write_fmt(format_args!("{:?}", self)),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArticleFilteredMode {
	Visible,
	Hidden,
	Transparent,
	Minimized,
}

const ALL_ARTICLEFILTEREDMODE: [ArticleFilteredMode; 4] = [
	ArticleFilteredMode::Visible,
	ArticleFilteredMode::Hidden,
	ArticleFilteredMode::Minimized,
	ArticleFilteredMode::Transparent,
];

impl ArticleFilteredMode {
	pub fn iter() -> impl ExactSizeIterator<Item = &'static ArticleFilteredMode> { ALL_ARTICLEFILTEREDMODE.iter() }
}

impl Display for ArticleFilteredMode {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}
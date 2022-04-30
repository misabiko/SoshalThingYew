use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;

use super::{AppSettings, ChangeSettingMsg, OnMediaClick, ArticleFilteredMode, SettingsAgent, SettingsResponse, SettingsRequest};
use crate::modals::ModalCard;
use crate::components::{Dropdown, DropdownLabel};
use crate::{Container, DisplayMode};

pub struct SettingsModal {
	enabled: bool,
	settings_agent: Box<dyn Bridge<SettingsAgent>>,
	favviewer_settings: DisplayMode,
}

pub enum SettingsModalMsg {
	SetEnabled(bool),
	ChangeColumnCount(u8),
	ChangeContainer(Container),
	SettingsResponse(SettingsResponse),
	ToggleFavViewerSettings,
	ChangeSetting(ChangeSettingMsg),
}

#[derive(Properties, PartialEq, Clone)]
pub struct SettingsModalProps {
	pub app_settings: AppSettings,
}

type Msg = SettingsModalMsg;
type Props = SettingsModalProps;

impl Component for SettingsModal {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut settings_agent = SettingsAgent::bridge(ctx.link().callback(Msg::SettingsResponse));
		settings_agent.send(SettingsRequest::RegisterModal);

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
				self.settings_agent.send(SettingsRequest::UpdateFavViewer(self.favviewer_settings));
				true
			}
			Msg::ChangeColumnCount(new_column_count) => {
				if let DisplayMode::Single {column_count, ..} = &mut self.favviewer_settings {
					*column_count = new_column_count;
				}
				self.settings_agent.send(SettingsRequest::UpdateFavViewer(self.favviewer_settings));
				true
			}
			Msg::SettingsResponse(response) => match response {
				SettingsResponse::ShowModal => {
					self.enabled = true;
					true
				}
				SettingsResponse::UpdateFavViewerSettings(settings) => {
					self.favviewer_settings = settings;
					true
				}
				SettingsResponse::ChangeSetting(_) => false,
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
				self.settings_agent.send(SettingsRequest::UpdateFavViewer(self.favviewer_settings));
				true
			}
			Msg::ChangeSetting(change_msg) => {
				self.settings_agent.send(SettingsRequest::ChangeSetting(change_msg));
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
				{ view_on_media_click_setting(
					ctx.props().app_settings.on_media_click,
					ctx.link().callback(Msg::ChangeSetting)
				) }
				{ view_article_filtered_mode_setting(
					ctx.props().app_settings.article_filtered_mode,
					ctx.link().callback(Msg::ChangeSetting)
				) }
				{ view_keep_column_count_setting(
					ctx.props().app_settings.keep_column_count,
					ctx.link().callback(Msg::ChangeSetting)
				) }
				{ view_masonry_independent_columns_setting(
					ctx.props().app_settings.masonry_independent_columns,
					ctx.link().callback(Msg::ChangeSetting)
				) }
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

pub fn view_on_media_click_setting(current: OnMediaClick, callback: Callback<ChangeSettingMsg>) -> Html {
	html! {
		<div class="block control">
			<label class="label">{"On Media Click"}</label>
			<Dropdown current_label={DropdownLabel::Text(current.to_string())}>
				{ for OnMediaClick::iter().map(|item| {
					let callback = callback.clone();
					html! {
						<a class="dropdown-item" onclick={Callback::from(move |_| callback.emit(ChangeSettingMsg::OnMediaClick(*item)))}> {item.to_string()} </a>
					}
				}) }
			</Dropdown>
		</div>
	}
}

pub fn view_article_filtered_mode_setting(current: ArticleFilteredMode, callback: Callback<ChangeSettingMsg>) -> Html {
	html! {
		<div class="block control">
			<label class="label">{"Article Filtered Mode"}</label>
			<Dropdown current_label={DropdownLabel::Text(current.to_string())}>
				{ for ArticleFilteredMode::iter().map(|item| {
					let callback = callback.clone();
					html! {
						<a class="dropdown-item" onclick={Callback::from(move |_| callback.emit(ChangeSettingMsg::ArticleFilteredMode(*item)))}> {item.to_string()} </a>
					}
				}) }
			</Dropdown>
		</div>
	}
}

pub fn view_keep_column_count_setting(checked: bool, callback: Callback<ChangeSettingMsg>) -> Html {
	html! {
		<div class="block control">
			<label class="checkbox">
				<input type="checkbox" {checked} onclick={Callback::from(move |_| callback.emit(ChangeSettingMsg::KeepColumnCount(!checked)))}/>
				{ " Keep column count when not enough articles" }
			</label>
		</div>
	}
}

pub fn view_masonry_independent_columns_setting(checked: bool, callback: Callback<ChangeSettingMsg>) -> Html {
	html! {
		<div class="block control">
			<label class="checkbox">
				<input type="checkbox" {checked} onclick={Callback::from(move |_| callback.emit(ChangeSettingMsg::MasonryIndependentColumns(!checked)))}/>
				{ " Keep articles on the same column in masonry" }
			</label>
		</div>
	}
}
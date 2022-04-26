use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};
use std::collections::HashMap;
use gloo_storage::Storage;

pub mod articles;
pub mod choose_endpoints;
pub mod components;
pub mod error;
pub mod favviewer;
pub mod modals;
pub mod notifications;
pub mod services;
pub mod settings;
pub mod timeline;
mod sidebar;

use components::{FA, IconSize};
use error::Result;
use favviewer::PageInfo;
use settings::{AppSettings, ArticleFilteredMode, OnMediaClick, SettingsModal, SettingsAgent, SettingsRequest, SettingsResponse};
use notifications::{NotificationAgent, Request as NotificationRequest, Response as NotificationResponse};
use services::{
	Endpoint,
	endpoint_agent::{EndpointId, EndpointAgent, TimelineEndpointWrapper, Request as EndpointRequest},
	pixiv::PixivAgent,
	dummy_service::DummyServiceAgent,
	twitter::{endpoints::*, TwitterAgent, Request as TwitterRequest, Response as TwitterResponse, SERVICE_INFO as TwitterServiceInfo},
	youtube::{YouTubeAgent, Request as YouTubeRequest, Response as YouTubeResponse, SERVICE_INFO as YouTubeServiceInfo},
	storages::SoshalLocalStorage,
};
use sidebar::Sidebar;
use timeline::{
	Container,
	timeline_container::{TimelineContainer, DisplayMode},
	agent::{TimelineAgent, Request as TimelineAgentRequest, Response as TimelineAgentResponse},
};
use crate::settings::ChangeSettingMsg;

#[derive(serde::Deserialize)]
pub struct AuthInfo {
	twitter: Option<String>,
	youtube: bool,
}

pub struct Model {
	endpoint_agent: Dispatcher<EndpointAgent>,
	_timeline_agent: Box<dyn Bridge<TimelineAgent>>,
	display_mode: DisplayMode,
	last_display_single: DisplayMode,
	page_info: Option<PageInfo>,
	twitter: Box<dyn Bridge<TwitterAgent>>,
	_pixiv: Dispatcher<PixivAgent>,
	_dummy_service: Dispatcher<DummyServiceAgent>,
	youtube: Box<dyn Bridge<YouTubeAgent>>,
	services_sidebar: HashMap<String, Html>,
	_notification_agent: Box<dyn Bridge<NotificationAgent>>,
	notifications: Vec<Html>,
	sidebar_favviewer: bool,
	app_settings: AppSettings,
	_settings_agent: Box<dyn Bridge<SettingsAgent>>,
}

pub enum Msg {
	EndpointAgentRequest(EndpointRequest),
	TimelineAgentResponse(TimelineAgentResponse),
	TwitterResponse(TwitterResponse),
	YouTubeResponse(YouTubeResponse),
	FetchedAuthInfo(Result<AuthInfo>),
	NotificationResponse(NotificationResponse),
	SettingsResponse(SettingsResponse),
	TimelineContainerCallback(TimelineContainerCallback),
}

#[derive(Properties, PartialEq, Default)]
pub struct Props {
	pub favviewer: bool,
	#[prop_or_default]
	pub display_mode: Option<DisplayMode>,
	#[prop_or_default]
	pub page_info: Option<PageInfo>,
	#[prop_or_default]
	pub services_sidebar: HashMap<String, Html>,
}

impl Component for Model {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut _notification_agent = NotificationAgent::bridge(ctx.link().callback(Msg::NotificationResponse));
		_notification_agent.send(NotificationRequest::RegisterTimelineContainer);

		let mut twitter = TwitterAgent::bridge(ctx.link().callback(Msg::TwitterResponse));
		twitter.send(TwitterRequest::Sidebar);
		let _pixiv = PixivAgent::dispatcher();
		let _dummy_service = DummyServiceAgent::dispatcher();
		let mut youtube = YouTubeAgent::bridge(ctx.link().callback(Msg::YouTubeResponse));
		youtube.send(YouTubeRequest::Sidebar);

		let mut _timeline_agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
		_timeline_agent.send(TimelineAgentRequest::RegisterDisplayMode);

		let mut _settings_agent = SettingsAgent::bridge(ctx.link().callback(Msg::SettingsResponse));
		_settings_agent.send(SettingsRequest::RegisterModel);

		let (_, search_opt) = parse_url();

		//TODO use memreplace Some(Setup) → None
		let page_info = match &ctx.props().page_info {
			Some(PageInfo::Setup { style_html, initial_style, make_activator, add_timelines }) => {
				(add_timelines)();

				Some(PageInfo::Ready {
					style_html: style_html.clone(),
					style: initial_style.clone(),
					favviewer_button: (make_activator)(ctx.link().callback(|_| Msg::TimelineContainerCallback(TimelineContainerCallback::ToggleFavViewer))),
				})
			}
			_ => None,
		};

		let single_timeline_bool = search_opt.as_ref()
			.and_then(|s| s.get("single_timeline"))
			.and_then(|s| s.parse().ok())
			.unwrap_or_default();

		let display_mode = if let Some(SoshalLocalStorage { display_mode, .. }) = gloo_storage::LocalStorage::get("SoshalThingYew").ok() {
			display_mode
		} else if let Some(display_mode) = &ctx.props().display_mode {
			*display_mode
		} else if single_timeline_bool {
			DisplayMode::Single {
				container: search_opt.as_ref()
					.and_then(|s| s.get("container"))
					.as_ref().and_then(|s| Container::from(s).ok())
					.unwrap_or(Container::Masonry),
				column_count: search_opt.as_ref()
					.and_then(|s| s.get("column_count"))
					.and_then(|s| s.parse().ok())
					.unwrap_or(4),
			}
		} else {
			DisplayMode::Default
		};

		if !ctx.props().favviewer {
			ctx.link().send_future(async {
				Msg::FetchedAuthInfo(fetch_auth_info().await)
			});
		}

		Self {
			last_display_single: match display_mode {
				DisplayMode::Single { .. } => display_mode,
				_ => DisplayMode::Single {
					container: Container::Masonry,
					column_count: 4,
				},
			},
			display_mode,
			_timeline_agent,
			endpoint_agent: EndpointAgent::dispatcher(),
			page_info,
			twitter,
			_pixiv,
			_dummy_service,
			youtube,
			services_sidebar: ctx.props().services_sidebar.clone(),
			_notification_agent,
			notifications: Vec::new(),
			sidebar_favviewer: false,
			app_settings: AppSettings {
				on_media_click: OnMediaClick::MarkAsRead,
				article_filtered_mode: ArticleFilteredMode::Hidden,
				keep_column_count: true,
				masonry_independent_columns: true,
			},
			_settings_agent,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::TimelineContainerCallback(callback) => match callback {
				TimelineContainerCallback::ToggleFavViewer => {
					if let Some(page_info) = &mut self.page_info {
						page_info.toggle_hidden();
						true
					} else {
						false
					}
				}
				TimelineContainerCallback::ToggleSidebarFavViewer => {
					self.sidebar_favviewer = !self.sidebar_favviewer;
					true
				}
				TimelineContainerCallback::ToggleDisplayMode => {
					self.display_mode = match self.display_mode {
						DisplayMode::Default => self.last_display_single,
						DisplayMode::Single { .. } => DisplayMode::Default,
					};
					true
				}
			}
			Msg::TimelineAgentResponse(response) => match response {
				TimelineAgentResponse::SetMainContainer(new_container) => {
					if let DisplayMode::Single { container, .. } = &mut self.display_mode {
						*container = new_container;
						true
					} else {
						log::warn!("DisplayMode not single");
						false
					}
				}
				TimelineAgentResponse::SetMainColumnCount(new_count) => {
					if let DisplayMode::Single { column_count, .. } = &mut self.display_mode {
						*column_count = new_count;
						true
					} else {
						log::warn!("DisplayMode not single");
						false
					}
				}
				_ => false,
			}
			Msg::EndpointAgentRequest(request) => {
				self.endpoint_agent.send(request);
				false
			}
			Msg::TwitterResponse(response) => {
				match response {
					TwitterResponse::Sidebar(html) => self.services_sidebar.insert(TwitterServiceInfo.name.to_owned(), html),
				};
				true
			}
			Msg::YouTubeResponse(response) => {
				match response {
					YouTubeResponse::Sidebar(html) => self.services_sidebar.insert(YouTubeServiceInfo.name.to_owned(), html),
				};
				true
			}
			Msg::FetchedAuthInfo(response) => {
				match response {
					Ok(auth_info) => {
						self.twitter.send(TwitterRequest::Auth(auth_info.twitter));
						self.youtube.send(YouTubeRequest::Auth(auth_info.youtube));
					}
					Err(err) => log::error!("{}", err),
				};
				false
			}
			Msg::NotificationResponse(response) => {
				match response {
					NotificationResponse::DrawNotifications(notifs) => self.notifications = notifs,
				};
				true
			}
			Msg::SettingsResponse(response) => match response {
				SettingsResponse::ChangeSetting(change_msg) => {
					match change_msg {
						ChangeSettingMsg::OnMediaClick(on_media_click) => self.app_settings.on_media_click = on_media_click,
						ChangeSettingMsg::ArticleFilteredMode(article_filtered_mode) => self.app_settings.article_filtered_mode = article_filtered_mode,
						ChangeSettingMsg::KeepColumnCount(keep_column_count) => self.app_settings.keep_column_count = keep_column_count,
						ChangeSettingMsg::MasonryIndependentColumns(masonry_independent_columns) => self.app_settings.masonry_independent_columns = masonry_independent_columns,
					}
					true
				}
				_ => false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let display_mode_toggle = {
			let (dm_title, dm_icon) = match self.display_mode {
				DisplayMode::Default => ("Single Timeline", "expand-alt"),
				DisplayMode::Single { .. } => ("Multiple Timelines", "columns"),
			};

			html! {
				<button onclick={ctx.link().callback(|_| Msg::TimelineContainerCallback(TimelineContainerCallback::ToggleDisplayMode))} title={dm_title}>
					<FA icon={dm_icon} size={IconSize::X2}/>
				</button>
			}
		};

		html! {
			<>
				<SettingsModal app_settings={self.app_settings}/>
				<div id="soshal-notifications">
					{ for self.notifications.iter().cloned() }
				</div>
				{ self.page_info.as_ref().map(|p| p.view()).unwrap_or_default() }
				{
					match self.sidebar_favviewer || !ctx.props().favviewer {
						true => html! {
							<Sidebar services={self.services_sidebar.values().cloned().collect::<Vec<Html>>()}>
								{ display_mode_toggle }
							</Sidebar>
						},
						false => html! {},
					}
				}
				<TimelineContainer
					parent_callback={ctx.link().callback(Msg::TimelineContainerCallback)}
					app_settings={self.app_settings}
					favviewer={ctx.props().favviewer}
					display_mode={self.display_mode}
				/>
			</>
		}
	}
}

pub enum TimelineContainerCallback {
	ToggleFavViewer,
	ToggleSidebarFavViewer,
	ToggleDisplayMode,
}

pub fn parse_url() -> (Option<String>, Option<web_sys::UrlSearchParams>) {
	match web_sys::window().map(|w| w.location()) {
		Some(location) => (match location.pathname() {
			Ok(pathname_opt) => Some(pathname_opt),
			Err(err) => {
				log_error!("Failed to get location.pathname", err);
				None
			}
		}, match location.search().and_then(|s| web_sys::UrlSearchParams::new_with_str(&s)) {
			Ok(search_opt) => Some(search_opt),
			Err(err) => {
				log_error!("Failed to get location.search", err);
				None
			}
		}),
		None => (None, None),
	}
}

//Maybe move to a generic util module?
pub fn base_url() -> String {
	let location = web_sys::window().unwrap().location();
	let host = location.host().unwrap();
	let protocol = location.protocol().unwrap();

	format!("{}//{}", protocol, host)
}

async fn fetch_auth_info() -> Result<AuthInfo> {
	Ok(reqwest::Client::builder()
		//.timeout(Duration::from_secs(10))
		.build()?
		.get(format!("{}/proxy/auth_info", base_url()))
		.send().await?
		.json::<AuthInfo>().await?)
}
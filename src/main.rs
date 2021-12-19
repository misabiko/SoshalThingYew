use yew::prelude::*;
use yew_agent::{Bridge, Dispatched, Dispatcher};
use yew_agent::utils::store::{Bridgeable, ReadOnly, StoreWrapper};
use std::collections::HashSet;

pub mod error;
pub mod timeline;
pub mod articles;
pub mod services;
pub mod modals;
pub mod dropdown;
mod sidebar;
mod favviewer;
pub mod choose_endpoints;

use crate::sidebar::Sidebar;
use crate::timeline::{Props as TimelineProps, Timeline};
use crate::services::{
	endpoints::{Endpoint, EndpointId, EndpointStore, Request as EndpointRequest, TimelineEndpoints},
	pixiv::{FollowEndpoint, PixivAgent},
	twitter::{endpoints::{HomeTimelineEndpoint, SingleTweetEndpoint, UserTimelineEndpoint}, TwitterAgent},
};
use crate::favviewer::{PageInfo, PixivPageInfo};
use crate::modals::AddTimelineModal;

#[derive(PartialEq, Clone)]
enum DisplayMode {
	Single {
		column_count: u8
	},
	Default
}

impl Default for DisplayMode {
	fn default() -> Self {
		DisplayMode::Default
	}
}

struct Model {
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	display_mode: DisplayMode,
	timelines: Vec<TimelineProps>,
	page_info: Option<Box<dyn PageInfo>>,
	_twitter: Dispatcher<TwitterAgent>,
	_pixiv: Dispatcher<PixivAgent>,
	timeline_counter: i16,
}

enum Msg {
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	AddTimeline(String, EndpointId),
	ToggleFavViewer,
	AddTimelineProps(Box<dyn FnOnce(i16) -> TimelineProps>),
	EndpointStoreResponse(ReadOnly<EndpointStore>),
	ToggleDisplayMode,
}

#[derive(Properties, PartialEq, Default)]
struct Props {
	favviewer: bool,
	#[prop_or_default]
	display_mode: Option<DisplayMode>,
}

impl Component for Model {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let (pathname_opt, search_opt) = parse_url();

		if let Some(pathname) = pathname_opt {
			parse_pathname(ctx, &pathname, &search_opt);
		}

		let single_timeline_bool = search_opt.as_ref()
			.and_then(|s| s.get("single_timeline"))
			.and_then(|s| s.parse().ok())
			.unwrap_or_default();

		let display_mode = if let Some(display_mode) = &ctx.props().display_mode {
			(*display_mode).clone()
		} else if single_timeline_bool {
			DisplayMode::Single {
				column_count: search_opt.as_ref()
					.and_then(|s| s.get("column_count"))
					.and_then(|s| s.parse().ok())
					.unwrap_or(1),
			}
		}else {
			DisplayMode::Default
		};

		//TODO Detect current page
		let page_info = match ctx.props().favviewer {
			true => Some(Box::new(PixivPageInfo::new(ctx.link().callback(|_| Msg::ToggleFavViewer))) as Box<dyn PageInfo>),
			false => None
		};

		Self {
			display_mode,
			timelines: Vec::new(),
			endpoint_store: EndpointStore::bridge(ctx.link().callback(Msg::EndpointStoreResponse)),
			page_info,
			_twitter: TwitterAgent::dispatcher(),
			_pixiv: PixivAgent::dispatcher(),
			timeline_counter: i16::MIN,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::AddEndpoint(e) => {
				self.endpoint_store.send(EndpointRequest::AddEndpoint(e));
				false
			}
			Msg::AddTimeline(name, id) => {
				let mut endpoints = HashSet::new();
				endpoints.insert(id);
				self.timelines.push(yew::props! { TimelineProps {
					name,
					id: self.timeline_counter,
					endpoints: TimelineEndpoints {
						start: endpoints.clone(),
						refresh: endpoints,
					}
				}});
				self.timeline_counter += 1;
				true
			}
			Msg::AddTimelineProps(props) => {
				self.timelines.push((props)(self.timeline_counter.clone()));
				self.timeline_counter += 1;

				true
			}
			Msg::ToggleFavViewer => {
				if let Some(page_info) = &mut self.page_info {
					page_info.toggle_hidden();
					true
				}else {
					false
				}
			}
			Msg::EndpointStoreResponse(_) => false,
			Msg::ToggleDisplayMode => {
				self.display_mode = match self.display_mode {
					DisplayMode::Default => DisplayMode::Single {column_count: 5},
					DisplayMode::Single { .. } => DisplayMode::Default,
				};
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let (dm_title, dm_icon) = match self.display_mode {
			DisplayMode::Default => ("Single Timeline", "fa-expand-alt"),
			DisplayMode::Single { .. } => ("Multiple Timeline", "fa-columns"),
		};
		let display_mode_toggle = html! {
			<button onclick={ctx.link().callback(|_| Msg::ToggleDisplayMode)} title={dm_title}>
				<span class="icon">
					<i class={classes!("fas", "fa-2x", dm_icon)}/>
				</span>
			</button>
		};

		html! {
			<>
				<AddTimelineModal add_timeline_callback={ctx.link().callback(|props| Msg::AddTimelineProps(props))}/>
				{
					self.page_info
						.as_ref()
						.map(Box::as_ref)
						.map(PageInfo::view)
						.unwrap_or_default()
				}
				{
					match ctx.props().favviewer {
						false => html! {
							<Sidebar>
								{ display_mode_toggle }
							</Sidebar>
						},
						true => html! {},
					}
				}
				<div id="timelineContainer">
					{
						match self.display_mode {
							DisplayMode::Default => html! {
								{for self.timelines.iter().map(|props| html! {
									<Timeline key={props.id.clone()} ..props.clone()/>
								})}
							},
							DisplayMode::Single {column_count} => html! {
								{for self.timelines.iter().enumerate().map(|(i, props)| match i {
									0 => html! {
										<Timeline key={props.id.clone()} main_timeline=true {column_count} ..props.clone()>
											{
												match ctx.props().favviewer {
													true => html! {
														<button title="Toggle FavViewer" onclick={ctx.link().callback(|_| Msg::ToggleFavViewer)}>
															<span class="icon">
																<i class="fas fa-eye-slash fa-lg"/>
															</span>
														</button>
													},
													false => html! {}
												}
											}
										</Timeline>
									},
									_ => html! {
										<Timeline hide=true key={props.id.clone()} ..props.clone()/>
									}
								})}
							}
						}
					}
				</div>
			</>
		}
	}
}

fn parse_url() -> (Option<String>, Option<web_sys::UrlSearchParams>) {
	match web_sys::window().map(|w| w.location()) {
		Some(location) => (match location.pathname() {
			Ok(pathname_opt) => Some(pathname_opt),
			Err(err) => {
				log::error!("Failed to get location.pathname.\n{:?}", err);
				None
			}
		}, match location.search().and_then(|s| web_sys::UrlSearchParams::new_with_str(&s)) {
			Ok(search_opt) => Some(search_opt),
			Err(err) => {
				log::error!("Failed to get location.search.\n{:?}", err);
				None
			}
		}),
		None => (None, None),
	}
}

fn parse_pathname(ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>) {
	if let Some(tweet_id) = pathname.strip_prefix("/twitter/status/").and_then(|s| s.parse::<u64>().ok()) {
		let callback = ctx.link().callback(|id|Msg::AddTimeline("Tweet".to_owned(), id));

		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(SingleTweetEndpoint::new(id, tweet_id.clone()))
			}))
		);
	} else if let Some(username) = pathname.strip_prefix("/twitter/user/").map(str::to_owned) {
		let (retweets, replies) = match search_opt {
			Some(search) => (
				search.get("rts")
					.and_then(|s| s.parse().ok())
					.unwrap_or_default(),
				search.get("replies")
				.and_then(|s| s.parse().ok())
				.unwrap_or_default()
			),
			None => (false, false)
		};
		let callback = ctx.link().callback(|id| Msg::AddTimeline("User".to_owned(), id));

		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(UserTimelineEndpoint::new(id, username.clone(), retweets, replies))
			}))
		);
	} else if pathname.starts_with("/twitter/home") {
		let callback = ctx.link().callback( |id| Msg::AddTimeline("Home".to_owned(), id));
		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(HomeTimelineEndpoint::new(id))
			}))
		);
	} else if ctx.props().favviewer {
		let callback = ctx.link().callback(|id| Msg::AddTimeline("Pixiv".to_owned(), id));
		ctx.link().send_message(
			Msg::AddEndpoint(Box::new(move |id| {
				callback.emit(id);
				Box::new(FollowEndpoint::new(id))
			}))
		);
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

	match web_sys::window()
		.map(|w| w.location())
		.map(|l| l.href()) {
		Some(Ok(href)) => {
			let href = href.as_str();
			if href.contains("pixiv.net/bookmark_new_illust") {
				let mount_point = gloo_utils::document().create_element("div").expect("to create empty div");
				mount_point.set_id("favviewer");

				gloo_utils::document()
					.query_selector("#root > div:last-child > div:nth-child(2)")
					.expect("can't get mount node for rendering")
					.expect("can't unwrap mount node")
					.append_with_node_1(&mount_point)
					.expect("can't append mount node");

				yew::start_app_with_props_in_element::<Model>(mount_point, yew::props! { Props {
					favviewer: true,
					display_mode: DisplayMode::Single {
						column_count: 5,
					}
				}});
			}else {
				yew::start_app::<Model>();
			}
		},
		None => log::error!("Failed to get location.href."),
		Some(Err(err)) => log::error!("Failed to get location.href.\n{}", &err.as_string().unwrap_or("Failed to parse the error.".to_string())),
	};
}

//TODO Save fetched articles
//TODO Load timeline data
//TODO Parse tweet text
//TODO Auto refresh
//TODO Display timeline errors
//TODO Youtube articles
//TODO Notifications
//TODO Save timeline data
//TODO Social expanded view
//TODO Prompt on not logged in
//TODO Avoid refreshing endpoint every watch update
//TODO HTTPS

//TODO Show multiple article types in same timeline

//TODO Fix articles not redrawing when they redraw...
//TODO Fix fontawesome sass warnings
//TODO Fix handler without callback thing
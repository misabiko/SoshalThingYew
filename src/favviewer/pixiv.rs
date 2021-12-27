use yew::prelude::*;
use std::collections::HashMap;
use gloo_timers::callback::Timeout;

use crate::favviewer::PageInfo;
use crate::{Model, Props as ModelProps, Msg as ModelMsg, DisplayMode, FollowPageEndpoint, FollowAPIEndpoint};

#[derive(PartialEq, Eq, Hash)]
pub enum Style {
	Hidden,
	Pixiv,
}

pub struct FollowPageInfo {
	style_html: HashMap<Style, Html>,
	style: Style,
	favviewer_button: Html,
}

impl FollowPageInfo {
	pub fn new(on_activator_click: Callback<web_sys::MouseEvent>) -> Self {
		let document_head = gloo_utils::document().head().expect("head element to be present");
		let mut style_html = HashMap::new();
		style_html.insert(Style::Pixiv, create_portal(html! {
                <style>{"#root {width: 100%} #root > :nth-child(2), .sc-1nr368f-2.bGUtlw { height: 100%; } .sc-jgyytr-1 {display: none}"}</style>
			}, document_head.clone().into()
		));
		style_html.insert(Style::Hidden, create_portal(html! {
                <style>{"#favviewer {display: none;} #root {width: 100%} "}</style>
			}, document_head.into()
		));

		let favviewer_button_mount = gloo_utils::document()
			.query_selector(".sc-s8zj3z-6.kstoDd")
			.expect("couldn't query activator mount point")
			.expect("couldn't find activator mount point");
		let favviewer_button = create_portal(html! {
			<a class="sc-d98f2c-0" onclick={on_activator_click}>
				<span class="sc-93qi7v-2 ibdURy">{"FavViewer"}</span>
			</a>
		}, favviewer_button_mount.into());

		Self {
			style_html,
			style: Style::Hidden,
			favviewer_button,
		}
	}
}

impl PageInfo for FollowPageInfo {
	fn style_html(&self) -> Html {
		self.style_html[&self.style].clone()
	}

	fn favviewer_button(&self) -> Html {
		self.favviewer_button.clone()
	}

	fn toggle_hidden(&mut self) {
		self.style = match &self.style {
			Style::Hidden => Style::Pixiv,
			Style::Pixiv => Style::Hidden,
		}
	}

	fn add_timeline(&self, ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>) {
		let callback = ctx.link().callback(|endpoints| ModelMsg::AddTimeline("Pixiv".to_owned(), endpoints));
		let r18 = pathname.contains("r18");
		let current_page = search_opt.as_ref()
			.and_then(|s| s.get("p"))
			.and_then(|s| s.parse().ok())
			.unwrap_or(1);
		ctx.link().send_message(
			ModelMsg::BatchAddEndpoints(vec![Box::new(move |id| {
				Box::new(FollowPageEndpoint::new(id))
			})], vec![Box::new(move |id| {
				Box::new(FollowAPIEndpoint::new(id, r18, current_page - 1))
			})], callback)
		);
	}
}

pub struct UserPageInfo {
	style_html: HashMap<Style, Html>,
	style: Style,
	favviewer_button: Html,
}

impl UserPageInfo {
	pub fn new(on_activator_click: Callback<web_sys::MouseEvent>) -> Self {
		let document_head = gloo_utils::document().head().expect("head element to be present");
		let mut style_html = HashMap::new();
		style_html.insert(Style::Pixiv, create_portal(html! {
                <style>{"#favviewer {width: 100%; height: 50%}#root {width: 100%} #root > :nth-child(2), .sc-1nr368f-2.bGUtlw { height: 100%; } .sc-jgyytr-1 {display: none}"}</style>
			}, document_head.clone().into()
		));
		style_html.insert(Style::Hidden, create_portal(html! {
                <style>{"#favviewer {display: none;} #root {width: 100%} "}</style>
			}, document_head.into()
		));

		let nav = gloo_utils::document()
			.get_elements_by_tag_name("nav").get_with_index(0).expect("couldn't find a nav");
		let activator_classes = nav.last_element_child().map(|c| c.class_name()).unwrap_or_default();

		let favviewer_button_mount = gloo_utils::document()
			.query_selector("nav")
			.expect("couldn't query activator mount point")
			.expect("couldn't find activator mount point");
		let favviewer_button = create_portal(html! {
			<a id="favvieweractivator" class={activator_classes} onclick={on_activator_click}>
				{"FavViewer"}
			</a>
		}, favviewer_button_mount.into());

		Self {
			style_html,
			style: Style::Hidden,
			favviewer_button,
		}
	}
}

impl PageInfo for UserPageInfo {
	fn style_html(&self) -> Html {
		self.style_html[&self.style].clone()
	}

	fn favviewer_button(&self) -> Html {
		self.favviewer_button.clone()
	}

	fn toggle_hidden(&mut self) {
		self.style = match &self.style {
			Style::Hidden => Style::Pixiv,
			Style::Pixiv => Style::Hidden,
		}
	}

	fn add_timeline(&self, ctx: &Context<Model>, pathname: &str, search_opt: &Option<web_sys::UrlSearchParams>) {
		let callback = ctx.link().callback(|endpoints| ModelMsg::AddTimeline("Pixiv".to_owned(), endpoints));
		let r18 = pathname.contains("r18");
		let current_page = search_opt.as_ref()
			.and_then(|s| s.get("p"))
			.and_then(|s| s.parse().ok())
			.unwrap_or(1);
		ctx.link().send_message(
			ModelMsg::BatchAddEndpoints(vec![Box::new(move |id| {
				Box::new(FollowPageEndpoint::new(id))
			})], vec![Box::new(move |id| {
				Box::new(FollowAPIEndpoint::new(id, r18, current_page - 1))
			})], callback)
		);
	}
}

pub fn setup(href: &str) -> bool {
	if href.contains("pixiv.net/bookmark_new_illust") {
		let mount_point = gloo_utils::document().create_element("div").expect("to create empty div");
		mount_point.set_id("favviewer");

		gloo_utils::document()
			.query_selector("#root > div:last-child > div:nth-child(2)")
			.expect("can't get mount node for rendering")
			.expect("can't unwrap mount node")
			.append_with_node_1(&mount_point)
			.expect("can't append mount node");

		yew::start_app_with_props_in_element::<Model>(mount_point, yew::props! { ModelProps {
			favviewer: true,
			display_mode: DisplayMode::Single {
				column_count: 5,
			}
		}});

		true
	}else if href.contains("pixiv.net/en/users") {
		Timeout::new(3_000, || {
			let mount_point = gloo_utils::document().create_element("div").expect("to create empty div");
			mount_point.set_id("favviewer");

			let nav = gloo_utils::document()
				.get_elements_by_tag_name("nav").get_with_index(0).expect("couldn't find a nav");

			let navGrandParent = nav.parent_element().expect("couldn't find nav parent").parent_element().expect("couldn't find nav grandparent");
			navGrandParent.after_with_node_1(&mount_point);

			yew::start_app_with_props_in_element::<Model>(mount_point, yew::props! { ModelProps {
				favviewer: true,
				display_mode: DisplayMode::Single {
					column_count: 5,
				}
			}});
		}).forget();

		true
	}else {
		false
	}
}

pub fn page_info(ctx: &Context<Model>) -> Option<Box<dyn PageInfo>> {
	let href = web_sys::window()
		.map(|w| w.location())
		.and_then(|l| l.href().ok()).unwrap();
	if href.contains("pixiv.net/bookmark_new_illust") {
		Some(Box::new(FollowPageInfo::new(ctx.link().callback(|_| ModelMsg::ToggleFavViewer))) as Box<dyn PageInfo>)
	}else if href.contains("pixiv.net/en/users") {
		Some(Box::new(UserPageInfo::new(ctx.link().callback(|_| ModelMsg::ToggleFavViewer))) as Box<dyn PageInfo>)
	}else {
		None
	}
}
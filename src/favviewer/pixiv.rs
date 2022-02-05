use yew::prelude::*;
use yew_agent::Dispatched;
use std::collections::HashMap;
use gloo_timers::callback::Timeout;

use crate::favviewer::{FavViewerStyle, PageInfo, default_hidden_style};
use crate::{Model, Props as ModelProps, DisplayMode, EndpointAgent};
use crate::modals::settings::{SettingsAgent, Request as SettingsAgentRequest};
use crate::timeline::Container;
use crate::services::{
	endpoint_agent::{Request as EndpointRequest, TimelineCreationRequest},
	pixiv::endpoints::*,
	storages::get_or_init_favviewer_settings,
};

fn make_follow_activator(onclick: Callback<MouseEvent>) -> Html {
	let favviewer_button_mount = gloo_utils::document()
		.query_selector(".sc-s8zj3z-6.kstoDd")
		.expect("couldn't query activator mount point")
		.expect("couldn't find activator mount point");

	create_portal(html! {
		<a class="sc-d98f2c-0" {onclick}>
			<span class="sc-93qi7v-2 ibdURy">{"FavViewer"}</span>
		</a>
	}, favviewer_button_mount.into())
}

fn add_follow_timelines() {
	let location = web_sys::window().unwrap().location();
	let pathname = location.pathname().unwrap();
	let search = web_sys::UrlSearchParams::new_with_str(&location.search().unwrap()).unwrap();

	let r18 = pathname.contains("r18");
	let current_page = search.get("p")
		.and_then(|s| s.parse().ok())
		.unwrap_or(1);

	let mut endpoint_agent = EndpointAgent::dispatcher();
	endpoint_agent.send(EndpointRequest::BatchAddEndpoints(
		vec![(
			Box::new(move |id| {
				Box::new(FollowPageEndpoint::new(id))
			}), true, false
		), (
			Box::new(move |id| {
				Box::new(FollowAPIEndpoint::new(id, r18, current_page - 1))
			}), false, true
		)],
		TimelineCreationRequest::NameEndpoints("Pixiv".to_owned())
	));
}

fn make_user_activator(onclick: Callback<MouseEvent>) -> Html {
	let nav = gloo_utils::document()
		.get_elements_by_tag_name("nav").get_with_index(0).expect("couldn't find a nav");
	let activator_classes = nav.last_element_child().map(|c| c.class_name()).unwrap_or_default();

	let favviewer_button_mount = gloo_utils::document()
		.query_selector("nav")
		.expect("couldn't query activator mount point")
		.expect("couldn't find activator mount point");
	create_portal(html! {
		<a id="favvieweractivator" class={activator_classes} {onclick}>
			{"FavViewer"}
		</a>
	}, favviewer_button_mount.into())
}

fn add_user_timeline() {

}

pub fn setup(href: &str) -> bool {
	if href.contains("pixiv.net/bookmark_new_illust") {
		let mount_point = gloo_utils::document().create_element("div").expect("to create empty div");
		mount_point.class_list().add_1("favviewer").expect("adding favviewer class");

		gloo_utils::document()
			.query_selector("#root > div:last-child > div:nth-child(2)")
			.expect("can't get mount node for rendering")
			.expect("can't unwrap mount node")
			.append_with_node_1(&mount_point)
			.expect("can't append mount node");

		let document_head = gloo_utils::document().head().expect("head element to be present");
		let mut style_html = HashMap::new();
		style_html.insert(FavViewerStyle::Normal, create_portal(html! {
                <style>{"#root {width: 100%} #root > :nth-child(2), .sc-1nr368f-2.bGUtlw { height: 100%; } .sc-jgyytr-1 {display: none}"}</style>
			}, document_head.clone().into()
		));
		style_html.insert(FavViewerStyle::Hidden, create_portal(html! {
                <style>{format!("{} #root {{width: 100%}}", default_hidden_style())}</style>
			}, document_head.into()
		));

		let display_mode = get_or_init_favviewer_settings(DisplayMode::Single {
			container: Container::Masonry,
			column_count: 5,
		});
		let mut settings_agent = SettingsAgent::dispatcher();
		settings_agent.send(SettingsAgentRequest::InitFavViewerSettings(display_mode));

		yew::start_app_with_props_in_element::<Model>(mount_point, yew::props! { ModelProps {
			favviewer: true,
			display_mode,
			page_info: Some(PageInfo::Setup {
				style_html,
				initial_style: FavViewerStyle::Hidden,
				make_activator: make_follow_activator,
				add_timelines: add_follow_timelines,
			})
		}});

		true
	}else if href.contains("pixiv.net/en/users") {
		Timeout::new(3_000, || {
			let mount_point = gloo_utils::document().create_element("div").expect("to create empty div");
			mount_point.class_list().add_1("favviewer").expect("adding favviewer class");

			let nav = gloo_utils::document()
				.get_elements_by_tag_name("nav").get_with_index(0).expect("couldn't find a nav");

			let nav_grand_parent = nav.parent_element().expect("couldn't find nav parent").parent_element().expect("couldn't find nav grandparent");
			nav_grand_parent.after_with_node_1(&mount_point).expect("couldn't add mount_point");

			let document_head = gloo_utils::document().head().expect("head element to be present");
			let mut style_html = HashMap::new();
			style_html.insert(FavViewerStyle::Normal, create_portal(html! {
                <style>{".favviewer {width: 100%; height: 50%}#root {width: 100%} #root > :nth-child(2), .sc-1nr368f-2.bGUtlw { height: 100%; } .sc-jgyytr-1 {display: none}"}</style>
			}, document_head.clone().into()
			));
			style_html.insert(FavViewerStyle::Hidden, create_portal(html! {
                <style>{".favviewer {display: none;} #root {width: 100%} "}</style>
			}, document_head.into()
			));

			yew::start_app_with_props_in_element::<Model>(mount_point, yew::props! { ModelProps {
				favviewer: true,
				display_mode: DisplayMode::Single {
					container: Container::Masonry,
					column_count: 5,
				},
				page_info: Some(PageInfo::Setup {
					style_html,
					initial_style: FavViewerStyle::Hidden,
					make_activator: make_user_activator,
					add_timelines: add_user_timeline,
				})
			}});
		}).forget();

		true
	}else {
		false
	}
}
use std::collections::HashMap;
use yew::prelude::*;
use yew_agent::{Agent, Context, HandlerId, AgentLink};

pub enum Notification {
	Generic(String),
	Login(String, String),
}

pub struct NotificationAgent {
	link: AgentLink<Self>,
	notifications: HashMap<String, Notification>,
	timeline_container: Option<HandlerId>,
}

pub enum Msg {

}

pub enum Request {
	RegisterTimelineContainer,
	Notify(Option<String>, Notification),
}

pub enum Response {
	DrawNotifications(Vec<Html>),
}

impl Agent for NotificationAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			notifications: HashMap::new(),
			timeline_container: None,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {

		}
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			//TODO use portal and Rc?
			Request::RegisterTimelineContainer => self.timeline_container = Some(id),
			Request::Notify(id, notif) => {
				let id = id.unwrap_or_else(|| {
					let i = 0;
					let mut id;
					loop {
						id = format!("Generated{}", i);
						if !self.notifications.contains_key(&id) { break; }
					}

					id
				});

				self.notifications.insert(id, notif);
				if let Some(handler) = self.timeline_container {
					self.link.respond(handler, Response::DrawNotifications(self.notifications.values().map(|n| self.view_notification(n)).collect()))
				}
			}
		}
	}
}

impl NotificationAgent {
	fn view_notification(&self, notif: &Notification) -> Html {
		let (classes, content) = match notif {
			Notification::Generic(text) => (None, html! {
				{ text }
			}),
			Notification::Login(service, url) => (Some("is-warning"), html! {
				<>
					<div class="block">
						<span>{ format!("Login to {}?", service) }</span>
					</div>
					<a class="button" href={url.clone()}>
						{ "Login" }
					</a>
				</>
			})
		};

		html! {
			<div class={classes!("notification", classes)}>
			  <button class="delete"></button>
			  { content }
			</div>
		}
	}
}
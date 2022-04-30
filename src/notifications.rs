use std::collections::HashMap;
use yew::prelude::*;
use yew_agent::{Agent, Context, HandlerId, AgentLink};

use crate::error::Error;

pub enum Notification {
	Generic(String),
	Login(String, String),
	Error(Error),
}

pub struct NotificationAgent {
	link: AgentLink<Self>,
	notifications: HashMap<String, Notification>,
	model: Option<HandlerId>,
}

pub enum NotificationMsg {
	Delete(String),
}

pub enum NotificationRequest {
	RegisterTimelineContainer,
	Notify(Option<String>, Notification),
}

pub enum NotificationResponse {
	DrawNotifications(Vec<Html>),
}

type Msg = NotificationMsg;
type Request = NotificationRequest;
type Response = NotificationResponse;

impl Agent for NotificationAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			notifications: HashMap::new(),
			model: None,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::Delete(id) => {
				let _ = self.notifications.remove(&id);
				self.draw_notifications();
			}
		}
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			//TODO use portal and Rc?
			Request::RegisterTimelineContainer => self.model = Some(id),
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
				self.draw_notifications();
			}
		}
	}
}

impl NotificationAgent {
	fn view_notification(&self, id: String, notif: &Notification) -> Html {
		let (classes, content) = match notif {
			Notification::Generic(text) => (None, html! {
				<div class="block">
					<span>{ text }</span>
				</div>
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
			}),
			Notification::Error(error) => (Some("is-danger"), html! {
				<div class="block">
					<span>{ error.message() }</span>
				</div>
			}),
		};

		html! {
			<div class={classes!("notification", classes)}>
			  <button class="delete" onclick={self.link.callback(move |_| Msg::Delete(id.clone()))}/>
			  { content }
			</div>
		}
	}

	fn draw_notifications(&self) {
		if let Some(handler) = self.model {
			self.link.respond(handler, Response::DrawNotifications(self.notifications.iter().map(|(id, n)| self.view_notification(id.to_string(), n)).collect()))
		}
	}
}
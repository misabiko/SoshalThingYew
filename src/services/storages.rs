/*use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use web_sys::Storage;

use crate::articles::ArticleData;

pub enum StorageType {
	Session,
}

pub struct StorageAgent {
	link: AgentLink<Self>,
	global_key: &'static str,
	session_storage: Storage,
}

pub enum Request {
	GetArticles(String, StorageType),
	SetArticle(String, StorageType, String, String, serde_json::Value),
}

pub enum Response {
}

impl Agent for StorageAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let session_storage = web_sys::window().expect("no global window")
			.session_storage
			.expect("couldn't open session storage")
			.expect("couldn't find session storage");

		let global_key = "SoshalThingYew";
		if session_storage.get_item(service).expect("couldn't access session storage").is_none() {
			session_storage.set_item(global_key, json!.to_string());
		}

		Self {
			link,
			global_key,
			session_storage,
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::GetArticle(service, storage) => {
				if let Some(content) = self.session_storage.get_item(service).expect("couldn't access session storage") {

				}
			}
			Request::SetArticle(service, storage, article_id, key, value) => {
				if let Some(content) = self.session_storage.get_item(service).expect("couldn't access session storage") {

				}
			}
		};
	}
}

impl StorageAgent {
	fn get_service(&self, service: String) -> serde_json::Value {
		if let Some(content) = self.session_storage.get_item(service).expect("couldn't access session storage") {
			let service = serde_json::Value::from_str(content)
		}
	}
}*/
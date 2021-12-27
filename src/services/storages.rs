use gloo_storage::Storage;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SessionStorageService {
	pub articles_marked_as_read: HashSet<String>,
	pub cached_articles: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SoshalSessionStorage {
	pub services: HashMap<String, SessionStorageService>,
}

pub fn get_service_session(service: &str) -> SessionStorageService {
	let storage: SoshalSessionStorage = gloo_storage::SessionStorage::get("SoshalThingYew").unwrap_or_default();
	storage.services.get(service).cloned().unwrap_or_default()
}

/*use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::rc::Weak;
use std::cell::RefCell;
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
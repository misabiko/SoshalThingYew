use gloo_storage::Storage;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::DisplayMode;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SessionStorageService {
	pub articles_marked_as_read: HashSet<String>,
	pub cached_articles: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SoshalSessionStorage {
	pub services: HashMap<String, SessionStorageService>,
}

pub fn get_service_storage(service: &str) -> ServiceStorage {
	let local: SoshalLocalStorage = gloo_storage::LocalStorage::get("SoshalThingYew").unwrap_or_default();
	let session: SoshalSessionStorage = gloo_storage::SessionStorage::get("SoshalThingYew").unwrap_or_default();

	ServiceStorage {
		local: local.services.get(service).cloned().unwrap_or_default(),
		session: session.services.get(service).cloned().unwrap_or_default(),
	}
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LocalStorageService {
	pub hidden_articles: HashSet<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SoshalLocalStorage {
	pub services: HashMap<String, LocalStorageService>,
	pub display_mode: DisplayMode,
}

pub struct ServiceStorage {
	pub local: LocalStorageService,
	pub session: SessionStorageService,
}

pub fn cache_articles(service_name: &str, articles: HashMap<String, serde_json::Value>) {
	let session_storage: SoshalSessionStorage = match gloo_storage::SessionStorage::get("SoshalThingYew") {
		Ok(storage) => {
			let mut session_storage: SoshalSessionStorage = storage;
			let service = match session_storage.services.get_mut(service_name) {
				Some(service) => service,
				None => {
					let service = SessionStorageService {
						articles_marked_as_read: HashSet::new(),
						cached_articles: HashMap::new(),
					};
					session_storage.services.insert(service_name.to_owned(), service);
					session_storage.services.get_mut(service_name).unwrap()
				}
			};

			for (id, article) in &articles {
				let _ = service.cached_articles.insert(id.to_string(), article.clone());
			}

			session_storage
		}
		Err(_err) => {
			SoshalSessionStorage {
				services: HashMap::from([
					(service_name.to_owned(), SessionStorageService {
						articles_marked_as_read: HashSet::new(),
						cached_articles: articles.iter()
							.map(|(id, a)| (id.to_string(), a.clone()))
							.collect(),
					})
				])
			}
		}
	};

	gloo_storage::SessionStorage::set("SoshalThingYew", &session_storage)
		.expect("couldn't write session storage");
}

pub fn mark_article_as_read(service_name: &str, id: String, value: bool) {
	let session_storage: SoshalSessionStorage = match gloo_storage::SessionStorage::get("SoshalThingYew") {
		Ok(storage) => {
			let mut session_storage: SoshalSessionStorage = storage;
			(match session_storage.services.get_mut(service_name) {
				Some(service) => Some(service),
				None => {
					let service = SessionStorageService {
						articles_marked_as_read: HashSet::new(),
						cached_articles: HashMap::new(),
					};
					session_storage.services.insert(service_name.to_owned(), service);
					session_storage.services.get_mut(service_name)
				}
			})
				.map(|s| &mut s.articles_marked_as_read).
				map(|cached| if value {
					cached.insert(id);
				}else {
					cached.remove(&id);
				});

			session_storage
		},
		Err(_err) => {
			SoshalSessionStorage {
				services: HashMap::from([
					(service_name.to_owned(), SessionStorageService {
						articles_marked_as_read: match value {
							true => {
								let mut set = HashSet::new();
								set.insert(id);
								set
							},
							false => HashSet::new(),
						},
						cached_articles: HashMap::new(),
					})
				])
			}
		}
	};

	gloo_storage::SessionStorage::set("SoshalThingYew", &session_storage)
		.expect("couldn't write session storage");
}

pub fn hide_article(service_name: &str, id: String, value: bool) {
	let session_storage: SoshalLocalStorage = match gloo_storage::LocalStorage::get("SoshalThingYew") {
		Ok(storage) => {
			let mut session_storage: SoshalLocalStorage = storage;
			(match session_storage.services.get_mut(service_name) {
				Some(service) => Some(service),
				None => {
					let service = LocalStorageService {
						hidden_articles: HashSet::new(),
					};
					session_storage.services.insert(service_name.to_owned(), service);
					session_storage.services.get_mut(service_name)
				}
			})
				.map(|s| &mut s.hidden_articles).
				map(|cached| if value {
					cached.insert(id);
				}else {
					cached.remove(&id);
				});

			session_storage
		},
		Err(_err) => {
			SoshalLocalStorage {
				services: HashMap::from([
					(service_name.to_owned(), LocalStorageService {
						hidden_articles: match value {
							true => {
								let mut set = HashSet::new();
								set.insert(id);
								set
							},
							false => HashSet::new(),
						},
					})
				]),
				display_mode: DisplayMode::Default,
			}
		}
	};

	gloo_storage::LocalStorage::set("SoshalThingYew", &session_storage)
		.expect("couldn't write session storage");
}
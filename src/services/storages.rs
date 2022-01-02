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
					cached.insert(id.clone());
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
								set.insert(id.clone());
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
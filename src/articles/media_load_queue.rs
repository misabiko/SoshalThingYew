use std::collections::{HashSet, HashMap, VecDeque};
use yew_agent::{Agent, AgentLink, HandlerId, Context};

static MAX_LOADING: usize = 5;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MediaLoadState {
	NotLoaded,
	Loading,
	Loaded,
}

pub struct MediaLoadAgent {
	link: AgentLink<Self>,
	load_queue: VecDeque<(String, usize, HashSet<HandlerId>)>,
	currently_loading: HashMap<(String, usize), HashSet<HandlerId>>,
}

pub enum Request {
	QueueMedia(String, usize),
	LoadMedia(String, usize),
	MediaLoaded(String, usize),
}

pub enum Response {
	UpdateState(usize, MediaLoadState),
}

impl Agent for MediaLoadAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			load_queue: VecDeque::new(),
			currently_loading: HashMap::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::QueueMedia(article_id, media_index) => {
				if self.currently_loading.len() >= MAX_LOADING {
					let existing_queued = self.load_queue.iter_mut()
						.find_map(|(a_id, m_index, ids)|
							if *a_id == article_id && *m_index == media_index {
								Some(ids)
							}else {
								None
							}
						);
					if let Some(ids) = existing_queued {
						ids.insert(id);
					}else {
						self.load_queue.push_back((article_id, media_index, HashSet::from([id])));
					}
				}else {
					self.loading(article_id, media_index, vec![&id]);
				}
			}
			Request::LoadMedia(article_id, media_index) => {
				let position = self.load_queue.iter()
					.position(|(a_id, m_index, _)| *a_id == article_id && *m_index == media_index);
				if let Some(index) = position {
					let (_, _, mut ids) = self.load_queue.remove(index).unwrap();
					ids.insert(id);
					self.loading(article_id, media_index, ids.iter().collect());
				}else {
					self.loading(article_id, media_index, vec![&id]);
				}
			}
			Request::MediaLoaded(article_id, media_index) => {
				log::debug!("Done loading {}/{}", &article_id, &media_index);
				match self.currently_loading.remove(&(article_id.clone(), media_index)) {
					Some(ids) => for id_i in ids {
						if id_i != id {
							self.link.respond(id_i, Response::UpdateState(media_index, MediaLoadState::Loaded));
						}
					},
					None => log::warn!("Media {}/{} wasn't in currently_loading", &article_id, &media_index),
				}

				if self.currently_loading.len() < MAX_LOADING {
					if let Some((next_article_id, next_media_index, mut next_ids)) = self.load_queue.pop_front() {
						next_ids.insert(id);
						self.loading(next_article_id, next_media_index, next_ids.iter().collect());
					}
				}
			}
		}

		log::trace!("Load queue: {}, Currently loading: {:?}", self.load_queue.len(), self.currently_loading.keys().map(|(a, _)| a));
	}

	fn disconnected(&mut self, id: HandlerId) {
		let mut to_remove: Option<usize> = None;
		for (i, (_, _, ids)) in self.load_queue.iter_mut().enumerate() {
			if ids.remove(&id) && ids.is_empty() {
				to_remove = Some(i);
				break;
			}
		}
		if let Some(to_remove) = to_remove {
			self.load_queue.remove(to_remove);
		}

		let mut to_remove: Option<(String, usize)> = None;
		for (key, ids) in self.currently_loading.iter_mut() {
			if ids.remove(&id) && ids.is_empty() {
				to_remove = Some(key.clone());
				break;
			}
		}
		if let Some(to_remove) = to_remove {
			self.currently_loading.remove(&to_remove);
		}
	}
}

impl MediaLoadAgent {
	fn loading(&mut self, article_id: String, media_index: usize, ids: Vec<&HandlerId>) {
		log::debug!("Loading {}/{}", &article_id, media_index);

		let ids_c = ids.clone();
		self.currently_loading.entry((article_id.clone(), media_index))
			.and_modify(|curr_ids| {
				log::warn!("Already loading {}/{}", &article_id, &media_index);
				curr_ids.extend(ids_c);
			})
			.or_insert_with(|| ids.iter().cloned().cloned().collect());	//...

		for id_i in ids {
			self.link.respond(*id_i, Response::UpdateState(media_index, MediaLoadState::Loading));
		}
	}
}
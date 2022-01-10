use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use yew_agent::{Agent, AgentLink, Context, Dispatcher, Dispatched, HandlerId};

mod article;
mod endpoints;

use article::YoutubeArticleData;
use crate::articles::ArticleData;
use crate::services::endpoint_agent::{EndpointAgent, EndpointConstructor, EndpointId, Request as EndpointRequest};
use crate::services::article_actions::{ArticleActionsAgent, ServiceActions, Request as ArticleActionsRequest};
use crate::services::endpoint_agent::EndpointConstructors;
use crate::services::RefreshTime;
use crate::services::youtube::endpoints::HardCodedEndpoint;

//TODO derive(Service)?
//TODO type<A> ServiceArticles = HashMap<A::Id, Rc<RefCell<A>>>
pub struct YoutubeAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
	articles: HashMap<String, Rc<RefCell<YoutubeArticleData>>>,
}

pub enum Msg {}

pub enum Request {
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<YoutubeArticleData>>>),
}

pub enum Response {}

impl Agent for YoutubeAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			"Youtube".to_owned(),
			EndpointConstructors {
				endpoint_types: vec![
					EndpointConstructor {
						name: "Hardcoded",
						param_template: vec![],
						callback: Rc::new(|id, _params| Box::new(HardCodedEndpoint::new(id))),
					},
				],
				user_endpoint: None,
			}));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init("Youtube", ServiceActions {
			like: None,
			repost: None,
			fetch_data: None,
		}));

		Self {
			endpoint_agent,
			link,
			actions_agent,
			articles: HashMap::new(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {}
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::AddArticles(refresh_time, endpoint_id, articles) => {
				let mut updated_articles = Vec::new();
				for article in articles.into_iter() {
					let borrow = article.borrow();
					let article = self.articles.entry(borrow.id.clone())
						.and_modify(|a| a.borrow_mut().update(&borrow))
						.or_insert_with(|| article.clone()).clone();

					updated_articles.push(article as Rc<RefCell<dyn ArticleData>>);
				}
				self.endpoint_agent.send(EndpointRequest::AddArticles(
					refresh_time,
					endpoint_id,
					updated_articles,
				));

				//self.check_unfetched_articles();
			}
		}
	}
}
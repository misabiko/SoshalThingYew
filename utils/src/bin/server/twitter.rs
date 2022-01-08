use actix_web::{
	web::{Data, Path, Query},
	web, get, HttpResponse, http::header};
use actix_identity::Identity;
use egg_mode::list::ListID;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use actix_web::dev::{HttpServiceFactory, ServiceRequest, ServiceResponse, Service, Transform};
use actix_web::guard::fn_guard;

use crate::{Error, Result, State};

#[derive(Deserialize)]
struct Credentials {
	consumer_key: String,
	consumer_secret: String,
}

#[derive(Debug)]
pub struct TwitterState {
	con_token: egg_mode::KeyPair,
	req_token: Mutex<Option<egg_mode::KeyPair>>,
	bearer_token: egg_mode::Token,
	tokens: Mutex<HashMap<u64, egg_mode::Token>>,
}

pub async fn state() -> Result<TwitterState> {
	let credentials = match (std::env::var("consumer_key"), std::env::var("consumer_secret")) {
		(Ok(consumer_key), Ok(consumer_secret)) => Ok(Credentials { consumer_key, consumer_secret }),
		(Ok(_), Err(err)) => {
			log::warn!("Found consumer_key environment variable, but no secret.\n{:?}", err);
			Err(())
		}
		(Err(err), Ok(_)) => {
			log::warn!("Found consumer_secret environment variable, but no key.\n{:?}", err);
			Err(())
		}
		(Err(_), Err(_)) => Err(()),
	}
	.or_else(|_|
		std::fs::read_to_string("credentials.json")
			.map_err(Error::from)
			.and_then(|c| serde_json::from_str(&c).map_err(Error::from))
			.map_err(|err| Error::from(format!("Please add credentials.json or set environment variables\n{:#?}", err)))
	);

	let r = match credentials {
		Ok(credentials) => {
			let con_token = egg_mode::KeyPair::new(credentials.consumer_key, credentials.consumer_secret);
			Ok(TwitterState {
				req_token: Mutex::new(None),
				bearer_token: egg_mode::auth::bearer_token(&con_token).await?,
				tokens: Mutex::new(HashMap::new()),
				con_token,
			})
		},
		Err(err) => Err(err)
	};

	println!("twitter::state() {}", r.is_ok());
	r
}

/*pub struct TwitterService;

pub struct CheckTwitterDataTransform;

impl<S: Service<ServiceRequest>> Transform<S, ServiceRequest> for CheckTwitterDataTransform {
	type Response = ServiceResponse;
	type Error = actix_web::Error;
	type Transform = CheckTwitterDataMiddleware<S>;
	type InitError = ();
	type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(CheckTwitterDataMiddleware { service }))
	}
}

pub struct CheckTwitterDataMiddleware<S> {
	service: S
}

impl<S: Service<ServiceRequest>> Service<ServiceRequest> for CheckTwitterDataMiddleware<S> {
	type Response = ServiceResponse;
	type Error = actix_web::Error;
	type Future = Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>>>>;

	fn poll_ready(&self, _ctx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}

	fn call(&self, req: ServiceRequest) -> Self::Future {
		//let mut extensions = req.extensions_mut();
		//println!("contains State? {}", extensions.contains::<State>());
		//let data = extensions.get::<State>().unwrap();
		//let has_data = data.twitter.is_ok();

		let fut = self.service.call(req);
		Box::pin(async {
			//if has_data {
				let res = fut.await?;
				Ok(res)
			//}else {
			//	Err(actix_web::Error::from(&HttpResponse::InternalServerError().finish()).as_response_error())
			//}
		})
		//ready(Err(actix_web::Error::from(Error::from("No twitter data".to_owned()))))
	}
}*/

pub fn service() -> impl HttpServiceFactory {
	web::scope("/twitter")
		//.wrap(CheckTwitterDataTransform)
		.wrap_fn(|req, service| {
			println!("State {}", req.app_data::<Data<State>>().is_some());
			println!("State {}", req.req_data().contains::<Data<State>>());
			let has_data = req.app_data::<Data<State>>().map(|s| s.twitter.is_some()).unwrap_or(false);
			let fut = service.call(req);
			println!("has_data {}", has_data);

			if has_data {
				fut
			}else {
				//TODO Properly send error response
				Box::pin(ready(Err(actix_web::Error::from(Error::from("No twitter data".to_owned())))))
			}
		})
		.service(status)
		.service(like)
		.service(unlike)
		.service(retweet)
		.service(unretweet)
		.service(user_timeline)
		.service(home_timeline)
		.service(list)
		.service(twitter_login)
		.service(twitter_login_callback)
	/*let error_message = err.to_string();
			web::scope("/twitter")
				.default_service(web::route().to(move || HttpResponse::InternalServerError().body(error_message.clone())))*/
}

fn get_token<'a>(id: &'a Identity, tokens: &'a HashMap<u64, egg_mode::Token>, bearer_token: &'a egg_mode::Token) -> &'a egg_mode::Token {
	match id.identity() {
		Some(user_id_str) => match user_id_str.parse::<u64>() {
			Ok(user_id) => match &tokens.get(&user_id) {
				Some(access_token) => {
					log::info!("Welcome! {}", &user_id);
					*access_token
				}
				None => {
					log::info!("Couldn't find token for {}", &user_id);
					bearer_token
				}
			}
			Err(err) => {
				log::info!("{}", err);
				bearer_token
			}
		}
		None => {
			log::info!("Welcome Anonymous!");
			bearer_token
		}
	}
}

fn get_access_token<'a>(id: &'a Identity, tokens: &'a HashMap<u64, egg_mode::Token>) -> Option<&'a egg_mode::Token> {
	match id.identity() {
		Some(user_id_str) => match user_id_str.parse::<u64>() {
			Ok(user_id) => match &tokens.get(&user_id) {
				Some(access_token) => {
					log::info!("Welcome! {}", &user_id);
					Some(*access_token)
				}
				None => {
					log::info!("Couldn't find token for {}", &user_id);
					None
				}
			}
			Err(err) => {
				log::info!("{}", err);
				None
			}
		}
		None => {
			log::info!("Welcome Anonymous!");
			None
		}
	}
}

fn tweet_to_http_response(feed: egg_mode::Response<impl Serialize>) -> HttpResponse {
	HttpResponse::Ok()
		.append_header(("x-rate-limit-limit".to_owned(), feed.rate_limit_status.limit.clone()))
		.append_header(("x-rate-limit-remaining".to_owned(), feed.rate_limit_status.remaining.clone()))
		.append_header(("x-rate-limit-reset".to_owned(), feed.rate_limit_status.reset.clone()))
		.json(&feed.response)
}

/*fn lock_tokens(data: &Data<State>) -> Result<MutexGuard<HashMap<u64, Token>>> {
	data.twitter.map(|d| d.tokens.lock().expect("locking token mutex"))
}*/

#[derive(Deserialize)]
struct TimelineQuery {
	replies: Option<bool>,
	rts: Option<bool>,
	count: Option<i32>,
	min_id: Option<u64>,
	max_id: Option<u64>,
}

#[get("list/{username}/{slug}")]
async fn list(id: Identity, path: Path<(String, String)>, query: Query<TimelineQuery>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token = get_token(&id, tokens, &data.twitter.as_ref().unwrap().bearer_token);

	let (username, slug) = path.into_inner();
	let timeline = egg_mode::list::statuses(ListID::from_slug(username, slug), query.rts.unwrap_or_default(), token)
		.with_page_size(query.count.unwrap_or(200));

	let feed = timeline.call(query.min_id, query.max_id).await?;

	Ok(tweet_to_http_response(feed))
}

#[get("status/{id}")]
async fn status(id: Identity, tweet_id: Path<u64>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token = get_token(&id, tokens, &data.twitter.as_ref().unwrap().bearer_token);

	let r = egg_mode::tweet::show(tweet_id.into_inner(), token).await?;

	Ok(tweet_to_http_response(r))
}

#[get("user/{username}")]
async fn user_timeline(id: Identity, username: Path<String>, query: Query<TimelineQuery>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token = get_token(&id, tokens, &data.twitter.as_ref().unwrap().bearer_token);

	let timeline = egg_mode::tweet::user_timeline(
		egg_mode::user::UserID::ScreenName(username.into_inner().into()),
		query.replies.unwrap_or(true),
		query.rts.unwrap_or(true),
		token
	)
		.with_page_size(query.count.unwrap_or(200));

	let feed = timeline.call(query.min_id, query.max_id).await?;

	Ok(tweet_to_http_response(feed))
}

#[get("home")]
async fn home_timeline(id: Identity, query: Query<TimelineQuery>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let timeline = egg_mode::tweet::home_timeline(token)
			.with_page_size(query.count.unwrap_or(200));

		let feed = timeline.call(query.min_id, query.max_id).await?;

		Ok(tweet_to_http_response(feed))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("like/{id}")]
async fn like(id: Identity, tweet_id: Path<u64>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::like(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("unlike/{id}")]
async fn unlike(id: Identity, tweet_id: Path<u64>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::unlike(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("retweet/{id}")]
async fn retweet(id: Identity, tweet_id: Path<u64>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::retweet(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("unretweet/{id}")]
async fn unretweet(id: Identity, tweet_id: Path<u64>, data: Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::unretweet(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("login")]
async fn twitter_login(data: Data<State>) -> Result<HttpResponse> {
	let new_req_token = egg_mode::auth::request_token(&data.twitter.as_ref().unwrap().con_token, "http://localhost:8080/proxy/twitter/callback").await?;
	*data.twitter.as_ref().unwrap().req_token.lock().expect("locking token mutex") = Some(new_req_token.clone());

	let authorize_url = egg_mode::auth::authorize_url(&new_req_token);
	log::info!("Redirecting to {}", &authorize_url);
	Ok(HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, authorize_url))
		.finish())
}

#[derive(Deserialize)]
struct LoginCallbackQuery {
	//oauth_token: String,
	oauth_verifier: String,
}

#[get("callback")]
async fn twitter_login_callback(id: Identity, query: Query<LoginCallbackQuery>, data: Data<State>) -> Result<HttpResponse> {
	if let Some(req_token) = &*data.twitter.as_ref().unwrap().req_token.lock().expect("locking token mutex") {
		let (access_token, user_id, _username) = egg_mode::auth::access_token(
			data.twitter.as_ref().unwrap().con_token.clone(),
			&req_token,
			query.oauth_verifier.clone(),
		).await?;

		data.twitter.as_ref().unwrap().tokens.lock().expect("locking token mutex").insert(user_id.clone(), access_token);
		log::info!("Remembering id {}", &user_id);
		id.remember(user_id.to_string());
	}

	Ok(HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, "http://localhost:8080/"))
		.finish())
}
use actix_web::{get, web, App, HttpResponse, HttpServer, middleware::Logger, web::Path, http::header};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use egg_mode::list::ListID;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
	Actix(actix_web::Error),
	EggMode(egg_mode::error::Error),
	IO(std::io::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Actix(err) => err.fmt(f),
			Error::EggMode(err) => err.fmt(f),
			Error::IO(err) => err.fmt(f),
		}
	}
}

impl From<actix_web::Error> for Error {
	fn from(err: actix_web::Error) -> Self {
		Error::Actix(err)
	}
}

impl From<egg_mode::error::Error> for Error {
	fn from(err: egg_mode::error::Error) -> Self {
		Error::EggMode(err)
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::IO(err)
	}
}

impl actix_web::ResponseError for Error {}

#[derive(Deserialize)]
struct Credentials {
	consumer_key: String,
	consumer_secret: String,
}

struct State {
	con_token: egg_mode::KeyPair,
	req_token: Mutex<Option<egg_mode::KeyPair>>,
	bearer_token: egg_mode::Token,
	tokens: Mutex<HashMap<u64, egg_mode::Token>>,
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

#[derive(Deserialize)]
struct TimelineQuery {
	replies: Option<bool>,
	rts: Option<bool>,
	count: Option<i32>,
	min_id: Option<u64>,
	max_id: Option<u64>,
}

#[get("twitter/list/{username}/{slug}")]
async fn list(id: Identity, path: Path<(String, String)>, query: web::Query<TimelineQuery>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token = get_token(&id, tokens, &data.bearer_token);

	let (username, slug) = path.into_inner();
	let timeline = egg_mode::list::statuses(ListID::from_slug(username, slug), query.rts.unwrap_or_default(), token)
		.with_page_size(query.count.unwrap_or(200));

	let feed = timeline.call(query.min_id, query.max_id).await?;

	Ok(tweet_to_http_response(feed))
}

#[get("/twitter/status/{id}")]
async fn status(id: Identity, tweet_id: Path<u64>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token = get_token(&id, tokens, &data.bearer_token);

	let r = egg_mode::tweet::show(tweet_id.into_inner(), token).await?;

	Ok(tweet_to_http_response(r))
}

#[get("/twitter/user/{username}")]
async fn user_timeline(id: Identity, username: Path<String>, query: web::Query<TimelineQuery>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token = get_token(&id, tokens, &data.bearer_token);

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

#[get("/twitter/home")]
async fn home_timeline(id: Identity, query: web::Query<TimelineQuery>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
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

#[get("/twitter/like/{id}")]
async fn like(id: Identity, tweet_id: Path<u64>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::like(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("/twitter/unlike/{id}")]
async fn unlike(id: Identity, tweet_id: Path<u64>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::unlike(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("/twitter/retweet/{id}")]
async fn retweet(id: Identity, tweet_id: Path<u64>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::retweet(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("/twitter/unretweet/{id}")]
async fn unretweet(id: Identity, tweet_id: Path<u64>, data: web::Data<State>) -> Result<HttpResponse> {
	let tokens = &*data.tokens.lock().expect("locking token mutex");
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let r = egg_mode::tweet::unretweet(tweet_id.into_inner(), token).await?;

		Ok(tweet_to_http_response(r))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("/twitter/login")]
async fn twitter_login(data: web::Data<State>) -> Result<HttpResponse> {
	let new_req_token = egg_mode::auth::request_token(&data.con_token, "http://localhost:8080/proxy/twitter/callback").await?;
	*data.req_token.lock().expect("locking token mutex") = Some(new_req_token.clone());

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

#[get("/twitter/callback")]
async fn twitter_login_callback(id: Identity, query: web::Query<LoginCallbackQuery>, data: web::Data<State>) -> Result<HttpResponse> {
	if let Some(req_token) = &*data.req_token.lock().expect("locking token mutex") {
		let (access_token, user_id, _username) = egg_mode::auth::access_token(
			data.con_token.clone(),
			&req_token,
			query.oauth_verifier.clone(),
		).await?;

		data.tokens.lock().expect("locking token mutex").insert(user_id.clone(), access_token);
		log::info!("Remembering id {}", &user_id);
		id.remember(user_id.to_string());
	}

	Ok(HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, "http://localhost:8080/"))
		.finish())
}

#[derive(Serialize)]
struct AuthInfo {
	twitter: Option<String>,
}

#[get("/auth_info")]
async fn auth_info(id: Identity) -> HttpResponse {
	HttpResponse::Ok().json(AuthInfo {
		twitter: id.identity(),
	})
}

#[actix_web::main]
async fn main() -> Result<()> {
	let credentials = match (std::env::var("consumer_key"), std::env::var("consumer_secret")) {
		(Ok(_), Err(err)) => {
			log::info!("Found consumer_key environment variable, but no secret.\n{:?}", err);
			None
		}
		(Err(err), Ok(_)) => {
			log::info!("Found consumer_secret environment variable, but no key.\n{:?}", err);
			None
		}
		(Ok(consumer_key), Ok(consumer_secret)) => Some(Credentials { consumer_key, consumer_secret }),
		(Err(_), Err(_)) => None,
	};

	//TODO Cleaner "Please add credentials.json or set environement variable" message then exit
	let credentials = credentials.unwrap_or_else(|| {
		let c = std::fs::read_to_string("credentials.json").expect("Couldn't find credentials.json");
		serde_json::from_str(&c).expect("Couldn't parse credentials.json")
	});

	let con_token = egg_mode::KeyPair::new(credentials.consumer_key, credentials.consumer_secret);
	let data = web::Data::new(State {
		req_token: Mutex::new(None),
		bearer_token: egg_mode::auth::bearer_token(&con_token).await?,
		tokens: Mutex::new(HashMap::new()),
		con_token,
	});

	std::env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	//TODO Use cookie key
	//TODO Set secure to true when HTTPS
	HttpServer::new(move || {
		App::new()
			.wrap(IdentityService::new(CookieIdentityPolicy::new(&[0; 32])
				.secure(false)))
			.wrap(Logger::default())
			.app_data(data.clone())
			.service(
				web::scope("/proxy")
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
					.service(auth_info)
			)
			.service(actix_files::Files::new("/", "./dist").index_file("index.html"))
	})
	.bind(format!("127.0.0.1:{}", if cfg!(debug_assertions) { 3000 } else { 8080 }))?
	.run()
	.await
	.map_err(|err| err.into())
}
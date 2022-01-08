use actix_web::{get, web, App, HttpResponse, HttpServer, middleware::Logger};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

mod twitter;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
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

pub struct State {
	con_token: egg_mode::KeyPair,
	req_token: Mutex<Option<egg_mode::KeyPair>>,
	bearer_token: egg_mode::Token,
	tokens: Mutex<HashMap<u64, egg_mode::Token>>,
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

	std::env::set_var("RUST_LOG", "actix_web=debug");
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
					.service(twitter::service())
					.service(auth_info)
			)
			.service(actix_files::Files::new("/", "./dist").index_file("index.html"))
	})
	.bind(format!("127.0.0.1:{}", if cfg!(debug_assertions) { 3000 } else { 8080 }))?
	.run()
	.await
	.map_err(|err| err.into())
}
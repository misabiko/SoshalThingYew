use actix_web::{get, web, App, HttpResponse, HttpServer, middleware::Logger, HttpRequest};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use actix_files::NamedFile;
use rand::Rng;

mod twitter;
use crate::twitter::TwitterState;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Text(String),
	Actix(actix_web::Error),
	EggMode(egg_mode::error::Error),
	IO(std::io::Error),
	Serde(serde_json::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Text(str) => f.write_str(&str),
			Error::Actix(err) => err.fmt(f),
			Error::EggMode(err) => err.fmt(f),
			Error::IO(err) => err.fmt(f),
			Error::Serde(err) => err.fmt(f),
		}
	}
}

impl From<String> for Error {
	fn from(err: String) -> Self {
		Error::Text(err)
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

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Self {
		Error::Serde(err)
	}
}

impl actix_web::ResponseError for Error {}

#[derive(Debug)]
pub struct State {
	pub twitter: Option<TwitterState>,
}

#[derive(Serialize)]
struct AuthInfo {
	pub twitter: Option<String>,
}

#[get("/auth_info")]
async fn auth_info(id: Identity) -> HttpResponse {
	HttpResponse::Ok().json(AuthInfo {
		twitter: id.identity(),
	})
}

async fn index(_req: HttpRequest) -> Result<NamedFile> {
	Ok(NamedFile::open("index.html")?)
}

#[actix_web::main]
async fn main() -> Result<()> {
	env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

	let data = web::Data::new(State {
		twitter: twitter::state().await.ok(),
	});

	let cookie_key = rand::thread_rng().gen::<[u8; 32]>();

	HttpServer::new(move || {
		App::new()
			.wrap(IdentityService::new(CookieIdentityPolicy::new(&cookie_key).secure(true)))
			.wrap(Logger::new(r#"%a %t "%r" %s "%{Referer}i" %T ms"#))
			.app_data(data.clone())
			.service(
				web::scope("/proxy")
					.service(twitter::service())
					.service(auth_info)
			)
			//TODO Fix /twitter/status/{id} shortcut
			//.route("/twitter/{_:.*}", web::to(index))
			.service(actix_files::Files::new("/", "./dist").index_file("index.html"))
	})
	.bind("127.0.0.1:8080")?
	.run()
	.await
	.map_err(|err| err.into())
}
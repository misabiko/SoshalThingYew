use actix_web::{get, web, App, HttpResponse, HttpServer, middleware::Logger, web::Path, Responder, http::header};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use egg_mode::list::ListID;
use serde::Deserialize;
use std::sync::Mutex;
use actix_files::NamedFile;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
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
					println!("Welcome! {}", &user_id);
					*access_token
				}
				None => {
					println!("Couldn't find token for {}", &user_id);
					bearer_token
				}
			}
			Err(err) => {
				println!("{:?}", err);
				bearer_token
			}
		}
		None => {
			println!("Welcome Anonymous!");
			bearer_token
		}
	}
}

fn get_access_token<'a>(id: &'a Identity, tokens: &'a HashMap<u64, egg_mode::Token>) -> Option<&'a egg_mode::Token> {
	match id.identity() {
		Some(user_id_str) => match user_id_str.parse::<u64>() {
			Ok(user_id) => match &tokens.get(&user_id) {
				Some(access_token) => {
					println!("Welcome! {}", &user_id);
					Some(*access_token)
				}
				None => {
					println!("Couldn't find token for {}", &user_id);
					None
				}
			}
			Err(err) => {
				println!("{:?}", err);
				None
			}
		}
		None => {
			println!("Welcome Anonymous!");
			None
		}
	}
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
async fn list(id: Identity, path: Path<(String, String)>, query: web::Query<TimelineQuery>, data: web::Data<State>) -> HttpResponse {
	let tokens = &*data.tokens.lock().unwrap();
	let token = get_token(&id, tokens, &data.bearer_token);

	let (username, slug) = path.into_inner();
	let timeline = egg_mode::list::statuses(ListID::from_slug(username, slug), query.rts.unwrap_or_default(), token)
		.with_page_size(query.count.unwrap_or(200));

	let feed = timeline.call(query.min_id, query.max_id).await.unwrap();

	HttpResponse::Ok()
		.append_header(("x-rate-limit-limit".to_owned(), feed.rate_limit_status.limit.clone()))
		.append_header(("x-rate-limit-remaining".to_owned(), feed.rate_limit_status.remaining.clone()))
		.append_header(("x-rate-limit-reset".to_owned(), feed.rate_limit_status.reset.clone()))
		.json(&feed.response)
}

#[get("/twitter/status/{id}")]
async fn status(id: Identity, tweet_id: Path<u64>, data: web::Data<State>) -> HttpResponse {
	let tokens = &*data.tokens.lock().unwrap();
	let token = get_token(&id, tokens, &data.bearer_token);

	match egg_mode::tweet::show(tweet_id.into_inner(), token).await {
		egg_mode::error::Result::Ok(r) => HttpResponse::Ok()
			.append_header(("x-rate-limit-limit".to_owned(), r.rate_limit_status.limit.clone()))
			.append_header(("x-rate-limit-remaining".to_owned(), r.rate_limit_status.remaining.clone()))
			.append_header(("x-rate-limit-reset".to_owned(), r.rate_limit_status.reset.clone()))
			.json(&r.response),
		egg_mode::error::Result::Err(err) => HttpResponse::InternalServerError().body(err.to_string())
	}
}

#[get("/twitter/user/{username}")]
async fn user_timeline(id: Identity, username: Path<String>, query: web::Query<TimelineQuery>, data: web::Data<State>) -> HttpResponse {
	let tokens = &*data.tokens.lock().unwrap();
	let token = get_token(&id, tokens, &data.bearer_token);

	let timeline = egg_mode::tweet::user_timeline(
		egg_mode::user::UserID::ScreenName(username.into_inner().into()),
		query.replies.unwrap_or(true),
		query.rts.unwrap_or(true),
		token
	)
	.with_page_size(query.count.unwrap_or(200));

	let feed = timeline.call(query.min_id, query.max_id).await.unwrap();

	HttpResponse::Ok()
		.append_header(("x-rate-limit-limit".to_owned(), feed.rate_limit_status.limit.clone()))
		.append_header(("x-rate-limit-remaining".to_owned(), feed.rate_limit_status.remaining.clone()))
		.append_header(("x-rate-limit-reset".to_owned(), feed.rate_limit_status.reset.clone()))
		.json(&feed.response)
}

#[get("/twitter/home")]
async fn home_timeline(id: Identity, query: web::Query<TimelineQuery>, data: web::Data<State>) -> HttpResponse {
	let tokens = &*data.tokens.lock().unwrap();
	let token_opt = get_access_token(&id, tokens);

	if let Some(token) = token_opt {
		let timeline = egg_mode::tweet::home_timeline(token)
			.with_page_size(query.count.unwrap_or(200));

		let feed = timeline.call(query.min_id, query.max_id).await.unwrap();

		HttpResponse::Ok()
			.append_header(("x-rate-limit-limit".to_owned(), feed.rate_limit_status.limit.clone()))
			.append_header(("x-rate-limit-remaining".to_owned(), feed.rate_limit_status.remaining.clone()))
			.append_header(("x-rate-limit-reset".to_owned(), feed.rate_limit_status.reset.clone()))
			.json(&feed.response)
	}else {
		HttpResponse::Unauthorized().finish()
	}
}

#[get("/twitter/login")]
async fn twitter_login(data: web::Data<State>) -> HttpResponse {
	let new_req_token = egg_mode::auth::request_token(&data.con_token, "http://localhost:8080/proxy/twitter/callback").await.unwrap();
	*data.req_token.lock().unwrap() = Some(new_req_token.clone());

	let authorize_url = egg_mode::auth::authorize_url(&new_req_token);
	println!("Redirecting to {}", &authorize_url);
	HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, authorize_url))
		.finish()
}

#[derive(Deserialize)]
struct LoginCallbackQuery {
	//oauth_token: String,
	oauth_verifier: String,
}

#[get("/twitter/callback")]
async fn twitter_login_callback(id: Identity, query: web::Query<LoginCallbackQuery>, data: web::Data<State>) -> HttpResponse {
	if let Some(req_token) = &*data.req_token.lock().unwrap() {
		let (access_token, user_id, _username) = egg_mode::auth::access_token(
			data.con_token.clone(),
			&req_token,
			query.oauth_verifier.clone(),
		).await.unwrap();

		data.tokens.lock().unwrap().insert(user_id.clone(), access_token);
		println!("Remembering id {}", &user_id);
		id.remember(user_id.to_string());
	}

	HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, "http://localhost:8080/"))
		.finish()
}

#[get("/{filename}")]
async fn favviewer(path: web::Path<String>) -> impl Responder {
	let path = path.into_inner();
	match path.as_str() {
		"init" => {
			let js_file = std::fs::read_dir("./dist")
				.unwrap()
				.find_map(|f| {
					f.ok()
						.map(|entry| entry.path())
						.and_then(|path| {
							match path
								.clone()
								.extension()
								.and_then(|ext| ext.to_str())
								.map(|ext| ext == "js") {
								Some(true) => Some(path.clone()),
								_ => None
							}
						})
				});

			match js_file.as_ref().and_then(|f| f.to_str()) {
				Some(file) => actix_files::NamedFile::open(&file),
				None => Err(std::io::Error::from(std::io::ErrorKind::NotFound))
			}
		},
		_ => actix_files::NamedFile::open(format!("dist/{}", &path)),
	}
}

#[get("/index")]
async fn index() -> NamedFile {
	NamedFile::open("dist/index.html").unwrap()
}

/*async fn static_files(req: HttpRequest) -> actix_web::Result<NamedFile> {
	let path: PathBuf = req.match_info().query("filename").parse().unwrap();

	println!("Path: {:?}", &path);
	if path.extension() == Some(std::ffi::OsStr::new("wasm")) {
		println!("{:?} ends with .wasm", &path);
		//Ok(NamedFile::open("dist/index.html")?)
		Ok(NamedFile::open(std::path::Path::new("dist").join(path))?
			.set_content_type("application/wasm".parse().unwrap()))
	}else if path.extension() == Some(std::ffi::OsStr::new("js")) {
		println!("{:?} ends with .js", &path);
		//Ok(NamedFile::open("dist/index.html")?)
		Ok(NamedFile::open(std::path::Path::new("dist").join(path))?
			.set_content_type(mime::APPLICATION_JAVASCRIPT))
	}else {
		Ok(NamedFile::open(std::path::Path::new("dist").join(path))?)
	}
}*/

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let credentials = std::fs::read_to_string("credentials.json").expect("Couldn't find credentials.json");
	let credentials: Credentials = serde_json::from_str(&credentials)?;

	let con_token = egg_mode::KeyPair::new(credentials.consumer_key, credentials.consumer_secret);
	let data = web::Data::new(State {
		req_token: Mutex::new(None),
		bearer_token: egg_mode::auth::bearer_token(&con_token).await.unwrap(),
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
					.service(user_timeline)
					.service(home_timeline)
					.service(list)
					.service(twitter_login)
					.service(twitter_login_callback)
			)
	})
	.bind("127.0.0.1:3000")?
	.run()
	.await
}
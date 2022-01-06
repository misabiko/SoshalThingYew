# SoshalThingYew [![CI@main](https://github.com/misabiko/SoshalThingYew/actions/workflows/ci.yml/badge.svg?branch=main "CI@main")](https://github.com/misabiko/SoshalThingYew/actions/workflows/ci.yml) <a href="https://bulma.io"> <img src="https://bulma.io/images/made-with-bulma.png" alt="Made with Bulma" width="128" height="24"> </a>

A Rust port of SoshalThing, using Yew.

---

### Dev
Launch the server with: `cargo run -p utils --bin server`  
Needs a `credentials.json` in the working directory with `consumer_key` and `consumer_secret` for a Twitter app.  
That or setting `consumer_key` and `consumer_secret` as environment variables.  
Necessary since Twitter won't allow cross-origin requests.

Serve the app on `localhost:8080`: `trunk serve`

### Release

If the server is built in release mode, it will serve the app on `localhost:8080` without need for trunk.  
So only the `server.exe`, `dist/` folder and `credentials.json` or environment variables are needed.

use serde_json::json;
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    router
        .get("/", |req, _| {
            if let Ok(Some(ip)) = req.headers().get("CF-Connecting-IP") {
                Response::ok(ip)
            } else {
                Response::ok("no idea")
            }
        })
        .get("/json", |req, _| {
            if let Ok(Some(ip)) = req.headers().get("CF-Connecting-IP") {
                Response::from_json(&json!({ "ip": ip }))
            } else {
                Response::from_json(&json!({ "err": "no idea" }))
            }
        })        
        .get("/version", |_, _| {
            let version = format!("v{}", env!("CARGO_PKG_VERSION"));
            Response::ok(version)
        })
        .run(req, env)
        .await
}

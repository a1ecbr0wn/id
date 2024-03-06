use serde_json::json;
use worker::*;

mod rate;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().unwrap().coordinates().unwrap_or_default(),
        req.cf()
            .unwrap()
            .region()
            .unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |req, ctx| checked(req, ctx, |ip| Response::ok(ip)))
        .get_async("/json", |req, ctx| {
            checked(req, ctx, |ip| Response::from_json(&json!({ "ip": ip })))
        })
        .get_async("/version", |req, ctx| {
            checked(req, ctx, |_| {
                let version = format!("v{}", env!("CARGO_PKG_VERSION"));
                Response::ok(version)
            })
        })
        .run(req, env)
        .await
}

// Check we have the ip header and check that the rate does not exceed the threshold
async fn checked<F>(req: Request, ctx: RouteContext<()>, f: F) -> Result<Response>
where
    F: FnOnce(&str) -> Result<Response>,
{
    if let Ok(Some(ip)) = req.headers().get("CF-Connecting-IP") {
        if let Ok(store) = ctx.kv("id") {
            if let Err(x) = rate::rate_control(store, &ip).await {
                Response::error(x, 429)
            } else {
                f(&ip)
            }
        } else {
            Response::error("Service unavailable :(", 503)
        }
    } else {
        Response::error("Missing header", 424)
    }
}

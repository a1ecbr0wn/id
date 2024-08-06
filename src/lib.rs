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
            .unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |req, ctx| {
            checked(
                req,
                ctx,
                |ip| Response::ok(ip),
                |msg, status| Response::error(msg, status),
            )
        })
        .get_async("/favicon.ico", |req, ctx| {
            env.bucket(binding)
            checked(
                req,
                ctx,
                |ip| Response::ok(ip),
                |msg, status| Response::error(msg, status),
            )
        })
        .get_async("/json", |req, ctx| {
            checked(
                req,
                ctx,
                |ip| Response::from_json(&json!({ "ip": ip })),
                |msg, status| Response::from_json(&json!({ "status": status, "msg": msg })),
            )
        })
        .get_async("/version", |req, ctx| {
            checked(
                req,
                ctx,
                |_| {
                    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
                    Response::ok(version)
                },
                |msg, status| Response::error(msg, status),
            )
        })
        .get("/hello", |_, _| Response::ok("Hello, World!"))
        .run(req, env)
        .await
    // Response::ok("Hello, World!")
}

// Check we have the ip header and check that the rate does not exceed the threshold
async fn checked<O, E>(req: Request, ctx: RouteContext<()>, o: O, e: E) -> Result<Response>
where
    O: FnOnce(&str) -> Result<Response>,
    E: FnOnce(&str, u16) -> Result<Response>,
{
    if let Ok(Some(ip)) = req.headers().get("CF-Connecting-IP") {
        if let Ok(store) = ctx.kv("id") {
            if let Err(x) = rate::rate_control(store, &ip).await {
                e(&x, 429)
            } else {
                o(&ip)
            }
        } else {
            e("Service unavailable :(", 503)
        }
    } else {
        console_log!("Missing 'CF-Connecting-IP' header");
        console_log!("{:?}", req.headers());
        e("Missing 'CF-Connecting-IP' header", 424)
    }
}

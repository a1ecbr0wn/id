use serde_json::json;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

mod rate;
mod utils;

fn log_request(req: &Request) {
    if let Some(cf) = req.cf() {
        console_log!(
            "{} - [{}], located at: {:?}, within: {}",
            Date::now().to_string(),
            req.path(),
            cf.coordinates().unwrap_or_default(),
            cf.region().unwrap_or("unknown region".into())
        );
    } else {
        console_log!("{} - [{}]", Date::now().to_string(), req.path());
    }
}

#[event(start)]
fn start() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false) // Only partially supported across JavaScript runtimes
                .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
                .with_writer(MakeConsoleWriter), // write events to the console
        )
        .with(performance_layer().with_details_from_fields(Pretty::default()))
        .init();
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
        .get_async("/id", |req, ctx| {
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
        .get_async("/more", |req, ctx| {
            let mut more = "".to_owned();
            if let Ok(Some(user_agent)) = req.headers().get("User-Agent") {
                more.push_str(&format!("\nUser Agent: {user_agent}"));
            }
            if let Some(cf) = req.cf() {
                more.push_str(&format!("\nHTTP Protocol: {}", cf.http_protocol()));
                more.push_str(&format!("\nTLS Version: {}", cf.tls_version()));
                more.push_str(&format!("\nTLS Cipher: {}", cf.tls_cipher()));
                if let Some(asn) = cf.asn() {
                    more.push_str(&format!("\nASN: {asn}"));
                }
                if let Some(organization) = cf.as_organization() {
                    more.push_str(&format!("\nOrganization: {organization}"));
                }
                more.push_str(&format!("\nTimezone: {}", cf.timezone_name()));
                if let Some(coordinates) = cf.coordinates() {
                    let (longitude, latitude) = coordinates;
                    more.push_str(&format!("\nLongitude: {longitude}"));
                    more.push_str(&format!("\nLatitude: {latitude}"));
                }
                if let Some(city) = cf.city() {
                    more.push_str(&format!("\nCity: {city}"));
                }
                if let Some(region) = cf.region() {
                    more.push_str(&format!("\nRegion: {region}"));
                }
                if let Some(region_code) = cf.region_code() {
                    more.push_str(&format!("\nRegion Code: {region_code}"));
                }
                if let Some(postal_code) = cf.postal_code() {
                    more.push_str(&format!("\nPostal Code: {postal_code}"));
                }
                if let Some(country) = cf.country() {
                    more.push_str(&format!("\nCountry: {country}"));
                }
            }

            if let Ok(Some(region)) = req.headers().get("region") {
                more.push_str(&format!("\nRegion: {region}"));
            }
            if let Ok(Some(region_code)) = req.headers().get("regionCode") {
                more.push_str(&format!("\nRegion Code: {region_code}"));
            }
            if let Ok(Some(country)) = req.headers().get("country") {
                more.push_str(&format!("\nCountry: {country}"));
            }
            if let Ok(Some(continent)) = req.headers().get("continent") {
                more.push_str(&format!("\nContinent: {continent}"));
            }
            checked(
                req,
                ctx,
                move |ip| Response::ok(format!("IpAddress: {ip}{more}")),
                |msg, status| Response::error(msg, status),
            )
        })
        .get_async("/more-json", |req, ctx| {
            let user_agent = req.headers().get("User-Agent").ok().flatten().map(|s| s.to_string());
            let cf_data = req.cf().cloned();
            
            checked(
                req,
                ctx,
                move |ip| {
                    let mut more = json!({ "ip": ip });
                    
                    if let Some(user_agent) = user_agent {
                        more["user_agent"] = json!(user_agent);
                    }
                    if let Some(cf) = cf_data {
                        more["http_protocol"] = json!(cf.http_protocol());
                        more["tls_version"] = json!(cf.tls_version());
                        more["tls_cipher"] = json!(cf.tls_cipher());
                        if let Some(asn) = cf.asn() {
                            more["asn"] = json!(asn);
                        }
                        if let Some(organization) = cf.as_organization() {
                            more["organization"] = json!(organization);
                        }
                        if let Some(coordinates) = cf.coordinates() {
                            let (longitude, latitude) = coordinates;
                            more["coordinates"] = json!([longitude, latitude]);
                        }
                        if let Some(city) = cf.city() {
                            more["city"] = json!(city);
                        }
                        if let Some(region) = cf.region() {
                            more["region"] = json!(region);
                        }
                        if let Some(region_code) = cf.region_code() {
                            more["region_code"] = json!(region_code);
                        }
                        if let Some(postal_code) = cf.postal_code() {
                            more["postal_code"] = json!(postal_code);
                        }
                        if let Some(country) = cf.country() {
                            more["country"] = json!(country);
                        }
                        more["timezone"] = json!(cf.timezone_name());
                    }
                    
                    Response::from_json(&more)
                },
                |msg, status| Response::error(msg, status),
            )
        })
        .get_async("/version", |req, ctx| {
            checked(
                req,
                ctx,
                |_| {
                    let version = format!(
                        "v{} ({})",
                        env!("CARGO_PKG_VERSION"),
                        env!("GIT_HASH_SHORT")
                    );
                    Response::ok(version)
                },
                |msg, status| Response::error(msg, status),
            )
        })
        .run(req, env)
        .await
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

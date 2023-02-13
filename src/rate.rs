use serde_json::{from_str, json};
use worker::console_log;
use worker::kv::KvStore;
use worker::Date;

const CHECK_PERIOD: u64 = 120;
const MAX_NUM: usize = 5;

pub async fn rate_control(store: KvStore, name: &str) -> Result<(), String> {
    let now_sec = Date::now().as_millis() / 1000;
    if let Ok(val) = store.get(name).text().await {
        if let Some(val) = val {
            console_log!("now: {now_sec} | value: {val}");
            let ts_secs: Vec<u64> = from_str(val.as_str()).unwrap();
            let mut ts_secs: Vec<u64> = ts_secs
                .into_iter()
                .filter(|x| now_sec - x < CHECK_PERIOD)
                .collect();
            ts_secs.push(now_sec);
            let to_store = json!(ts_secs);
            if ts_secs.len() > MAX_NUM {
                if let Ok(to_put) = store.put(name, json!(&to_store)) {
                    let _ = to_put.expiration_ttl(CHECK_PERIOD).execute().await;
                    Err("Too many requests".to_string())
                } else {
                    Err("data store not available".to_string())
                }
            } else if let Ok(to_put) = store.put(name, json!(&to_store)) {
                let _ = to_put.expiration_ttl(CHECK_PERIOD).execute().await;
                Ok(())
            } else {
                Err("data store not available".to_string())
            }
        } else {
            console_log!("now: {now_sec}");
            let ts_secs: Vec<u64> = vec![now_sec];
            let to_store = json!(ts_secs);
            if let Ok(to_put) = store.put(name, &to_store) {
                let _ = to_put.expiration_ttl(CHECK_PERIOD).execute().await;
                Ok(())
            } else {
                Err("data store not available".to_string())
            }
        }
    } else {
        Err("Too many requests - not found".to_string())
    }
}

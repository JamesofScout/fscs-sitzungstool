use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::models::Sitzung;

pub const SITE_URL: &str = match option_env!("FSCS_SITE_URL") {
    Some(a) if a.is_empty() => a,
    _ => "https://fscs.hhu.de",
};

pub async fn api_get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{SITE_URL}/api{path}");
    let response = Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned HTTP {}", response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| format!("API response could not be decoded: {error}"))
}

pub async fn api_post<T: DeserializeOwned, B: Serialize>(path: &str, body: B) -> Result<T, String> {
    let url = format!("{SITE_URL}/api{path}");
    let response = Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned HTTP {}", response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| format!("API response could not be decoded: {error}"))
}

pub async fn api_post_without_body<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{SITE_URL}/api{path}");
    let response = Client::new()
        .post(&url)
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned HTTP {}", response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| format!("API response could not be decoded: {error}"))
}

pub async fn api_patch<B: Serialize>(path: &str, body: B) -> Result<(), String> {
    let url = format!("{SITE_URL}/api{path}");
    let response = Client::new()
        .patch(&url)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned HTTP {}", response.status()));
    }

    Ok(())
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let url = format!("{SITE_URL}/api{path}");
    let response = Client::new()
        .delete(&url)
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned HTTP {}", response.status()));
    }

    Ok(())
}

pub async fn api_delete_with_body<B: Serialize>(path: &str, body: B) -> Result<(), String> {
    let url = format!("{SITE_URL}/api{path}");
    let response = Client::new()
        .delete(&url)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned HTTP {}", response.status()));
    }

    Ok(())
}

pub fn encode_query(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub fn format_date(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|_| value.to_string())
}

pub fn format_datetime(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.format("%d.%m.%Y, %H:%M Uhr").to_string())
        .unwrap_or_else(|_| value.to_string())
}

pub fn format_location(value: &str) -> String {
    if value.is_empty() {
        "Ort offen".to_string()
    } else {
        value.to_string()
    }
}

pub fn protocol_url(session: &Sitzung) -> Option<String> {
    let date = DateTime::parse_from_rfc3339(&session.datetime)
        .ok()?
        .with_timezone(&Utc);
    if date >= Utc::now() {
        return None;
    }

    let date = date.format("%Y-%m-%d");
    let suffix = match session.typ.as_str() {
        "vv" => "vv-protokoll",
        "wahlvv" => "wahl-vv-protokoll",
        _ => "protokoll",
    };
    Some(format!("{SITE_URL}/de/protokolle/{date}-{suffix}"))
}

pub fn session_sort_key(session: &Sitzung) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&session.datetime)
        .map(|date| date.with_timezone(&Utc))
        .unwrap_or(DateTime::<Utc>::MIN_UTC)
}

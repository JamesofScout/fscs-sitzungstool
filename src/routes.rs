use web_sys::window;

pub fn route_from_path(path: &str) -> (String, Option<String>) {
    let path = path.trim_end_matches('/');
    match path {
        "" | "/" => ("Sitzungen".to_string(), None),
        "/sitzungen" => ("Sitzungen".to_string(), None),
        "/antrag-einreichen" => ("Antrag einreichen".to_string(), None),
        "/sitzung-erstellen" => ("Sitzung erstellen".to_string(), None),
        "/antraege-loeschen" => ("Anträge löschen".to_string(), None),
        path if path.starts_with("/sitzungen/") => (
            "Sitzungsdetails".to_string(),
            path.strip_prefix("/sitzungen/").map(str::to_string),
        ),
        _ => ("Sitzungen".to_string(), None),
    }
}

pub fn path_for_route(page: &str, session_id: Option<&str>) -> String {
    match page {
        "Sitzungen" => "/sitzungen".to_string(),
        "Sitzungsdetails" => session_id
            .map(|id| format!("/sitzungen/{id}"))
            .unwrap_or_else(|| "/sitzungen".to_string()),
        "Antrag einreichen" => "/antrag-einreichen".to_string(),
        "Sitzung erstellen" => "/sitzung-erstellen".to_string(),
        "Anträge löschen" => "/antraege-loeschen".to_string(),
        _ => "/sitzungen".to_string(),
    }
}

pub fn frontend_origin() -> String {
    window()
        .and_then(|window| window.location().href().ok())
        .filter(|href| !href.is_empty())
        .unwrap_or_else(|| format!("{}/sitzungen", crate::api::SITE_URL))
}

use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{PopStateEvent, window};

use crate::api::{
    SITE_URL, api_delete, api_delete_with_body, api_get, api_patch, api_post,
    api_post_without_body, encode_query, format_date, format_datetime, format_location,
    protocol_url, session_sort_key,
};
use crate::models::{
    Antrag, AssocAntrag, CreateAntrag, CreateSitzung, CreateTop, LegislaturPeriode, Role,
    Sitzung, TopWithAntraege,
};
use crate::routes::{frontend_origin, path_for_route, route_from_path};

#[component]
pub fn App() -> Element {
    let (initial_page, initial_session_id) = window()
        .map(|window| route_from_path(&window.location().pathname().unwrap_or_default()))
        .unwrap_or_else(|| ("Sitzungen".to_string(), None));
    let mut page = use_signal(|| initial_page);
    let selected_session_id = use_signal(|| initial_session_id);
    let mut dark_mode = use_signal(|| {
        window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("fscs-dark-mode").ok().flatten())
            .as_deref()
            == Some("true")
    });
    let mut skip_history_update = use_signal(|| true);
    {
        let mut page = page;
        let mut selected_session_id = selected_session_id;
        let mut skip_history_update = skip_history_update;
        use_hook(move || {
            let Some(window) = window() else {
                return;
            };
            let callback_window = window.clone();
            let callback = Closure::wrap(Box::new(move |_event: PopStateEvent| {
                let Ok(pathname) = callback_window.location().pathname() else {
                    return;
                };
                let (next_page, next_session_id) = route_from_path(&pathname);
                skip_history_update.set(true);
                selected_session_id.set(next_session_id);
                page.set(next_page);
            }) as Box<dyn FnMut(PopStateEvent)>);
            let _ = window
                .add_event_listener_with_callback("popstate", callback.as_ref().unchecked_ref());
            callback.forget();
        });
    }
    use_effect(move || {
        let current_page = page();
        let current_session_id = selected_session_id();
        if skip_history_update() {
            skip_history_update.set(false);
            return;
        }
        if let Some(window) = window() {
            if let Ok(history) = window.history() {
                let path = path_for_route(&current_page, current_session_id.as_deref());
                let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
            }
        }
    });

    let session_date = use_signal(String::new);
    let session_time = use_signal(String::new);
    let session_deadline_date = use_signal(String::new);
    let session_deadline_time = use_signal(String::new);
    let session_location = use_signal(String::new);
    let session_type = use_signal(|| "normal".to_string());
    let mut session_period = use_signal(String::new);
    let session_feedback = use_signal(|| None::<String>);
    let legislative_period_name = use_signal(String::new);
    let legislative_period_feedback = use_signal(|| None::<String>);
    let application_title = use_signal(String::new);
    let application_text = use_signal(String::new);
    let application_reason = use_signal(String::new);
    let application_feedback = use_signal(|| None::<String>);
    let top_name = use_signal(String::new);
    let top_content = use_signal(String::new);
    let top_type = use_signal(|| "normal".to_string());
    let top_feedback = use_signal(|| None::<String>);
    let selected_new_top_antraege = use_signal(Vec::<String>::new);
    let selected_antrag_id = use_signal(|| None::<String>);
    let expanded_top_id = use_signal(|| None::<String>);
    let mut period_refresh = use_signal(|| 0u32);
    let sessions = use_resource(|| async { api_get::<Vec<Sitzung>>("/sitzungen").await });
    let legislative_periods = use_resource(move || {
        let _ = period_refresh();
        async { api_get::<Vec<LegislaturPeriode>>("/legislative-periods").await }
    });
    use_effect(move || {
        if page() == "Sitzung erstellen" {
            period_refresh += 1;
        }
    });
    let mut session_tops = use_resource(move || {
        let selected_id = selected_session_id();
        async move {
            match selected_id {
                Some(id) => api_get::<Vec<TopWithAntraege>>(&format!("/sitzungen/{id}/tops")).await,
                None => Ok(Vec::new()),
            }
        }
    });
    let mut orphan_antraege = use_resource(move || {
        let _ = page();
        async { api_get::<Vec<Antrag>>("/antraege/orphans").await }
    });

    let periods = legislative_periods
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let default_period = periods.last().map(|period| period.id.clone());
    use_effect(move || {
        if session_period().is_empty() {
            if let Some(period_id) = default_period.clone() {
                session_period.set(period_id);
            }
        }
    });

    let mut session_list = sessions
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    session_list.sort_by_key(session_sort_key);
    session_list.reverse();
    let selected_session = selected_session_id().and_then(|id| {
        session_list
            .iter()
            .find(|session| session.id == id)
            .cloned()
    });
    let selected_tops = session_tops
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let orphan_list = orphan_antraege
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    let navigation = [
        ("Sitzungen", "▣"),
        ("Antrag einreichen", "＋"),
        ("Verwaiste Anträge", "🗑"),
    ];

    rsx! {
        style { {include_str!("style.css")} }
        div { class: if dark_mode() { "app dark-mode" } else { "app" },
            header { class: "topbar",
                div { class: "topbar-inner",
                    button {
                        class: "mobile-menu",
                        aria_label: "Navigation öffnen",
                        onclick: move |_| page.set("Sitzungen".to_string()),
                        "☰"
                    }
                    a { class: "brand", href: "#",
                        span { class: "brand-mark", "FS" }
                        span { class: "brand-copy",
                            strong { "Fachschaft" }
                            span { "Informatik" }
                        }
                    }
                    div { class: "topbar-actions",
                        button {
                            class: "round-button",
                            aria_label: "Darkmode umschalten",
                            aria_pressed: "{dark_mode()}",
                            title: if dark_mode() { "Helles Design aktivieren" } else { "Dunkles Design aktivieren" },
                            onclick: move |_| {
                                let enabled = !dark_mode();
                                dark_mode.set(enabled);
                                if let Some(storage) = window().and_then(|window| window.local_storage().ok().flatten()) {
                                    let _ = storage.set_item("fscs-dark-mode", if enabled { "true" } else { "false" });
                                }
                            },
                            if dark_mode() { "☀" } else { "☾" }
                        }
                        button { class: "round-button", aria_label: "Sprache auswählen", "🇩🇪" }
                        a {
                            class: "login-button",
                            href: format!("{SITE_URL}/auth/login?path={}", encode_query(&frontend_origin())),
                            "Anmelden"
                        }
                    }
                }
            }

            div { class: "page-layout",
                aside { class: "sidebar",
                    div { class: "sidebar-heading",
                        span { class: "sidebar-logo", "FS" }
                        div {
                            strong { "Fachschaft" }
                            span { "Informatik" }
                        }
                    }
                    nav {
                        for (label, icon) in navigation {
                            button {
                                class: if page() == label { "nav-link active" } else { "nav-link" },
                                onclick: move |_| page.set(label.to_string()),
                                span { class: "nav-icon", "{icon}" }
                                span { "{label}" }
                            }
                        }
                    }
                    div { class: "sidebar-footer",
                        span { "HHU Düsseldorf" }
                        span { "FS Informatik" }
                    }
                }

                main { class: "content",
                    match page().as_str() {
                        "Sitzungen" => rsx! {
                            SitzungenPage {
                                sessions: session_list.clone(),
                                page,
                                selected_session_id,
                            }
                        },
                        "Sitzungsdetails" => rsx! {
                            SitzungsdetailsPage {
                                selected_session,
                                selected_tops: selected_tops.clone(),
                                orphan_list: orphan_list.clone(),
                                page,
                                selected_session_id,
                                selected_antrag_id,
                                expanded_top_id,
                                top_feedback,
                                top_name,
                                top_content,
                                top_type,
                                selected_new_top_antraege,
                                on_refresh_tops: move |_| session_tops.restart(),
                                on_refresh_orphans: move |_| orphan_antraege.restart(),
                            }
                        },
                        "Sitzung erstellen" => rsx! {
                            SitzungErstellenPage {
                                periods: periods.clone(),
                                session_date,
                                session_time,
                                session_deadline_date,
                                session_deadline_time,
                                session_location,
                                session_type,
                                session_period,
                                session_feedback,
                                legislative_period_name,
                                legislative_period_feedback,
                                period_refresh,
                            }
                        },
                        "Antrag einreichen" => rsx! {
                            AntragEinreichenPage {
                                application_title,
                                application_text,
                                application_reason,
                                application_feedback,
                            }
                        },
                        "Verwaiste Anträge" => rsx! {
                            VerwaisteAntraegePage {
                                orphan_list: orphan_list.clone(),
                                on_refresh: move |_| orphan_antraege.restart(),
                            }
                        },
                        _ => rsx! {
                            SitzungenPage {
                                sessions: session_list.clone(),
                                page,
                                selected_session_id,
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn SitzungenPage(
    sessions: Vec<Sitzung>,
    mut page: Signal<String>,
    mut selected_session_id: Signal<Option<String>>,
) -> Element {
    rsx! {
        PageHeader { title: "Sitzungen", text: "Alle veröffentlichten Sitzungen des Fachschaftsrats." }
        section { class: "panel",
            div { class: "panel-title-row",
                h2 { "Sitzungsübersicht" }
                div { class: "panel-actions",
                    span { class: "count-badge", "{sessions.len()} Einträge" }
                    button {
                        class: "primary-button compact",
                        onclick: move |_| page.set("Sitzung erstellen".to_string()),
                        "Sitzung erstellen"
                    }
                }
            }
            if sessions.is_empty() {
                EmptyState { text: "Keine Sitzungen konnten geladen werden." }
            } else {
                div { class: "session-list",
                    for session in sessions.iter() {
                        SessionRow {
                            session: session.clone(),
                            on_open: move |id| {
                                selected_session_id.set(Some(id));
                                page.set("Sitzungsdetails".to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SitzungsdetailsPage(
    selected_session: Option<Sitzung>,
    selected_tops: Vec<TopWithAntraege>,
    orphan_list: Vec<Antrag>,
    mut page: Signal<String>,
    selected_session_id: Signal<Option<String>>,
    mut selected_antrag_id: Signal<Option<String>>,
    mut expanded_top_id: Signal<Option<String>>,
    mut top_feedback: Signal<Option<String>>,
    top_name: Signal<String>,
    top_content: Signal<String>,
    mut top_type: Signal<String>,
    mut selected_new_top_antraege: Signal<Vec<String>>,
    on_refresh_tops: EventHandler<()>,
    on_refresh_orphans: EventHandler<()>,
) -> Element {
    let selected_orphan = orphan_list
        .iter()
        .find(|antrag| Some(antrag.id.as_str()) == selected_antrag_id().as_deref())
        .cloned();

    rsx! {
        if let Some(session) = selected_session {
            PageHeader { title: "Sitzungsdetails", text: "Datum, Ort, Antragsfrist und Tagesordnung dieser Sitzung." }
            section { class: "panel detail-panel",
                button {
                    class: "text-button back-button",
                    onclick: move |_| page.set("Sitzungen".to_string()),
                    "← Zurück zur Sitzungsübersicht"
                }
                div { class: "detail-header",
                    div { class: "session-date large",
                        strong { "{format_date(&session.datetime)}" }
                        span { "{session.typ}" }
                    }
                    div {
                        h2 { "{session.legislatur_periode.name}" }
                        p { "{format_datetime(&session.datetime)} · {format_location(&session.ort)}" }
                        small { "Antragsfrist: {format_datetime(&session.antragsfrist)}" }
                    }
                }
                if let Some(url) = protocol_url(&session) {
                    a { class: "protocol-link", href: "{url}", target: "_blank", rel: "noreferrer", "Protokoll öffnen ↗" }
                }
                h2 { class: "agenda-title", "Tagesordnung" }
                if selected_tops.is_empty() {
                    EmptyState { text: "Für diese Sitzung sind keine Tagesordnungspunkte veröffentlicht." }
                } else {
                    div { class: "agenda-list",
                        for (top, expand_id, delete_id) in
                            selected_tops.clone().into_iter().map(|top| {
                                let id = top.id.clone();
                                (top, id.clone(), id)
                            }) {
                            article { class: "agenda-item",
                                div { class: "agenda-number", "{top.weight + 1}" }
                                div {
                                    div { class: "agenda-item-header",
                                        div {
                                            h3 { "{top.name}" }
                                            span { class: "agenda-type", "{top.typ}" }
                                        }
                                        button {
                                            class: "danger-button top-delete",
                                            r#type: "button",
                                            onclick: move |_| {
                                                let path = format!(
                                                    "/sitzungen/{}/tops/{}",
                                                    selected_session_id().unwrap_or_default(),
                                                    delete_id
                                                );
                                                spawn(async move {
                                                    let result = api_delete(&path).await;
                                                    top_feedback.set(Some(match result {
                                                        Ok(()) => {
                                                            on_refresh_tops.call(());
                                                            "TOP erfolgreich gelöscht.".to_string()
                                                        }
                                                        Err(error) => format!("TOP konnte nicht gelöscht werden: {error}"),
                                                    }));
                                                });
                                            },
                                            "TOP löschen"
                                        }
                                    }
                                    if !top.inhalt.is_empty() {
                                        p { "{top.inhalt}" }
                                    }
                                    for (antrag, antrag_top_id) in top
                                        .antraege
                                        .clone()
                                        .into_iter()
                                        .map(|antrag| (antrag, top.id.clone()))
                                    {
                                        div { class: "application-card",
                                            div { class: "application-card-header",
                                                strong { "{antrag.titel}" }
                                                button {
                                                    class: "danger-button application-remove",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        let path = format!(
                                                            "/sitzungen/{}/tops/{}/assoc",
                                                            selected_session_id().unwrap_or_default(),
                                                            antrag_top_id
                                                        );
                                                        let payload = AssocAntrag {
                                                            antrag_id: antrag.id.clone(),
                                                        };
                                                        spawn(async move {
                                                            let result = api_delete_with_body(&path, payload).await;
                                                            top_feedback.set(Some(match result {
                                                                Ok(()) => {
                                                                    on_refresh_tops.call(());
                                                                    on_refresh_orphans.call(());
                                                                    "Antrag vom TOP entfernt.".to_string()
                                                                }
                                                                Err(error) => format!("Antrag konnte nicht vom TOP entfernt werden: {error}"),
                                                            }));
                                                        });
                                                    },
                                                    "Entfernen"
                                                }
                                            }
                                            p { "{antrag.antragstext}" }
                                            small { "Eingereicht am {format_date(&antrag.erstellt_am)}" }
                                        }
                                    }
                                    button {
                                        class: "secondary-button add-application-button",
                                        r#type: "button",
                                        onclick: move |_| expanded_top_id.set(Some(expand_id.clone())),
                                        "Antrag hinzufügen"
                                    }
                                    if expanded_top_id().as_deref() == Some(expand_id.as_str()) {
                                        div { class: "top-association-form",
                                            label { class: "form-field",
                                                span { "Vorhandenen Orphan-Antrag auswählen" }
                                                select {
                                                    value: "{selected_antrag_id().unwrap_or_default()}",
                                                    onchange: move |event| selected_antrag_id.set(Some(event.value())),
                                                    option { value: "", "Antrag auswählen ..." }
                                                    for antrag in orphan_list.clone() {
                                                        option { value: "{antrag.id}", "{antrag.titel}" }
                                                    }
                                                }
                                            }
                                            if orphan_list.is_empty() {
                                                p { class: "form-hint", "Es sind derzeit keine nicht zugeordneten Anträge vorhanden." }
                                            }
                                            if let Some(antrag) = &selected_orphan {
                                                div { class: "application-card",
                                                    div { class: "application-card-header",
                                                        strong { "{antrag.titel}" }
                                                    }
                                                    p { "{antrag.antragstext}" }
                                                    if !antrag.begruendung.trim().is_empty() {
                                                        p { "Begründung: {antrag.begruendung}" }
                                                    }
                                                    small { "Eingereicht am {format_date(&antrag.erstellt_am)}" }
                                                }
                                            }
                                            div { class: "association-actions",
                                                button {
                                                    class: "primary-button",
                                                    r#type: "button",
                                                    disabled: selected_antrag_id().is_none(),
                                                    onclick: move |_| {
                                                        let Some(antrag_id) = selected_antrag_id() else {
                                                            top_feedback.set(Some("Bitte zuerst einen Antrag auswählen.".to_string()));
                                                            return;
                                                        };
                                                        let path = format!(
                                                            "/sitzungen/{}/tops/{}/assoc",
                                                            selected_session_id().unwrap_or_default(),
                                                            expanded_top_id().unwrap_or_default()
                                                        );
                                                        let payload = AssocAntrag { antrag_id };
                                                        spawn(async move {
                                                            let result = api_patch(&path, payload).await;
                                                            top_feedback.set(Some(match result {
                                                                Ok(()) => {
                                                                    on_refresh_tops.call(());
                                                                    on_refresh_orphans.call(());
                                                                    selected_antrag_id.set(None);
                                                                    expanded_top_id.set(None);
                                                                    "Antrag erfolgreich dem TOP zugeordnet.".to_string()
                                                                }
                                                                Err(error) => format!("Antrag konnte nicht zugeordnet werden: {error}"),
                                                            }));
                                                        });
                                                    },
                                                    "Antrag zuordnen"
                                                }
                                                button {
                                                    class: "text-button",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        selected_antrag_id.set(None);
                                                        expanded_top_id.set(None);
                                                    },
                                                    "Abbrechen"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                section { class: "form-panel top-form",
                    h2 { "TOP hinzufügen" }
                    FormField { label: "Titel", placeholder: "Titel des Tagesordnungspunkts", value: top_name, input_type: "text" }
                    label { class: "form-field",
                        span { "TOP-Typ" }
                        select {
                            value: "{top_type}",
                            onchange: move |event| top_type.set(event.value()),
                            option { value: "normal", "Normal" }
                            option { value: "regularia", "Regularia" }
                            option { value: "bericht", "Bericht" }
                            option { value: "verschiedenes", "Verschiedenes" }
                        }
                    }
                    TextAreaField { label: "Inhalt", placeholder: "Beschreibung des Tagesordnungspunkts", value: top_content }
                    if orphan_list.is_empty() {
                        p { class: "form-hint", "Es sind derzeit keine nicht zugeordneten Anträge vorhanden." }
                    } else {
                        div { class: "top-application-selection",
                            span { class: "form-label", "Anträge direkt hinzufügen" }
                            for antrag in orphan_list.clone() {
                                label { class: "application-option",
                                    input {
                                        r#type: "checkbox",
                                        checked: selected_new_top_antraege().contains(&antrag.id),
                                        onchange: move |_| {
                                            let mut selected = selected_new_top_antraege();
                                            if let Some(index) = selected.iter().position(|id| id == &antrag.id) {
                                                selected.remove(index);
                                            } else {
                                                selected.push(antrag.id.clone());
                                            }
                                            selected_new_top_antraege.set(selected);
                                        }
                                    }
                                    span { "{antrag.titel}" }
                                }
                            }
                        }
                    }
                    button {
                        class: "primary-button",
                        r#type: "button",
                        onclick: move |_| {
                            let payload = CreateTop {
                                name: top_name().trim().to_string(),
                                typ: top_type(),
                                inhalt: top_content().trim().to_string(),
                            };
                            let path = format!(
                                "/sitzungen/{}/tops",
                                selected_session_id().unwrap_or_default()
                            );
                            let selected_antraege = selected_new_top_antraege();
                            spawn(async move {
                                let result = api_post::<TopWithAntraege, _>(&path, payload).await;
                                match result {
                                    Ok(created_top) => {
                                        let mut association_error = None;
                                        for antrag_id in selected_antraege {
                                            let association_path = format!(
                                                "/sitzungen/{}/tops/{}/assoc",
                                                selected_session_id().unwrap_or_default(),
                                                created_top.id
                                            );
                                            if let Err(error) = api_patch(
                                                &association_path,
                                                AssocAntrag { antrag_id },
                                            )
                                            .await
                                            {
                                                association_error = Some(error);
                                                break;
                                            }
                                        }
                                        if let Some(error) = association_error {
                                            top_feedback.set(Some(format!(
                                                "TOP wurde erstellt, aber ein Antrag konnte nicht zugeordnet werden: {error}"
                                            )));
                                        } else {
                                            selected_new_top_antraege.set(Vec::new());
                                            on_refresh_tops.call(());
                                            on_refresh_orphans.call(());
                                            top_feedback.set(Some(
                                                "TOP erfolgreich hinzugefügt.".to_string(),
                                            ));
                                        }
                                    }
                                    Err(error) => top_feedback.set(Some(format!(
                                        "TOP konnte nicht hinzugefügt werden: {error}"
                                    ))),
                                }
                            });
                        },
                        "TOP hinzufügen"
                    }
                    if let Some(feedback) = top_feedback() {
                        p { class: "form-feedback", "{feedback}" }
                    }
                }
            }
        } else {
            PageHeader { title: "Sitzung nicht gefunden", text: "Wähle zunächst eine Sitzung aus der Übersicht." }
            button { class: "primary-button", onclick: move |_| page.set("Sitzungen".to_string()), "Zur Übersicht" }
        }
    }
}

#[component]
fn VerwaisteAntraegePage(orphan_list: Vec<Antrag>, on_refresh: EventHandler<()>) -> Element {
    let mut delete_feedback = use_signal(|| None::<String>);

    rsx! {
        PageHeader {
            title: "Verwaiste Anträge",
            text: "Anträge, die noch keinem Tagesordnungspunkt zugeordnet sind, können hier gelöscht werden."
        }
        section { class: "panel",
            div { class: "panel-title-row",
                h2 { "Anträge ohne Zuordnung" }
                span { class: "count-badge", "{orphan_list.len()} Einträge" }
            }
            if orphan_list.is_empty() {
                EmptyState { text: "Keine verwaisten Anträge gefunden." }
            } else {
                div { class: "session-list",
                    for antrag in orphan_list.iter().cloned() {
                        article { class: "agenda-item",
                            div { class: "agenda-number", "!" }
                            div {
                                div { class: "agenda-item-header",
                                    div {
                                        h3 { "{antrag.titel}" }
                                    }
                                    button {
                                        class: "danger-button top-delete",
                                        r#type: "button",
                                        onclick: move |_| {
                                            let antrag_id = antrag.id.clone();
                                            spawn(async move {
                                                let result = api_delete(&format!("/antraege/{antrag_id}")).await;
                                                delete_feedback.set(Some(match result {
                                                    Ok(()) => {
                                                        on_refresh.call(());
                                                        "Antrag erfolgreich gelöscht.".to_string()
                                                    }
                                                    Err(error) => format!("Antrag konnte nicht gelöscht werden: {error}"),
                                                }));
                                            });
                                        },
                                        "Löschen"
                                    }
                                }
                                p { "{antrag.antragstext}" }
                                if !antrag.begruendung.trim().is_empty() {
                                    p { "Begründung: {antrag.begruendung}" }
                                }
                                small { "Eingereicht am {format_date(&antrag.erstellt_am)}" }
                            }
                        }
                    }
                }
            }
            if let Some(feedback) = delete_feedback() {
                p { class: "form-feedback", "{feedback}" }
            }
        }
    }
}

#[component]
fn AntraegePage(mut page: Signal<String>) -> Element {
    rsx! {
        PageHeader { title: "Anträge", text: "Anträge und Beschlüsse werden über das Sitzungsarchiv veröffentlicht." }
        section { class: "callout",
            span { class: "card-icon", "≡" }
            div {
                h2 { "Anträge einsehen" }
                p { "Öffne eine Sitzung, um Tagesordnungspunkte und zugehörige Anträge einzusehen." }
                button { class: "primary-button", onclick: move |_| page.set("Sitzungen".to_string()), "Zu den Sitzungen" }
            }
        }
    }
}

#[component]
fn SitzungErstellenPage(
    periods: Vec<LegislaturPeriode>,
    session_date: Signal<String>,
    session_time: Signal<String>,
    session_deadline_date: Signal<String>,
    session_deadline_time: Signal<String>,
    session_location: Signal<String>,
    mut session_type: Signal<String>,
    mut session_period: Signal<String>,
    mut session_feedback: Signal<Option<String>>,
    legislative_period_name: Signal<String>,
    mut legislative_period_feedback: Signal<Option<String>>,
    mut period_refresh: Signal<u32>,
) -> Element {
    rsx! {
        PageHeader { title: "Sitzung erstellen", text: "Lege eine neue Sitzung für eine bestehende Legislaturperiode an." }
        section { class: "form-panel",
            div { class: "form-row",
                FormField { label: "Datum", placeholder: "", value: session_date, input_type: "date" }
                FormField { label: "Uhrzeit", placeholder: "", value: session_time, input_type: "time" }
            }
            div { class: "form-row",
                FormField { label: "Antragsfrist – Datum", placeholder: "", value: session_deadline_date, input_type: "date" }
                FormField { label: "Antragsfrist – Uhrzeit", placeholder: "", value: session_deadline_time, input_type: "time" }
            }
            FormField { label: "Ort", placeholder: "25.22.00.82", value: session_location, input_type: "text" }
            label { class: "form-field",
                span { "Sitzungstyp" }
                select {
                    value: "{session_type}",
                    onchange: move |event| session_type.set(event.value()),
                    option { value: "normal", "Normal" }
                    option { value: "vv", "Vollversammlung" }
                    option { value: "wahlvv", "Wahlvollversammlung" }
                    option { value: "ersatz", "Ersatzsitzung" }
                    option { value: "konsti", "Konstituierend" }
                    option { value: "dringlichkeit", "Dringlichkeit" }
                }
            }
            label { class: "form-field",
                span { "Legislaturperiode" }
                select {
                    value: "{session_period}",
                    onchange: move |event| session_period.set(event.value()),
                    if periods.is_empty() {
                        option { value: "", "Legislaturperioden werden geladen ..." }
                    } else {
                        for period in periods.iter().rev() {
                            option { value: "{period.id}", "{period.name}" }
                        }
                    }
                }
            }
            p { class: "form-hint", "Die aktuelle Legislaturperiode wird automatisch vorausgewählt. Die API erhält weiterhin ihre UUID." }
            button {
                class: "primary-button",
                r#type: "button",
                onclick: move |_| {
                    let payload = CreateSitzung {
                        datetime: format!("{}T{}:00Z", session_date(), session_time()),
                        ort: session_location().trim().to_string(),
                        typ: session_type(),
                        antragsfrist: format!("{}T{}:00Z", session_deadline_date(), session_deadline_time()),
                        legislative_period: session_period().trim().to_string(),
                    };
                    spawn(async move {
                        let result = api_post::<Sitzung, _>("/sitzungen", payload).await;
                        session_feedback.set(Some(match result {
                            Ok(_) => "Sitzung erfolgreich erstellt.".to_string(),
                            Err(error) => format!("Sitzung konnte nicht erstellt werden: {error}"),
                        }));
                    });
                },
                "Sitzung speichern"
            }
            if let Some(feedback) = session_feedback() {
                p { class: "form-feedback", "{feedback}" }
            }
        }
        section { class: "form-panel secondary-form",
            h2 { "Neue Legislaturperiode" }
            p { class: "form-hint", "Füge eine neue Periode hinzu. Sie wird anschließend direkt im Sitzungsformular auswählbar." }
            FormField { label: "Name", placeholder: "FSR 26/27", value: legislative_period_name, input_type: "text" }
            button {
                class: "primary-button",
                r#type: "button",
                onclick: move |_| {
                    let name = legislative_period_name().trim().to_string();
                    let path = format!("/legislative-periods?name={}", encode_query(&name));
                    spawn(async move {
                        let result = api_post_without_body::<LegislaturPeriode>(&path).await;
                        legislative_period_feedback.set(Some(match result {
                            Ok(_) => {
                                period_refresh += 1;
                                "Legislaturperiode erfolgreich erstellt.".to_string()
                            }
                            Err(error) => format!("Legislaturperiode konnte nicht erstellt werden: {error}"),
                        }));
                    });
                },
                "Legislaturperiode hinzufügen"
            }
            if let Some(feedback) = legislative_period_feedback() {
                p { class: "form-feedback", "{feedback}" }
            }
        }
    }
}

#[component]
fn AntragEinreichenPage(
    application_title: Signal<String>,
    application_text: Signal<String>,
    application_reason: Signal<String>,
    mut application_feedback: Signal<Option<String>>,
) -> Element {
    rsx! {
        PageHeader { title: "Antrag einreichen", text: "Reiche einen Antrag zur Beratung durch den Fachschaftsrat ein." }
        section { class: "form-panel",
            FormField { label: "Titel", placeholder: "Kurzer Titel des Antrags", value: application_title, input_type: "text" }
            TextAreaField { label: "Antragstext", placeholder: "Der Fachschaftsrat möge beschließen, dass ...", value: application_text }
            TextAreaField { label: "Begründung", placeholder: "Warum soll der Antrag beschlossen werden?", value: application_reason }
            p { class: "form-hint", "Nach dem Absenden wird der Antrag über POST /api/antraege an das Backend übertragen." }
            button {
                class: "primary-button",
                r#type: "button",
                onclick: move |_| {
                    let payload = CreateAntrag {
                        titel: application_title().trim().to_string(),
                        antragstext: application_text().trim().to_string(),
                        begruendung: application_reason().trim().to_string(),
                    };
                    spawn(async move {
                        let result = api_post::<serde_json::Value, _>("/antraege", payload).await;
                        application_feedback.set(Some(match result {
                            Ok(_) => "Antrag erfolgreich eingereicht.".to_string(),
                            Err(error) => format!("Antrag konnte nicht eingereicht werden: {error}"),
                        }));
                    });
                },
                "Antrag einreichen"
            }
            if let Some(feedback) = application_feedback() {
                p { class: "form-feedback", "{feedback}" }
            }
        }
    }
}

#[component]
fn HomePage(
    mut page: Signal<String>,
    session_list_len: usize,
    people_len: usize,
    calendar_list_len: usize,
    latest_session: Option<Sitzung>,
    role_list: Vec<Role>,
) -> Element {
    rsx! {
        section { class: "hero",
            div { class: "hero-content",
                h1 { "Fachschaft Informatik" }
                p { "Deine Anlaufstelle für Studium, Hochschulpolitik und Fachschaftsleben an der HHU." }
                div { class: "hero-actions",
                    button { class: "primary-button light", onclick: move |_| page.set("Sitzungen".to_string()), "Sitzungen ansehen →" }
                    a { class: "secondary-button", href: "{SITE_URL}/de/kontakt/", "Kontakt aufnehmen" }
                }
            }
            div { class: "hero-orbit",
                span { "FS" }
                i {}
                i {}
                i {}
            }
        }

        div { class: "stats-grid",
            StatCard { value: "{session_list_len}", label: "Sitzungen" }
            StatCard { value: "{people_len}", label: "Mitglieder" }
            StatCard { value: "{calendar_list_len}", label: "Kalender" }
        }

        div { class: "dashboard-grid",
            section { class: "panel",
                div { class: "panel-title-row",
                    div {
                        h2 { "Letzte Sitzung" }
                    }
                    button { class: "text-button", onclick: move |_| page.set("Sitzungen".to_string()), "Alle anzeigen →" }
                }
                if let Some(session) = latest_session {
                    SessionFeature { session }
                } else {
                    EmptyState { text: "Keine Sitzungsdaten verfügbar." }
                }
            }
            section { class: "panel accent-panel",
                h2 { "Unsere Rollen" }
                p { "Engagierte Studierende gestalten Studium und Campus mit." }
                div { class: "tag-list",
                    for role in role_list.iter() {
                        span { class: "tag", "{role.name}" }
                    }
                }
                a { class: "text-button", href: "{SITE_URL}/de/aboutus/", "Mehr über uns →" }
            }
        }
    }
}

#[component]
fn PageHeader(title: &'static str, text: &'static str) -> Element {
    rsx! {
        div { class: "page-header",
            h1 { "{title}" }
            p { "{text}" }
        }
    }
}

#[component]
fn StatCard(value: String, label: &'static str) -> Element {
    rsx! {
        div { class: "stat-card",
            strong { "{value}" }
            span { "{label}" }
        }
    }
}

#[component]
fn FormField(
    label: &'static str,
    placeholder: &'static str,
    value: Signal<String>,
    input_type: &'static str,
) -> Element {
    rsx! {
        label { class: "form-field",
            span { "{label}" }
            input {
                r#type: input_type,
                placeholder,
                value: "{value}",
                oninput: move |event| value.set(event.value()),
            }
        }
    }
}

#[component]
fn TextAreaField(label: &'static str, placeholder: &'static str, value: Signal<String>) -> Element {
    rsx! {
        label { class: "form-field",
            span { "{label}" }
            textarea {
                placeholder,
                value: "{value}",
                oninput: move |event| value.set(event.value()),
            }
        }
    }
}

#[component]
fn EmptyState(text: &'static str) -> Element {
    rsx! {
        div { class: "empty-state",
            span { "•" }
            p { "{text}" }
        }
    }
}

#[component]
fn SessionRow(session: Sitzung, on_open: EventHandler<String>) -> Element {
    rsx! {
        div { class: "session-row",
            div { class: "session-date",
                strong { "{format_date(&session.datetime)}" }
                span { "{session.typ}" }
            }
            button {
                class: "session-row-main",
                onclick: move |_| on_open.call(session.id.clone()),
                div { class: "session-details",
                    h3 { "{session.legislatur_periode.name}" }
                    p { "{format_datetime(&session.datetime)} · {format_location(&session.ort)}" }
                    small { "Antragsfrist: {format_datetime(&session.antragsfrist)}" }
                }
                span { class: "arrow", "›" }
            }
        }
    }
}

#[component]
fn SessionFeature(session: Sitzung) -> Element {
    rsx! {
        div { class: "session-feature",
            div { class: "session-date",
                strong { "{format_date(&session.datetime)}" }
                span { "{session.typ}" }
            }
            div { class: "session-details",
                h3 { "{session.legislatur_periode.name}" }
                p { "{format_datetime(&session.datetime)} · {format_location(&session.ort)}" }
                small { "Antragsfrist: {format_datetime(&session.antragsfrist)}" }
            }
        }
    }
}

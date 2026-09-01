# Sitzungstool

Frontend für die Verwaltung von Sitzungen, Tagesordnungspunkten und Anträgen der Fachschaft Informatik.

## Überblick

Die Anwendung bietet eine einfache Weboberfläche für:

- Sitzungsübersicht
- Sitzungsdetails mit Tagesordnung
- Erstellung neuer Sitzungen
- Erstellung neuer Legislaturperioden
- Einreichen von Anträgen
- Verwaltung verwaister Anträge
- Zuweisung von Anträgen zu Tagesordnungspunkten

## Technologien

- Rust
- Dioxus (Web frontend)
- Reqwest für API-Calls
- Serde / Serde JSON
- Chrono für Datums- und Zeitformatierung
- WebAssembly / browser runtime

## Voraussetzungen

- Rust und Cargo
- `trunk` für lokale Dioxus-Webentwicklung

Installation mit Rustup:

```bash
rustup default stable
cargo install trunk
```

## Lokale Entwicklung

1. Backend starten (z. B. auf `http://localhost:8080`)
2. Projekt ausführen:

```bash
trunk serve --open
```

Alternativ die Web-App via Cargo bauen, sofern im Projekt ein passender Run-Target konfiguriert ist.

## Deployment mit Nix


```bash
nix develop
cargo build
trunk build --release
```

Die erzeugte Ausgabe liegt danach im Ordner `dist/` und kann als statische Web-App ausgeliefert werden.

Wichtig: `FSCS_SITE_URL` muss in der Laufzeitumgebung korrekt gesetzt sein, damit die Frontend-API-Requests auf das Backend zeigen.

## Deployment ohne Nix

Ohne Nix kann das Frontend mit den Standard-Rust- und Trunk-Tools gebaut und deployed werden.

### 1. Abhängigkeiten installieren

```bash
rustup default stable
cargo install trunk
```

### 2. Build erzeugen

```bash
trunk build --release
```

### 3. Ausliefern

Die Dateien aus `dist/` können auf einem Webserver veröffentlicht werden, z. B.:

- GitHub Pages
- nginx
- Caddy
- statischer Webserver im Docker-Container

Beispiel für nginx:

```nginx
server {
    listen 80;
    server_name example.com;
    root /var/www/sitzungstool2/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

### 4. Laufzeitkonfiguration

Vor dem Start auf dem Server sollte `FSCS_SITE_URL` gesetzt werden, falls das Frontend nicht lokal auf `localhost:8080` laufen soll.

Beispiel:

```bash
export FSCS_SITE_URL=https://api.example.com
```

## Umgebungsvariablen

Das Frontend verwendet optional die Umgebungsvariable `FSCS_SITE_URL`:

```bash
export FSCS_SITE_URL=http://localhost:8080
```

Wenn sie nicht gesetzt ist, wird als Standardwert verwendet:

```text
http://localhost:8080
```

Diese Variable wird für die API-Anfragen verwendet.

## Projektstruktur

```text
src/
├── main.rs          # Einstiegspunkt
├── app.rs           # Haupt-UI und Komponenten
├── api.rs           # API-Aufrufe und Formatierungsfunktionen
├── models.rs        # Datenmodelle
├── routes.rs        # Routing und URL-Logik
├── style.css        # Styling
```

## Wichtige Funktionen

### Login-Redirect
Der Login-Link setzt den aktuellen Frontend-Standort als `path`-Parameter, damit nach dem Login wieder auf die aktuell aufgerufene Seite zurückgekehrt werden kann.

### Verwaiste Anträge
Unter der Seite `Verwaiste Anträge` lassen sich nicht zugeordnete Anträge anzeigen und löschen.

### Sitzungsverwaltung
Sitzungen können erstellt, angezeigt und mit Tagesordnungspunkten und Anträgen verknüpft werden.

## Hinweise

- Alle API-Aufrufe werden gegen `/api/...` auf `FSCS_SITE_URL` gemacht.
- Die Anwendung erwartet ein kompatibles Backend mit den passenden Endpunkten für Sitzungen, Anträge und Tagesordnungspunkte.
- Das aktuelle Backend ist `https://github.com/fscs/websiter-server`
- Für Produktivbetrieb sollte das Frontend über eine passende Web-Server- oder Deployment-Konfiguration gebündelt werden.

use chrono::{DateTime, Utc};
use futures::prelude::*;
use icalendar::{Calendar, CalendarComponent, Component, Event, EventLike};
use poem::{
    get, handler,
    listener::TcpListener,
    web::{Data, RealIp},
    EndpointExt, IntoResponse, Route, Server,
};
use poem_openapi::payload::PlainText;
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use state::AppState;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod state;

#[async_std::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    info!("Starting ethglobal-events");

    let state = Arc::new(AppState::new().await);

    let app = Route::new()
        .at("/ethglobal.ics", get(get_events))
        .data(state);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    data: PublishedEventsPayload,
}

#[derive(Serialize, Deserialize)]
pub struct PublishedEventsPayload {
    getPublishedEvents: Vec<PublishedEvent>,
}

#[derive(Serialize, Deserialize)]
pub struct PublishedEvent {
    id: u64,
    name: String,
    slug: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "startTime")]
    start_time: DateTime<Utc>,
    #[serde(rename = "endTime")]
    end_time: DateTime<Utc>,
    website: Option<String>,
    city: Option<CityData>,
}

#[derive(Serialize, Deserialize)]
pub struct CityData {
    pub name: String,
    pub country: Option<CountryData>,
}

#[derive(Serialize, Deserialize)]
pub struct CountryData {
    pub name: String,
}

#[handler]
async fn get_events(state: Data<&Arc<AppState>>, ip: RealIp) -> impl IntoResponse {
    let ip_str = ip.0.map(|ip| ip.to_string());
    info!("get_events from {}", ip_str.unwrap_or("unknown".to_string()));

    let client = reqwest::Client::new();
    let query = r#"{"query":"query {\n\tgetPublishedEvents {\n\t\tid\n\t\tname\n\t\tslug\n\t\ttype\n\t\tstartTime\n\t\tendTime\n\t\twebsite\n\t\tcity {\n\t\t\tname\n\t\t\tcountry {\n\t\t\t\tname\n\t\t\t}\n\t\t}\n\t}\n}"}"#;

    let response = client
        .post("https://api.ethglobal.com/graphql")
        .header("Content-Type", "application/json")
        .header("Origin", "https://ethglobal.com")
        .body(query)
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let body = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response".to_string());

                let body: QueryResponse = match serde_json::from_str(&body) {
                    Ok(a) => a,
                    Err(b) => {
                        warn!("Error parsing json {:?}", b);
                        return StatusCode::BAD_REQUEST.into_response();
                    }
                };

                let mut events: Vec<CalendarComponent> = Vec::new();
                for event in body.data.getPublishedEvents {
                    let mut cevent = Event::new();
                    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, event.id.to_string().as_bytes());
                    cevent
                        .uid(uuid.to_string().as_str())
                        .summary(&event.name)
                        .starts(event.start_time)
                        .ends(event.end_time);

                    if let Some(city) = event.city {
                        let location_name =
                            format!("{}, {}", city.name, city.country.unwrap().name);
                        cevent.location(location_name.as_str());
                    }

                    if let Some(website) = event.website {
                        cevent.url(website.as_str());
                    }

                    let event_name = event.name;
                    let event_url = format!("https://ethglobal.com/events/{}", event.slug);
                    let event_type = event._type;

                    let mut description = String::new();
                    description.push_str(&event_name);
                    description.push_str("\\\\n");
                    description.push_str(&event_type);
                    description.push_str("\\\\n\\\\n");
                    description.push_str(&event_url);

                    cevent.description(description.as_str());

                    let cevent = cevent.done();

                    events.push(cevent.into());
                }
                let mut calendar = Calendar::from_iter(events).name("ETHGlobal Events").done();

                let body = calendar.to_string();

                PlainText(body)
                    .with_content_type("text/calendar; charset=utf-8")
                    .with_header(
                        "Content-Disposition",
                        "attachment; filename=\"ethglobal.ics\"",
                    )
                    .with_header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .with_header("Pragma", "no-cache")
                    .with_header("Expires", "0")
                    .with_header("Access-Control-Allow-Origin", "*")
                    // .with_header("Content-Length", format!("{}", calendar.len()))
                    .into_response()
            } else {
                format!("Error: {}", res.status()).into_response()
            }
        }
        Err(err) => {
            error!("Request failed: {}", err);

            "Request failed".to_string().into_response()
        }
    }
}

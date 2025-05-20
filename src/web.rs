use chrono::Local;
use futures::StreamExt;
use json::JsonValue;
use actix_web::{error, Error, HttpResponse};
use std::{collections::HashMap, ops::DerefMut, sync::{Arc, Mutex}};
use crate::{error::ResultOpenHems, schedule::Schedule, server::DecrementTime, time};

const DATE_FORMAT:&str = "%d/%m/%Y";

pub struct AppState {
    pub schedules: HashMap<String, Arc<Mutex<Schedule>>>,
}
impl AppState {
	pub fn new() -> Self {
		AppState {
			schedules: HashMap::new(),
		}
	}
	pub fn decrement_time(&self, duration:u32) -> ResultOpenHems<bool> {
		log::debug!("AppState::decrement_time() for {} seconds", duration);
		for schedule in self.schedules.values() {
			let mut sch = schedule.lock().unwrap();
			sch.decrement_time(duration)?;
		}
		Ok(true)
	}
}

fn nodes_json(data: &AppState) -> String {
	let mut nodes = "{".to_string();
	let mut sep = "";
	for (key, schedule) in &data.schedules {
		let schedule = schedule.lock().unwrap();
		nodes.push_str(sep);
		nodes.push_str(&format!(
				"\"{}\":{{\"name\":\"{}\", \"duration\":{}, \"date\":\"{}\", \"timeout\":\"{}\"}}",
				key, schedule.get_name(), schedule.get_duration(),
				schedule.get_timeout().format(DATE_FORMAT), schedule.get_timeout().format("%H:%M")));
		sep = ",";
	}
	nodes.push_str("}");
	nodes
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k
pub async fn states(
			_tmpl: actix_web::web::Data<tera::Tera>,
			data: actix_web::web::Data<Arc<AppState>>,
			mut payload: actix_web::web::Payload
		) -> Result<HttpResponse, Error> {
	// payload is a stream of Bytes objects
    let mut body = actix_web::web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
	let json_str = String::from_utf8(body.to_vec()).map_err(|_| error::ErrorBadRequest("invalid utf-8"))?;
	println!("Received body: {}", json_str);
	let json_values = json::parse(&json_str).unwrap();
	if let JsonValue::Object(object) =  json_values {
		for (key, schedule_json) in object.iter() {
			if let Some(schedule) = data.schedules.get(key) {
				if let JsonValue::Object(sch) = schedule_json {

					let mut update = false;
					let mut timeout = time::MIN_DATETIME.clone();
					let mut duration = 0;
					if let Some(d) = sch.get("duration") {
						if let Some(d1) = d.as_i32() {
							duration = d1 as u32;
							update = true;
						}
					}
					if let Some(date) = sch.get("timeout") {
						if let Some(d1) = date.as_str() {
							if let Ok(timeout_new) = chrono::NaiveTime::parse_from_str(d1, "%H:%M") {
								let now = Local::now();
								timeout = time::time2datetime(&timeout_new, &now);
								update = true;
							}
						}
					}
					if update {
						let mut schedule_mutex = schedule.lock().unwrap();
						schedule_mutex.set_duration(duration);
						schedule_mutex.set_timeout(&timeout);
					} else {
						return Err(error::ErrorBadRequest("missing timeout"));
					}
				}
			}
		}
	}

	let nodes = nodes_json(&data);
	let mut response = HttpResponse::Ok().body(nodes);
	response.headers_mut().insert(
		actix_web::http::header::HeaderName::from_static("content-type"),
		actix_web::http::header::HeaderValue::from_static("text/plain")
	);
    Ok(response)
}

pub async fn index(
			tmpl: actix_web::web::Data<tera::Tera>,
			data: actix_web::web::Data<Arc<AppState>>
		) -> HttpResponse {
	let mut ctx = tera::Context::new();
    ctx.insert("translate_tooltip_duration", "Duration");
    ctx.insert("tooltip_duration", "Duration");
    ctx.insert("translate_tooltip_timeout", "Timeout");
    ctx.insert("tooltip_timeout", "Timeout");
    ctx.insert("text_for", "for");
    ctx.insert("text_before", "before");
    ctx.insert("DATE_FORMAT", &DATE_FORMAT);
	let nodes = nodes_json(&data);
    ctx.insert("nodes", &nodes);
	let rendered = tmpl.render("panel.jinja2", &ctx)
        .unwrap_or_else(|_| "Template error".into());
    HttpResponse::Ok().body(rendered)
}
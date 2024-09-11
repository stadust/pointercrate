use pointercrate_demonlist::config;
use rocket::serde::json::Json;
use serde_json::{json, Value};

#[rocket::get("/")]
pub fn list_information() -> Json<Value> {
    let data = json! {
        {
            "list_size": config::list_size(),
            "extended_list_size": config::extended_list_size()
        }
    };

    Json(data)
}

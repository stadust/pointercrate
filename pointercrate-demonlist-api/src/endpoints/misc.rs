use pointercrate_demonlist::config;
use rocket::response::content::Json;
use serde_json::json;

#[rocket::get("/")]
pub fn list_information() -> Json<String> {
    let data = json! {
        {
            "list_size": config::list_size(),
            "extended_list_size": config::extended_list_size()
        }
    };

    Json(data.to_string())
}

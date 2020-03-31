use crate::config;
use actix_web::HttpResponse;
use actix_web_codegen::get;
use serde_json::json;

#[get("/list_information/")]
pub fn list_information() -> HttpResponse {
    HttpResponse::Ok().json(json! {
        {
            "list_size": config::list_size(),
            "extended_list_size": config::extended_list_size()
        }
    })
}

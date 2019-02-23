use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    state::PointercrateState,
};
use actix_web::{HttpRequest, HttpResponse, Responder};
use serde_json::json;

pub fn list_information(_: &HttpRequest<PointercrateState>) -> impl Responder {
    HttpResponse::Ok().json(json! {
        {
            "list_size": (*LIST_SIZE),
            "extended_list_size": (*EXTENDED_LIST_SIZE)
        }
    })
}

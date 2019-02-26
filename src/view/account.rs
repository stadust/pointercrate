use super::Page;
use crate::{
    actor::database::TokenAuth, api::PCResponder, model::user::User, state::PointercrateState,
};
use actix_web::{AsyncResponder, HttpRequest, Responder};
use log::info;
use maud::{html, Markup};
use tokio::prelude::Future;

#[derive(Debug)]
pub struct AccountPage {
    user: User,
}

pub fn handler(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /account/");

    let req_clone = req.clone();

    req.state()
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .map(move |user: User| {
            AccountPage { user }
                .render(&req_clone)
                .respond_to(&req_clone)
                .unwrap()
        })
        .responder()
}

impl Page for AccountPage {
    fn title(&self) -> String {
        format!("Account - {}", self.user.name)
    }

    fn description(&self) -> String {
        String::new()
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec![]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            div.m-center.flex.panel.fade.col.wrap style = "margin: 100px 0px;"{
                h1.underlined.pad {
                    "Pointercrate Account"
                }
            }
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![]
    }
}

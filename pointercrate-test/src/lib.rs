//! Utilities for pointercrate integration tests

use pointercrate_user::AuthenticatedUser;

use rocket::{
    http::{Header, Status},
    local::asynchronous::{Client, LocalRequest, LocalResponse},
};
use serde::{de::DeserializeOwned, Serialize};

use std::collections::HashMap;

pub mod demonlist;
pub mod user;

pub struct TestClient(Client);

impl TestClient {
    fn new(client: Client) -> Self {
        TestClient(client)
    }

    pub fn get(&self, url: impl Into<String>) -> TestRequest {
        TestRequest::new(self.0.get(url.into()))
    }

    pub fn put(&self, url: impl Into<String>) -> TestRequest {
        TestRequest::new(self.0.put(url.into()))
    }

    pub fn post(&self, url: impl Into<String>, body: &impl Serialize) -> TestRequest {
        TestRequest::new(self.0.post(url.into()).json(body))
    }
}

pub struct TestRequest<'c> {
    request: LocalRequest<'c>,
    expected_status: Status,
    expected_headers: HashMap<String, String>,
}

impl<'c> TestRequest<'c> {
    fn new(request: LocalRequest<'c>) -> Self {
        TestRequest {
            request,
            expected_status: Status::Ok,
            expected_headers: HashMap::new(),
        }
        .header("X-Real-Ip", "127.0.0.1")
    }

    pub fn header(mut self, header_name: impl Into<String>, header_value: impl Into<String>) -> Self {
        self.request = self.request.header(Header::new(header_name.into(), header_value.into()));
        self
    }

    pub fn authorize_as(self, user: &AuthenticatedUser) -> Self {
        self.header("Authorization", format!("Bearer {}", user.generate_access_token()))
    }

    pub fn expect_status(mut self, status: Status) -> Self {
        self.expected_status = status;
        self
    }

    pub fn expect_header(mut self, header_name: impl Into<String>, header_value: impl Into<String>) -> Self {
        self.expected_headers.insert(header_name.into(), header_value.into());
        self
    }

    pub async fn get_result<Result: DeserializeOwned>(self) -> Result {
        let body_text = self.execute().await.into_string().await.unwrap();
        serde_json::from_str(&body_text).unwrap()
    }

    pub async fn execute(self) -> LocalResponse<'c> {
        let response = self.request.dispatch().await;

        assert_eq!(response.status(), self.expected_status);

        for (name, value) in self.expected_headers {
            let header = response.headers().get_one(&name);

            assert!(header.is_some(), "missing required header value");

            assert_eq!(header.unwrap(), value);
        }

        response
    }
}

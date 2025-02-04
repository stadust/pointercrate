use crate::User;

use super::AuthenticationType;

pub struct OAuth2AuthenticatedUser {
    user: User,
    google_account_id: Option<String>,
    discord_account_id: Option<String>,
}

pub mod get;
pub mod post;

impl OAuth2AuthenticatedUser {
    pub fn into_user(self) -> User {
        self.user
    }

    pub fn user(&self) -> &User {
        &self.user
    }
}

impl AuthenticationType {
    pub fn oauth2(user: User, google_account_id: Option<String>, discord_account_id: Option<String>) -> Self {
        AuthenticationType::OAuth2(OAuth2AuthenticatedUser {
            user,
            google_account_id,
            discord_account_id,
        })
    }
}

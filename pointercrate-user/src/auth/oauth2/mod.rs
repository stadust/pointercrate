use crate::User;

use super::AuthenticatedUser;

pub struct OAuth2AuthenticatedUser {
    user: User,
    email_address: String,
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

    pub fn oauth2(user: User, email_address: String, google_account_id: String) -> Self {
        AuthenticatedUser::OAuth2(OAuth2AuthenticatedUser {
            user,
            email_address,
            google_account_id: Some(google_account_id),
            discord_account_id: None,
        })
    }
}

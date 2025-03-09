mod post;

#[cfg(feature = "oauth2")]
pub use post::GoogleOauthPayload;

use crate::User;

use super::AuthenticationType;

pub struct OA2AuthenticatedUser {
    user: User,
}

impl OA2AuthenticatedUser {
    pub fn into_user(self) -> User {
        self.user
    }

    pub fn user(&self) -> &User {
        &self.user
    }
}

impl AuthenticationType {
    pub fn oauth(user: User) -> AuthenticationType {
        AuthenticationType::Oauth2(OA2AuthenticatedUser { user })
    }
}

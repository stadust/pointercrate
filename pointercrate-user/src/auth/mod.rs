//! Module containing pointercrate's authentication system
//!
//! Pointercrate makes use of three authentication methods, although the first one can be considered deprecated:
//! 1. HTTP Basic Auth/"Password based auth": Authenticating with a password is considered the
//!    highest level of authentication, and allows all administrative account actions  (e.g. changing password,
//!    deleting account). However, it does not allow API requests
//! 2. Browser based auth: When logging into pointercrate from a browser, two cookies will be set: `access_token`
//!    and `csrf_token`. The `access_token` cookie should be more accurately names `session_token`, as that's
//!    what it really is. The `csrf_token` allows CSRF prevention using the signed double-submit cookie pattern,
//!    which is submitted on each request in the `X-CSRF-TOKEN`` header.
//!    Different requests are authenticated in different ways:
//!    - `GET`: GET requests are authenticated using only the session token. This is because GET requests are not
//!      mutating, and because CSRF attacks cannot read out the response to requests they make, meaning GET endpoints
//!      are not targettable. Additionally, we cannot set the X-CSRF-TOKEN header on GET requests that are triggered
//!      by top-level browser navigation.
//!    - non-`GET`: These are authenticated using the session token, which is validated using the csrf token.
//!      Browser-based auth allows both administrative account actions (except changing password) and API access
//! 3. HTTP Bearer Auth: Authenticating using a bearer token allows API access, but does not allow user account actions.
//!
//! See [`AuthenticatedUser`] for implementation details.

pub use self::patch::PatchMe;
use crate::{config, error::Result, User};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use legacy::LegacyAuthenticatedUser;
use pointercrate_core::{error::CoreError, util::csprng_u64};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod delete;
mod get;
pub mod legacy;
pub mod oauth;
mod patch;
mod post;

/// Indicates that no authentication has occurred yet
///
/// See also [`AuthenticatedUser::by_id`] and [`AuthenticatedUser::by_name`].
pub struct NoAuth;

/// Indicates that authentication using a bearer token from an `Authorization` header has taken place
///
/// See also [`AuthenticatedUser::validate_api_access`]
pub struct ApiToken;

/// Indicates that authentication using an `access_token` Cookie occurred
///
/// Until a matching CSRF token has been validated, this form of authentication only
/// allows non-mutating GET requests.
///
/// See also [`AuthenticatedUser::validate_cookie_claims`]
pub struct NonMutating(AccessClaims);

/// Indicates "full" authentication. Either a password/username pair passed via HTTP basic auth,
/// or an access token/csrf token pair pass via `access_token` cookie and `X-CSRF-TOKEN` header were validated.
///
/// See also [`AuthenticatedUser::validate_csrf_token`] and [`AuthenticatedUser::verify_password`]
pub struct PasswordOrBrowser(bool);

impl PasswordOrBrowser {
    pub fn is_password(&self) -> bool {
        self.0
    }
}

/// Represents an authenticated user that made a request to the server
///
/// The `Auth` parameter determines "how authenticated" we are, and we use the type
/// system to encode the authentication state-machine, to ensure statically that no unauthorized
/// actions can be performed.
///
/// See also [`NoAuth`], [`ApiToken`], [`NonMutating`] and [`PasswordOrBrowser`]
pub struct AuthenticatedUser<Auth> {
    gen: i64,
    auth_type: AuthenticationType,
    auth_artifact: Auth,
}

pub enum AuthenticationType {
    Legacy(LegacyAuthenticatedUser),
    Oauth2(oauth::OA2AuthenticatedUser),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)] // to prevent a CSRF token to work as an access token
pub struct AccessClaims {
    /// The id of the pointercrate account this token authenticates
    ///
    /// Stored as a string as that's a requirement of [`jsonwebtoken`]'s validation facilities
    sub: String,

    /// An optional session ID.
    ///
    /// Access tokens without associated session IDs cannot be used in the web interface. The
    /// session ID is found again in the CSRF token, to ensure CSRF tokens regenerate with
    /// each new session.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    session_uuid: Option<u64>,

    /// A generation ID that allows token invalidation. Tokens are only accepted if the generation
    /// ID in the token matches the current generation id on the [`AuthenticatedUser`]
    gen: i64,
}

impl AccessClaims {
    pub fn decode(access_token: &str) -> Result<Self> {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.set_required_spec_claims(&["sub"]);

        decode_jwt(access_token, &validation)
    }

    pub fn id(&self) -> Result<i32> {
        self.sub.parse().map_err(|_| CoreError::Unauthorized.into())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct CSRFClaims {
    sub: String, // we're using the jsonwebtoken library's validation to check this field, which expect it to be a string
    exp: u64,
    session: u64,
}

/// Generates a JWT from the given claims, signed with this servers key, [`config::secret`].
pub fn generate_jwt<C: Serialize>(claims: &C) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(&config::secret()),
    )
    .unwrap()
}

pub fn decode_jwt<C: DeserializeOwned>(jwt: &str, validation: &Validation) -> Result<C> {
    jsonwebtoken::decode::<C>(jwt, &DecodingKey::from_secret(&config::secret()), validation)
        .map_err(|_| CoreError::Unauthorized.into())
        .map(|token_data| token_data.claims)
}

impl<Auth> AuthenticatedUser<Auth> {
    pub fn is_legacy(&self) -> bool {
        matches!(self.auth_type, AuthenticationType::Legacy(_))
    }

    pub fn into_user(self) -> User {
        match self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.into_user(),
            AuthenticationType::Oauth2(oauth) => oauth.into_user(),
        }
    }

    pub fn user(&self) -> &User {
        match &self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.user(),
            AuthenticationType::Oauth2(oauth) => oauth.user(),
        }
    }

    pub fn validate_jwt<C: DeserializeOwned>(&self, jwt: &str, mut validation: Validation) -> Result<C> {
        validation.sub = Some(self.user().id.to_string());
        validation.required_spec_claims.insert("sub".to_string());

        decode_jwt(jwt, &validation)
    }

    pub fn auth_type(&self) -> &AuthenticationType {
        &self.auth_type
    }
}

impl AuthenticatedUser<NoAuth> {
    pub fn validate_api_access(self, claims: AccessClaims) -> Result<AuthenticatedUser<ApiToken>> {
        if claims.gen != self.gen || claims.id()? != self.user().id || claims.session_uuid.is_some() {
            Err(CoreError::Unauthorized.into())
        } else {
            Ok(AuthenticatedUser {
                gen: self.gen,
                auth_type: self.auth_type,
                auth_artifact: ApiToken,
            })
        }
    }

    pub fn validate_cookie_claims(self, claims: AccessClaims) -> Result<AuthenticatedUser<NonMutating>> {
        if claims.gen != self.gen || claims.id()? != self.user().id || claims.session_uuid.is_none() {
            Err(CoreError::Unauthorized.into())
        } else {
            Ok(AuthenticatedUser {
                gen: self.gen,
                auth_type: self.auth_type,
                auth_artifact: NonMutating(claims),
            })
        }
    }

    pub fn verify_password(self, password: &str) -> Result<AuthenticatedUser<PasswordOrBrowser>> {
        match &self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.verify(password)?,
            _ => return Err(CoreError::Unauthorized.into()),
        }

        Ok(AuthenticatedUser {
            auth_type: self.auth_type,
            gen: self.gen,
            auth_artifact: PasswordOrBrowser(true),
        })
    }
}

impl AuthenticatedUser<NonMutating> {
    pub fn validate_csrf_token(self, csrf_token: &str) -> Result<AuthenticatedUser<PasswordOrBrowser>> {
        let csrf_claims = self.validate_jwt::<CSRFClaims>(csrf_token, Validation::default())?;

        match self.auth_artifact.0.session_uuid {
            Some(session_uuid) if csrf_claims.session == session_uuid => Ok(AuthenticatedUser {
                gen: self.gen,
                auth_type: self.auth_type,
                auth_artifact: PasswordOrBrowser(false),
            }),
            _ => Err(CoreError::Unauthorized.into()),
        }
    }
}

impl AuthenticatedUser<PasswordOrBrowser> {
    pub fn generate_token_pair(&self) -> Result<(String, String)> {
        let session_uuid = csprng_u64()?;
        let csrf_exp = (SystemTime::now() + Duration::from_secs(7 * 24 * 3600))
            .duration_since(UNIX_EPOCH)
            .expect("7 days in the future is earlier than the Unix Epoch. Wtf?")
            .as_secs();

        let access_claims = AccessClaims {
            sub: self.user().id.to_string(),
            session_uuid: Some(session_uuid),
            gen: self.gen,
        };

        let csrf_claims = CSRFClaims {
            sub: self.user().id.to_string(),
            exp: csrf_exp,
            session: session_uuid,
        };

        let access_token = generate_jwt(&access_claims);
        let csrf_token = generate_jwt(&csrf_claims);

        Ok((access_token, csrf_token))
    }

    /// Generates an access token that can be used for programmatic access to the pointercrate API.
    ///
    /// These tokens are not tied to a user session, and as such cannot be used for administrative
    /// user account actions.
    pub fn generate_programmatic_access_token(&self) -> String {
        generate_jwt(&AccessClaims {
            sub: self.user().id.to_string(),
            session_uuid: None,
            gen: self.gen,
        })
    }

    pub fn downgrade_auth_type(self) -> Result<AuthenticatedUser<ApiToken>> {
        if self.auth_artifact.0 {
            Err(CoreError::Unauthorized.into())
        } else {
            Ok(AuthenticatedUser {
                gen: self.gen,
                auth_type: self.auth_type,
                auth_artifact: ApiToken,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{AccessClaims, AuthenticatedUser, User};

    use super::{AuthenticationType, NoAuth};

    fn make_patrick() -> AuthenticatedUser<NoAuth> {
        AuthenticatedUser {
            auth_type: AuthenticationType::legacy(
                User {
                    id: 0,
                    name: "Patrick".to_string(),
                    permissions: 0,
                    display_name: None,
                    youtube_channel: None,
                },
                bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap(),
            ),
            gen: 0,
            auth_artifact: NoAuth,
        }
    }

    fn make_jacob() -> AuthenticatedUser<NoAuth> {
        AuthenticatedUser {
            auth_type: AuthenticationType::legacy(
                User {
                    id: 1,
                    name: "".to_string(),
                    permissions: 0,
                    display_name: None,
                    youtube_channel: None,
                },
                bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap(),
            ),
            gen: 0,
            auth_artifact: NoAuth,
        }
    }

    #[test]
    fn test_password() {
        assert!(make_patrick().verify_password("bad password").is_ok());
        assert!(make_patrick().verify_password("lksafdölksad").is_err());
        assert!(make_patrick().verify_password("bad password with suffix").is_err());
    }

    #[test]
    fn test_token_pair() {
        let (patricks_access_token, patricks_csrf_token) = make_patrick()
            .verify_password("bad password")
            .unwrap()
            .generate_token_pair()
            .unwrap();

        // doing the authentication the intended way works
        let patricks_access_claims = AccessClaims::decode(&patricks_access_token).unwrap();
        let user = make_patrick().validate_cookie_claims(patricks_access_claims).unwrap();
        let patrick = user.validate_csrf_token(&patricks_csrf_token).unwrap();
        assert!(patrick.downgrade_auth_type().is_ok());

        // make sure only the correct user can decode them
        let patricks_access_claims = AccessClaims::decode(&patricks_access_token).unwrap();
        assert!(make_jacob().validate_cookie_claims(patricks_access_claims).is_err());

        // cookie tokens don't work for ApiToken authentication
        let patricks_access_claims = AccessClaims::decode(&patricks_access_token).unwrap();
        assert!(make_patrick().validate_api_access(patricks_access_claims).is_err());

        // cannot decode csrf token as access token
        assert!(AccessClaims::decode(&patricks_csrf_token).is_err());
    }

    #[test]
    fn test_cannot_transfer_csrf_tokens_across_sessions() {
        let patrick = make_patrick().verify_password("bad password").unwrap();

        let (access_token, _) = patrick.generate_token_pair().unwrap();
        let (_, csrf_token) = patrick.generate_token_pair().unwrap();

        assert!(make_patrick()
            .validate_cookie_claims(AccessClaims::decode(&access_token).unwrap())
            .unwrap()
            .validate_csrf_token(&csrf_token)
            .is_err());
    }

    #[test]
    fn test_cannot_use_programmatic_token_with_csrf_token() {
        let patrick = make_patrick().verify_password("bad password").unwrap();

        let access_token = patrick.generate_programmatic_access_token();

        assert!(make_patrick()
            .validate_cookie_claims(AccessClaims::decode(&access_token).unwrap())
            .is_err());
    }

    #[test]
    fn test_programmatic_access_token() {
        let patrick = make_patrick().verify_password("bad password").unwrap();

        let patricks_access_token = patrick.generate_programmatic_access_token();

        assert!(make_patrick()
            .validate_api_access(AccessClaims::decode(&patricks_access_token).unwrap())
            .is_ok());
        assert!(make_jacob()
            .validate_api_access(AccessClaims::decode(&patricks_access_token).unwrap())
            .is_err());
    }

    #[test]
    fn test_generation_id_change_invalidates_tokens() {
        let access_token = make_patrick()
            .verify_password("bad password")
            .unwrap()
            .generate_programmatic_access_token();
        let mut patrick = make_patrick();
        patrick.gen = 1;
        assert!(patrick.validate_api_access(AccessClaims::decode(&access_token).unwrap()).is_err());

        let (access_token, _) = make_patrick()
            .verify_password("bad password")
            .unwrap()
            .generate_token_pair()
            .unwrap();
        let mut patrick = make_patrick();
        patrick.gen = 1;
        assert!(patrick
            .validate_cookie_claims(AccessClaims::decode(&access_token).unwrap())
            .is_err());
    }
}

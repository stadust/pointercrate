use base64::{engine::general_purpose::STANDARD, Engine};
use log::warn;
use pointercrate_core::{
    error::CoreError,
    permission::{Permission, PermissionsManager},
    pool::{audit_connection, PointercratePool},
};
use pointercrate_core_api::error::IntoOutcome2;
use pointercrate_core_api::{tryo_result, tryo_state};
use pointercrate_user::{
    auth::{AccessClaims, ApiToken, AuthenticatedUser, NonMutating, PasswordOrBrowser},
    error::UserError,
};
use rocket::{
    http::{Method, Status},
    request::{FromRequest, Outcome},
    Request,
};
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;

#[allow(non_upper_case_globals)]
pub struct Auth<A> {
    pub user: AuthenticatedUser<A>,
    pub connection: Transaction<'static, Postgres>,
    pub permissions: PermissionsManager,
}

impl<A> Auth<A> {
    pub async fn commit(self) -> Result<(), UserError> {
        self.connection.commit().await.map_err(UserError::from)
    }

    pub fn require_permission(&self, permission: Permission) -> Result<(), UserError> {
        self.permissions.require_permission(self.user.user().permissions, permission)?;

        Ok(())
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.require_permission(permission).is_ok()
    }

    pub fn assignable_permissions(&self) -> HashSet<Permission> {
        self.permissions.assignable_by_bits(self.user.user().permissions)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<NonMutating> {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(access_token) = request.cookies().get("access_token") else {
            return Outcome::Forward(Status::Unauthorized);
        };

        let pool = tryo_state!(request, PointercratePool);
        let permission_manager = tryo_state!(request, PermissionsManager).clone();

        let mut connection = tryo_result!(pool.transaction().await);

        let access_claims = tryo_result!(AccessClaims::decode(access_token.value()));
        let user = tryo_result!(AuthenticatedUser::by_id(tryo_result!(access_claims.id()), &mut connection).await);
        let authenticated_for_get = tryo_result!(user.validate_cookie_claims(access_claims));

        tryo_result!(audit_connection(&mut connection, authenticated_for_get.user().id).await);

        Outcome::Success(Auth {
            user: authenticated_for_get,
            connection,
            permissions: permission_manager,
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<ApiToken> {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // No auth header set, forward to the request handler that doesnt require authorization (if one exists)
        if request.headers().get_one("Authorization").is_none() && request.cookies().get("access_token").is_none() {
            return Outcome::Forward(Status::Unauthorized);
        }

        let pool = tryo_state!(request, PointercratePool);
        let permission_manager = tryo_state!(request, PermissionsManager).clone();

        let mut connection = tryo_result!(pool.transaction().await);

        for authorization in request.headers().get("Authorization") {
            if let ["Bearer", token] = authorization.split(' ').collect::<Vec<_>>()[..] {
                let access_claims = tryo_result!(AccessClaims::decode(token));
                let user = tryo_result!(AuthenticatedUser::by_id(tryo_result!(access_claims.id()), &mut connection).await);
                let authenticated_user = tryo_result!(user.validate_api_access(access_claims));

                tryo_result!(audit_connection(&mut connection, authenticated_user.user().id).await);

                return Outcome::Success(Auth {
                    user: authenticated_user,
                    connection,
                    permissions: permission_manager,
                });
            }
        }

        // no matching auth header, lets try the cookie
        if let (Some(access_token), Some(csrf_token)) = (request.cookies().get("access_token"), request.headers().get_one("X-CSRF-TOKEN")) {
            let access_claims = tryo_result!(AccessClaims::decode(access_token.value()));
            let user = tryo_result!(AuthenticatedUser::by_id(tryo_result!(access_claims.id()), &mut connection).await);
            let authenticated_for_get = tryo_result!(user.validate_cookie_claims(access_claims));
            let authenticated = tryo_result!(authenticated_for_get.validate_csrf_token(csrf_token));

            tryo_result!(audit_connection(&mut connection, authenticated.user().id).await);

            return Outcome::Success(Auth {
                user: authenticated.downgrade_auth_type().unwrap(), // cannot fail: we are not password authenticated
                connection,
                permissions: permission_manager,
            });
        }

        Outcome::Forward(Status::Unauthorized)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<PasswordOrBrowser> {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if request.method() == Method::Get {
            return CoreError::internal_server_error("Requiring higher authentication on a GET request. This is nonsense").into_outcome();
        }

        // No auth header set, forward to the request handler that doesnt require authorization (if one exists)
        if request.headers().get_one("Authorization").is_none() && request.cookies().get("access_token").is_none() {
            return Outcome::Forward(Status::Unauthorized);
        }

        let pool = tryo_state!(request, PointercratePool);
        let permission_manager = tryo_state!(request, PermissionsManager).clone();

        let mut connection = tryo_result!(pool.transaction().await);

        for authorization in request.headers().get("Authorization") {
            if let ["Basic", basic_auth] = authorization.split(' ').collect::<Vec<_>>()[..] {
                let decoded = tryo_result!(STANDARD
                    .decode(basic_auth)
                    .map_err(|_| ())
                    .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
                    .map_err(|_| {
                        warn!("Malformed 'Authorization' header");

                        CoreError::InvalidHeaderValue { header: "Authorization" }
                    }));

                if let [username, password] = &decoded.splitn(2, ':').collect::<Vec<_>>()[..] {
                    let user = tryo_result!(AuthenticatedUser::by_name(username, &mut connection).await);
                    let authenticated = tryo_result!(user.verify_password(password));

                    tryo_result!(audit_connection(&mut connection, authenticated.user().id).await);

                    return Outcome::Success(Auth {
                        user: authenticated,
                        connection,
                        permissions: permission_manager,
                    });
                }
            }
        }
        // no matching auth header, lets try the cookie
        if let (Some(access_token), Some(csrf_token)) = (request.cookies().get("access_token"), request.headers().get_one("X-CSRF-TOKEN")) {
            let access_claims = tryo_result!(AccessClaims::decode(access_token.value()));
            let user = tryo_result!(AuthenticatedUser::by_id(tryo_result!(access_claims.id()), &mut connection).await);
            let authenticated_for_get = tryo_result!(user.validate_cookie_claims(access_claims));
            let authenticated = tryo_result!(authenticated_for_get.validate_csrf_token(csrf_token));

            tryo_result!(audit_connection(&mut connection, authenticated.user().id).await);

            return Outcome::Success(Auth {
                user: authenticated,
                connection,
                permissions: permission_manager,
            });
        }

        Outcome::Forward(Status::Unauthorized)
    }
}

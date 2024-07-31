use base64::{engine::general_purpose::STANDARD, Engine};
use log::{debug, warn};
use pointercrate_core::{
    error::{CoreError, PointercrateError},
    permission::{Permission, PermissionsManager},
    pool::{audit_connection, PointercratePool},
};
use pointercrate_user::{error::UserError, AuthenticatedUser};
use rocket::{
    http::{Method, Status},
    request::{FromRequest, Outcome},
    Request, State,
};
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;

#[allow(non_upper_case_globals)]
pub struct Auth<const IsToken: bool> {
    pub user: AuthenticatedUser,
    pub connection: Transaction<'static, Postgres>,
    pub permissions: PermissionsManager,

    /* The secret, either token or password */
    pub(crate) secret: String,
}

#[allow(non_upper_case_globals)]
impl<const IsToken: bool> Auth<IsToken> {
    pub async fn commit(self) -> Result<(), UserError> {
        self.connection.commit().await.map_err(UserError::from)
    }

    pub fn require_permission(&self, permission: Permission) -> Result<(), UserError> {
        self.permissions.require_permission(self.user.inner().permissions, permission)?;

        Ok(())
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.require_permission(permission).is_ok()
    }

    pub fn assignable_permissions(&self) -> HashSet<Permission> {
        self.permissions.assignable_by_bits(self.user.inner().permissions)
    }
}

pub type BasicAuth = Auth<false>;
pub type TokenAuth = Auth<true>;

macro_rules! try_outcome {
    ($outcome:expr) => {
        match $outcome {
            Ok(success) => success,
            Err(error) => return Outcome::Error((Status::from_code(error.status_code()).unwrap(), error.into())),
        }
    };
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<true> {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // No auth header set, forward to the request handler that doesnt require authorization (if one exists)
        if request.headers().get_one("Authorization").is_none() && request.cookies().get("access_token").is_none() {
            return Outcome::Error((Status::NotFound, CoreError::NotFound.into()));
        }

        let pool = request.guard::<&State<PointercratePool>>().await;
        let permission_manager = match request.guard::<&State<PermissionsManager>>().await {
            Outcome::Success(manager) => manager.inner().clone(),
            Outcome::Error(err) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    CoreError::internal_server_error(format!("PermissionsManager not retrievable from rocket state: {:?}", err)).into(),
                ))
            },
            Outcome::Forward(_) => unreachable!(), // by impl FromRequest for State
        };

        let mut connection = match pool {
            Outcome::Success(pool) => try_outcome!(pool.transaction().await),
            Outcome::Error(err) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    CoreError::internal_server_error(format!("PointercratePool not retrievable from rocket state: {:?}", err)).into(),
                ));
            },
            Outcome::Forward(_) => unreachable!(), // by impl FromRequest for State
        };

        for authorization in request.headers().get("Authorization") {
            if let ["Bearer", token] = authorization.split(' ').collect::<Vec<_>>()[..] {
                let user = try_outcome!(AuthenticatedUser::token_auth(token, None, &mut *connection).await);

                try_outcome!(audit_connection(&mut *connection, user.inner().id).await);

                return Outcome::Success(Auth {
                    user,
                    connection,
                    permissions: permission_manager,
                    secret: token.to_string(),
                });
            }
        }

        // no matching auth header, lets try the cookie
        if let Some(access_token) = request.cookies().get("access_token") {
            let access_token = access_token.value();

            if request.method() == Method::Get {
                debug!("GET request, the cookie is enough");

                let user = try_outcome!(AuthenticatedUser::token_auth(access_token, None, &mut *connection).await);

                try_outcome!(audit_connection(&mut *connection, user.inner().id).await);

                return Outcome::Success(Auth {
                    user,
                    connection,
                    permissions: permission_manager,
                    secret: access_token.to_string(),
                });
            }

            debug!("Non-GET request, testing X-CSRF-TOKEN header");
            // if we're doing cookie based authorization, there needs to be a X-CSRF-TOKEN
            // header set, unless we're in GET requests, in which case everything is fine
            // :tm:

            if let Some(csrf_token) = request.headers().get_one("X-CSRF-TOKEN") {
                let user = try_outcome!(AuthenticatedUser::token_auth(access_token, Some(csrf_token), &mut *connection).await);

                try_outcome!(audit_connection(&mut *connection, user.inner().id).await);

                return Outcome::Success(Auth {
                    user,
                    connection,
                    permissions: permission_manager,
                    secret: access_token.to_string(),
                });
            } else {
                warn!("Cookie based authentication was used, but no CSRF-token was provided. This might be a CSRF attack!");
            }
        }

        Outcome::Error((Status::Unauthorized, CoreError::Unauthorized.into()))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<false> {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // No auth header set, forward to the request handler that doesnt require authorization (if one exists)
        if request.headers().get_one("Authorization").is_none() {
            return Outcome::Forward(Status::NotFound);
        }

        let pool = request.guard::<&State<PointercratePool>>().await;
        let permission_manager = match request.guard::<&State<PermissionsManager>>().await {
            Outcome::Success(manager) => manager.inner().clone(),
            Outcome::Error(err) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    CoreError::internal_server_error(format!("PermissionsManager not retrievable from rocket state: {:?}", err)).into(),
                ))
            },
            Outcome::Forward(_) => unreachable!(), // by impl FromRequest for State
        };

        let mut connection = match pool {
            Outcome::Success(pool) => try_outcome!(pool.transaction().await),
            Outcome::Error(err) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    CoreError::internal_server_error(format!("PointercratePool not retrievable from rocket state: {:?}", err)).into(),
                ));
            },
            Outcome::Forward(_) => unreachable!(), // by impl FromRequest for State
        };

        for authorization in request.headers().get("Authorization") {
            if let ["Basic", basic_auth] = authorization.split(' ').collect::<Vec<_>>()[..] {
                let decoded = try_outcome!(STANDARD
                    .decode(basic_auth)
                    .map_err(|_| ())
                    .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
                    .map_err(|_| {
                        warn!("Malformed 'Authorization' header");

                        CoreError::InvalidHeaderValue { header: "Authorization" }
                    }));

                if let [username, password] = &decoded.splitn(2, ':').collect::<Vec<_>>()[..] {
                    let user = try_outcome!(AuthenticatedUser::basic_auth(username, password, &mut *connection).await);

                    try_outcome!(audit_connection(&mut *connection, user.inner().id).await);

                    return Outcome::Success(Auth {
                        user,
                        connection,
                        permissions: permission_manager,
                        secret: password.to_string(),
                    });
                }
            }
        }

        Outcome::Error((Status::Unauthorized, CoreError::Unauthorized.into()))
    }
}

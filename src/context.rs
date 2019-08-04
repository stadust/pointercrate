use crate::{
    error::PointercrateError,
    middleware::{auth::Me, cond::IfMatch},
    permissions::PermissionsSet,
    ratelimit::{RatelimitScope, Ratelimits},
    Result,
};
use actix_web::HttpRequest;
use diesel::PgConnection;
use ipnetwork::IpNetwork;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Debug)]
pub enum RequestData {
    Internal,
    External {
        ip: IpNetwork,
        user: Option<Me>,
        if_match: Option<IfMatch>,
    },
}

#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub enum RequestContext<'a> {
    Internal(&'a PgConnection),
    External {
        ip: IpNetwork,
        user: Option<&'a Me>,
        if_match: Option<&'a IfMatch>,
        connection: &'a PgConnection,

        /// A [`Ratelimits`] instance thought which the ratelimits for the current pointercrate
        /// instance can be managed.
        ///
        /// ## Why are we taking a normal reference?
        /// [`RequestContext`]  should stay Copy. A [`Ratelimits`] is, while just a collection of
        /// pointers, not copy. It is, however, very easy to clone. Since checking a ratelimit
        /// shouldn't really be a hot code path, it's just fine to clone from this reference to get
        /// an owned [`Ratelimits`] everytime we want to check a ratelimit.
        ratelimits: &'a Ratelimits,
    },
}

impl RequestData {
    pub fn new(ip: IpNetwork) -> Self {
        RequestData::External {
            ip,
            user: None,
            if_match: None,
        }
    }

    pub fn with_user(mut self, me: Me) -> Self {
        if let RequestData::External { ref mut user, .. } = self {
            *user = Some(me);
        }
        self
    }

    pub fn with_if_match(mut self, condition: Option<IfMatch>) -> Self {
        if let RequestData::External {
            ref mut if_match, ..
        } = self
        {
            *if_match = condition;
        }
        self
    }

    pub fn ctx<'a>(
        &'a self,
        connection: &'a PgConnection,
        ratelimits: &'a Ratelimits,
    ) -> RequestContext<'a> {
        match self {
            RequestData::Internal => RequestContext::Internal(connection),
            RequestData::External { ip, user, if_match } =>
                RequestContext::External {
                    ip: *ip,
                    user: user.as_ref(),
                    if_match: if_match.as_ref(),
                    connection,
                    ratelimits,
                },
        }
    }

    pub fn from_request<S>(req: &HttpRequest<S>) -> Self {
        let mut extensions_mut = req.extensions_mut();

        RequestData::External {
            user: None,
            if_match: extensions_mut.remove(),
            ip: extensions_mut.remove().unwrap(),
        }
    }
}

impl<'a> RequestContext<'a> {
    pub fn check_permissions(&self, permissions: PermissionsSet) -> Result<()> {
        if permissions.is_empty() {
            return Ok(())
        }

        match self {
            RequestContext::External { user: None, .. } => Err(PointercrateError::Unauthorized),
            RequestContext::External {
                user: Some(user), ..
            } if !user.0.has_any(&permissions) =>
                Err(PointercrateError::MissingPermissions {
                    required: permissions,
                }),
            _ => Ok(()),
        }
    }

    pub fn is_list_mod(&self) -> bool {
        match self {
            RequestContext::Internal(_) => true,
            RequestContext::External {
                user: Some(Me(ref user)),
                ..
            } => user.list_team_member(),
            _ => false,
        }
    }

    pub fn check_if_match<H: Hash>(&self, h: &H) -> Result<()> {
        match self {
            RequestContext::External {
                if_match: Some(if_match),
                ..
            } => {
                let mut hasher = DefaultHasher::new();
                h.hash(&mut hasher);

                if if_match.met(hasher.finish()) {
                    Ok(())
                } else {
                    Err(PointercrateError::PreconditionFailed)
                }
            },
            RequestContext::External { if_match: None, .. } =>
                Err(PointercrateError::PreconditionRequired),
            _ => Ok(()),
        }
    }

    pub fn connection(&self) -> &PgConnection {
        match self {
            RequestContext::Internal(connection) => connection,
            RequestContext::External { connection, .. } => connection,
        }
    }

    pub fn ratelimit(&self, bucket: RatelimitScope) -> Result<()> {
        if let RequestContext::External { ratelimits, ip, .. } = self {
            ratelimits.check(bucket, *ip).map_err(|remaining| {
                PointercrateError::Ratelimited {
                    scope: bucket,
                    remaining,
                }
            })
        } else {
            Ok(())
        }
    }
}

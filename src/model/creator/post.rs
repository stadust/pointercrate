use super::Creator;
use crate::{
    citext::{CiStr, CiString},
    context::RequestContext,
    error::PointercrateError,
    model::{Demon, EmbeddedPlayer},
    operation::{Get, Post},
    permissions::PermissionsSet,
    schema::creators,
    Result,
};
use diesel::{insert_into, Connection, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostCreator {
    pub creator: CiString,
}

#[derive(Debug, Insertable)]
#[table_name = "creators"]
struct NewCreator<'a> {
    demon: &'a CiStr,
    creator: i32,
}

impl<'a> Post<(&'a CiStr, &'a CiStr)> for Creator {
    fn create_from(
        (demon, player): (&'a CiStr, &'a CiStr), ctx: RequestContext, connection: &PgConnection,
    ) -> Result<Creator> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        info!("Adding '{}' as creator of demon '{}'", player, demon);

        connection.transaction(|| {
            let demon = Demon::get(demon, ctx, connection)?;
            let player = EmbeddedPlayer::get(player, ctx, connection)?;

            insert_into(creators::table)
                .values(&NewCreator {
                    demon: demon.name.as_ref(),
                    creator: player.id,
                })
                .get_result(connection)
                .map_err(PointercrateError::database)
        })
    }
}

impl Post<(CiString, CiString)> for Creator {
    fn create_from(
        (demon, player): (CiString, CiString), ctx: RequestContext, connection: &PgConnection,
    ) -> Result<Creator> {
        Creator::create_from((demon.as_ref(), player.as_ref()), ctx, connection)
    }
}

// FIXME: this impl is stuuuupid
impl<'a> Post<(i16, &'a CiStr)> for Creator {
    fn create_from(
        (position, player): (i16, &'a CiStr), ctx: RequestContext, connection: &PgConnection,
    ) -> Result<Self> {
        let demon = Demon::get(position, ctx, connection)?;

        Creator::create_from((demon.name.as_ref(), player), ctx, connection)
    }
}

impl Post<(i16, CiString)> for Creator {
    fn create_from(
        (position, player): (i16, CiString), ctx: RequestContext, connection: &PgConnection,
    ) -> Result<Self> {
        Creator::create_from((position, player.as_ref()), ctx, connection)
    }
}

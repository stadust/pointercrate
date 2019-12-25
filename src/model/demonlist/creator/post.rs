use super::Creator;
use crate::{
    citext::CiString,
    context::RequestContext,
    error::PointercrateError,
    model::demonlist::{DatabasePlayer, Demon},
    operation::{Get, Post},
    schema::creators,
    Result,
};
use diesel::{insert_into, Connection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostCreator {
    pub creator: CiString,
}

#[derive(Debug, Insertable)]
#[table_name = "creators"]
struct NewCreator {
    demon: i32,
    creator: i32,
}

impl Post<(i16, CiString)> for Creator {
    fn create_from((position, player): (i16, CiString), ctx: RequestContext) -> Result<Self> {
        let demon = Demon::get(position, ctx)?;

        Creator::create_from((demon.id, player), ctx)
        //Creator::create_from((position, player.as_ref()), ctx)
    }
}

impl Post<(i32, CiString)> for Creator {
    fn create_from((demon_id, player): (i32, CiString), ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        info!(
            "Adding '{}' as creator of demon with ID '{}'",
            player, demon_id
        );

        let connection = ctx.connection();

        connection.transaction(|| {
            //let demon = Demon::get(demon, ctx)?;
            let player = DatabasePlayer::get(player.as_ref(), ctx)?;

            insert_into(creators::table)
                .values(&NewCreator {
                    demon: demon_id,
                    creator: player.id,
                })
                .get_result(connection)
                .map_err(PointercrateError::database)
        })
    }
}

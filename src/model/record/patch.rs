use super::{Record, RecordStatus};
use crate::{
    citext::{CiStr, CiString},
    error::PointercrateError,
    model::{
        demon::{Demon, EmbeddedDemon},
        player::Player,
        Model,
    },
    operation::{deserialize_non_optional, deserialize_optional, Get, Patch},
    permissions::PermissionsSet,
    schema::records,
    Result,
};
use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchRecord {
        progress: i16,
        video: Option<String>,
        status: RecordStatus,
        player: CiString,
        demon: CiString,
    }
}

impl Patch<PatchRecord> for Record {
    fn patch(mut self, mut patch: PatchRecord, connection: &PgConnection) -> Result<Self> {
        info!("Patching record {} with {}", self, patch);

        validate_nullable!(patch: Record::validate_video[video]);

        let demon = Demon::get(
            match patch.demon {
                None => self.demon.name.as_ref(),
                Some(ref demon) => demon.as_ref(),
            },
            connection,
        )?;
        let progress = patch.progress.unwrap_or(self.progress);

        if progress > 100 || progress < demon.requirement {
            return Err(PointercrateError::InvalidProgress {
                requirement: demon.requirement,
            })?
        }

        let map = move |_| {
            EmbeddedDemon {
                name: demon.name,
                position: demon.position,
            }
        };
        let map2 = |name: &CiStr| Player::get(name, connection);

        map_patch!(self, patch: map => demon);
        try_map_patch!(self, patch: map2 => player);
        patch!(self, patch: progress, video, status);

        connection.transaction(move || {
            // If there is a record that would validate the unique (status_, demon, player),
            // with higher progress than this one, this query would find it
            let max_progress: Option<i16> = Record::all()
                .filter(records::player.eq(&self.player.id))
                .filter(records::demon.eq(&self.demon.name))
                .filter(records::status_.eq(&self.status))
                .filter(records::id.ne(&self.id))
                .select(diesel::dsl::max(records::progress))
                .get_result::<Option<i16>>(connection)?;

            if let Some(max_progress) = max_progress {
                if max_progress > self.progress {
                    // We simply make `self` the same as that record, causing it to later get deleted
                    let record = Record::all()
                        .filter(records::player.eq(&self.player.id))
                        .filter(records::demon.eq(&self.demon.name))
                        .filter(records::status_.eq(&self.status))
                        .filter(records::progress.eq(&max_progress))
                        .get_result::<Record>(connection)?;

                    self.video = record.video;
                    self.progress = record.progress;
                }
            }

            // By now, our record is for sure the one with the highest progress - all others can be deleted
            diesel::sql_query(format!(
                "DELETE FROM records WHERE player = '{0}' AND demon = '{1}' AND (status_ = '{2}' OR '{2}' = 'approved') AND progress <= {3} AND id <> {4}",
                self.player.id, self.demon.name, self.status.to_string().to_uppercase(), self.progress, self.id
            )).execute(connection)?;

            diesel::update(records::table)
                .filter(records::id.eq(&self.id))
                .set((
                    records::progress.eq(&self.progress),
                    records::video.eq(&self.video),
                    records::status_.eq(&self.status),
                    records::player.eq(&self.player.id),
                    records::demon.eq(&self.demon.name),
                ))
                .execute(connection)?;

                Ok(self)
        })
    }

    fn permissions_for(&self, _: &PatchRecord) -> PermissionsSet {
        perms!(ListHelper or ListModerator or ListAdministrator)
    }
}

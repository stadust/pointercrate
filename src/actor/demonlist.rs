use crate::{
    actor::{
        database::{DatabaseActor, DeleteMessage},
        http::HttpActor,
    },
    context::{RequestContext, RequestData},
    model::{
        demonlist::{demon::PartialDemon, Record},
        nationality::Nationality,
        Model,
    },
    operation::Get,
    permissions::Permissions,
    view::demonlist::DemonlistOverview,
    Result,
};
use actix::{AsyncContext, Handler, Message, WrapFuture};
use diesel::{QueryDsl, RunQueryDsl};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio::prelude::Future;

#[derive(Debug, Copy, Clone)]
pub struct GetDemonlistOverview;

impl Message for GetDemonlistOverview {
    type Result = Result<DemonlistOverview>;
}

impl Handler<GetDemonlistOverview> for DatabaseActor {
    type Result = Result<DemonlistOverview>;

    fn handle(&mut self, _: GetDemonlistOverview, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.connection()?;
        let (admins, mods, helpers) = Get::get(
            (
                Permissions::ListAdministrator,
                Permissions::ListModerator,
                Permissions::ListHelper,
            ),
            RequestContext::Internal(connection),
        )?;
        let all_demons = PartialDemon::all()
            .order_by(crate::schema::demons::position)
            .load(connection)?;
        let nations = Nationality::all()
            .order_by(crate::schema::nationalities::iso_country_code)
            .load(connection)?;

        Ok(DemonlistOverview {
            demon_overview: all_demons,
            admins,
            mods,
            helpers,
            nations,
        })
    }
}
#[derive(Debug)]
pub struct PostProcessRecord(pub Option<Record>);

impl Message for PostProcessRecord {
    type Result = Option<Record>;
}

impl Handler<PostProcessRecord> for HttpActor {
    type Result = Option<Record>;

    fn handle(
        &mut self,
        PostProcessRecord(record): PostProcessRecord,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Some(ref record) = record {
            info!("Post processing record {}", record);

            let record_id = record.id;
            let progress = f32::from(record.progress) / 100f32;

            let mut payload = json!({
                "content": format!("**New record submitted! ID: {}**", record_id),
                "embeds": [
                    {
                        "type": "rich",
                        "title": format!("{}% on {}", record.progress, record.demon.name),
                        "description": format!("{} just got {}% on {}! Go add his record!", record.player.name, record.progress, record.demon.name),
                        "footer": {
                            "text": format!("This record has been submitted by submitter #{}", record.submitter.unwrap_or(1))
                        },
                        "color": (0x9e0000 as f32 * progress) as i32 & 0xFF0000 + (0x00e000 as f32 * progress) as i32 & 0x00FF00,
                        "author": {
                            "name": format!("{} (ID: {})", record.player.name, record.player.id),
                            "url": record.video
                        },
                        "thumbnail": {
                            "url": "https://cdn.discordapp.com/avatars/277391246035648512/b03c85d94dc02084c413a7fdbe2cea79.webp?size=1024"
                        },
                    }
                ]
            });

            if let Some(ref video) = record.video {
                // FIXME: this isn't supported by discord. We need to figure out another way then :(
                payload["embeds"][0]["video"] = json! {
                    {"url": video}
                };
                payload["embeds"][0]["fields"] = json! {
                    [{
                        "name": "Video Proof:",
                        "value": video
                    }]
                };
            }

            let deletor = self.deletor.clone();
            let payload_future = self.execute_discord_webhook(payload);

            if let Some(ref video) = record.video {
                debug!("Asynchronously validating video '{}'", video);

                let future = self.if_exists(video).or_else(move |_| {
                    warn!("A HEAD request to video yielded an error response, automatically deleting submission!");

                    deletor
                        .send(DeleteMessage::new(record_id, RequestData::Internal))
                        .map_err(move |error| error!("INTERNAL SERVER ERROR: Failure to delete record {} - {:?}!", record_id, error))
                        .map(|_| ())
                        .and_then(|_| Err(()))
                });

                ctx.spawn(future.and_then(move |_| payload_future).into_actor(self));
            } else {
                ctx.spawn(payload_future.into_actor(self));
            }
        }

        record
    }
}

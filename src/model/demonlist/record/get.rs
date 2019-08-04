use super::{EmbeddedRecordD, EmbeddedRecordP, EmbeddedRecordPD, Record};
use crate::{
    context::RequestContext,
    error::PointercrateError,
    model::{
        demonlist::{demon::Demon, record::RecordStatus, submitter::Submitter},
        By, Model,
    },
    operation::Get,
    schema::{demons, records},
    Result,
};
use diesel::{result::Error, ExpressionMethods, QueryDsl, RunQueryDsl};

impl Get<i32> for Record {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        let mut record: Record = match Record::by(id).first(ctx.connection()) {
            Ok(record) => record,
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Record",
                    identified_by: id.to_string(),
                })?,
            Err(err) => Err(PointercrateError::database(err))?,
        };

        if !ctx.is_list_mod() {
            record.submitter = None;
        }

        if record.status != RecordStatus::Approved {
            ctx.check_permissions(perms!(ListHelper or ListModerator or ListAdministrator))?;
        }

        Ok(record)
    }
}

impl Get<i32> for Vec<EmbeddedRecordD> {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        Ok(
            EmbeddedRecordD::by_player_and_status(id, RecordStatus::Approved)
                .order_by(demons::name)
                .load(ctx.connection())?,
        )
    }
}

impl<'a> Get<&'a Demon> for Vec<EmbeddedRecordP> {
    fn get(demon: &'a Demon, ctx: RequestContext) -> Result<Self> {
        Ok(
            EmbeddedRecordP::by_demon_and_status(demon.name.as_ref(), RecordStatus::Approved)
                .order_by((records::progress.desc(), records::id))
                .load(ctx.connection())?,
        )
    }
}

impl<'a> Get<&'a Submitter> for Vec<EmbeddedRecordPD> {
    fn get(submitter: &'a Submitter, ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        Ok(EmbeddedRecordPD::all()
            .filter(records::submitter.eq(&submitter.id))
            .load(ctx.connection())?)
    }
}

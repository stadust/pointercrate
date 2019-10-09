use super::{EmbeddedRecordPD, MinimalRecordD, MinimalRecordP, Record};
use crate::{
    context::RequestContext,
    error::PointercrateError,
    model::{
        demonlist::{
            demon::Demon,
            record::{records_d, records_p, records_pd, FullRecord, RecordStatus},
            submitter::Submitter,
        },
        By, Model,
    },
    operation::Get,
    Result,
};
use diesel::{result::Error, ExpressionMethods, QueryDsl, RunQueryDsl};

impl Get<i32> for FullRecord {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        let mut record: FullRecord = match FullRecord::by(id).first(ctx.connection()) {
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
            record.notes = None;
        }

        if record.status != RecordStatus::Approved {
            ctx.check_permissions(perms!(ListHelper or ListModerator or ListAdministrator))?;
        }

        Ok(record)
    }
}

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

impl Get<i32> for Vec<MinimalRecordD> {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        MinimalRecordD::all()
            .filter(records_d::player.eq(id))
            .filter(records_d::status_.eq(RecordStatus::Approved))
            .order_by(records_d::demon_name)
            .load(ctx.connection())
            .map_err(Into::into)
    }
}

impl<'a> Get<&'a Demon> for Vec<MinimalRecordP> {
    fn get(demon: &'a Demon, ctx: RequestContext) -> Result<Self> {
        MinimalRecordP::all()
            .filter(records_p::demon.eq(demon.id))
            .filter(records_p::status_.eq(RecordStatus::Approved))
            .order_by((records_p::progress.desc(), records_p::id))
            .load(ctx.connection())
            .map_err(Into::into)
    }
}

impl<'a> Get<&'a Submitter> for Vec<EmbeddedRecordPD> {
    fn get(submitter: &'a Submitter, ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        Ok(EmbeddedRecordPD::all()
            .filter(records_pd::submitter_id.eq(&submitter.id))
            .load(ctx.connection())?)
    }
}

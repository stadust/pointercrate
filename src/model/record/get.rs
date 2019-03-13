use super::{EmbeddedRecordD, EmbeddedRecordP, EmbeddedRecordPD, Record};
use crate::{
    error::PointercrateError,
    model::{demon::Demon, record::RecordStatus, submitter::Submitter, user::User, By, Model},
    operation::Get,
    permissions::{self, AccessRestrictions},
    schema::records,
    Result,
};
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

impl Get<i32> for Record {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match Record::by(id).first(connection) {
            Ok(record) => Ok(record),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Record",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for Vec<EmbeddedRecordD> {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        Ok(EmbeddedRecordD::by_player_and_status(id, RecordStatus::Approved).load(connection)?)
    }
}

impl<'a> Get<&'a Demon> for Vec<EmbeddedRecordP> {
    fn get(demon: &'a Demon, connection: &PgConnection) -> Result<Self> {
        Ok(
            EmbeddedRecordP::by_demon_and_status(demon.name.as_ref(), RecordStatus::Approved)
                .order_by(records::progress.desc())
                .load(connection)?,
        )
    }
}

impl<'a> Get<&'a Submitter> for Vec<EmbeddedRecordPD> {
    fn get(submitter: &'a Submitter, connection: &PgConnection) -> Result<Self> {
        Ok(EmbeddedRecordPD::all()
            .filter(records::submitter.eq(&submitter.id))
            .load(connection)?)
    }
}

impl AccessRestrictions for Record {
    fn access(mut self, user: Option<&User>) -> Result<Self> {
        // Theoretically we disclose information here, as a non-authorized user can figure out which
        // records dont exist and which ones are simply non-approved. Practically, we dont give a
        // shit
        if self.status() != RecordStatus::Approved {
            permissions::demand(
                perms!(ListHelper or ListModerator or ListAdministrator),
                user,
            )?;
        }

        if user.is_none() || !user.unwrap().list_team_member() {
            self.submitter = None;
        }

        Ok(self)
    }

    /*fn pre_page_access(user: Option<&User>) -> Result<()> {
        permissions::demand(
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
            user,
        )
    }*/

    fn page_access(mut page: Vec<Self>, user: Option<&User>) -> Result<Vec<Self>> {
        if user.is_none() || !user.unwrap().list_team_member() {
            page.retain(|record| record.status() == RecordStatus::Approved);

            for record in page.iter_mut() {
                record.submitter = None
            }
        }

        Ok(page)
    }

    fn pre_delete(&self, user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(ListModerator or ListAdministrator), user)
    }
}

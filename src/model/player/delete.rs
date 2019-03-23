/*
impl Delete for EmbeddedPlayer {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        info!("Deleting player {}", self);

        delete(players::table)
            .filter(players::id.eq(self.id))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
*/

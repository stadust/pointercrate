use sqlx::{Connect, PgConnection};

/// Connects to a local test database (called pointercrate_test) with a dummy account (username:
/// pc_test, password: test), starts a transaction in a new connection and inserts some dummy data
/// into it, which can be used by tests
pub async fn test_setup() -> PgConnection {
    let _ = env_logger::try_init();

    let mut connection = PgConnection::connect("postgres://pc_test:test@localhost/pointercrate_test")
        .await
        .unwrap();
    sqlx::query!("BEGIN TRANSACTION").execute(&mut connection).await.unwrap();
    let player_ids = sqlx::query!(
        "INSERT INTO players (name) VALUES ('stardust1971'), ('Aquatias'), ('Mullsy'), ('Samifying'), ('Aeon Air'), ('Aaron Ari') \
         RETURNING id",
    )
    .fetch_all(&mut connection)
    .await
    .unwrap();
    let submitter_id = sqlx::query!("INSERT INTO submitters (ip_address) VALUES ('127.0.0.1'::INET) RETURNING submitter_id")
        .fetch_one(&mut connection)
        .await
        .unwrap()
        .submitter_id;
    let demon_ids = sqlx::query!(
        "INSERT INTO demons (name, position, requirement, verifier, publisher) VALUES ('abstract interpretation', 1, 52, $1, $1), \
         ('Trichotomy', 2, 84, $1, $1), ('terminal void', 3, 35, $2,$2), ('taraturusus', 4, 90, $3, $3) RETURNING id",
        player_ids[0].id,
        player_ids[1].id,
        player_ids[2].id
    )
    .fetch_all(&mut connection)
    .await
    .unwrap();
    let record_ids = sqlx::query!(
        "INSERT INTO records (progress, status_, player, submitter, demon) VALUES (100, 'SUBMITTED', $1, $5, $6), (90, 'APPROVED', $1, \
         $5, $6), (80, 'REJECTED', $2, $5, $6), (90, 'APPROVED', $3, $5, $7), (100, 'APPROVED', $4, $5, $7), (100, 'APPROVED', $4, $5, \
         $8), (100, 'APPROVED', $1, $5, $7) RETURNING id",
        player_ids[0].id,
        player_ids[1].id,
        player_ids[4].id,
        player_ids[5].id,
        submitter_id,
        demon_ids[0].id,
        demon_ids[1].id,
        demon_ids[2].id
    )
    .fetch_all(&mut connection)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO record_notes (record, content) VALUES ($1, 'This is a test')",
        record_ids[2].id
    )
    .execute(&mut connection)
    .await
    .unwrap();

    // password is: password1234567890, see also unit test for registration
    sqlx::query!(
        "INSERT INTO members (name, password_hash) VALUES ('stadust_existing', \
         '$2b$12$4lKrHzdA39hJj0IcGeWCMucV2dOlh26sp.H/PFAYNItgN08.TKUki')"
    )
    .execute(&mut connection)
    .await
    .unwrap();

    connection
}

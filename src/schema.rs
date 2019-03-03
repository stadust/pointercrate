table! {
    creators (demon, creator) {
        demon -> Text,
        creator -> Int4,
    }
}

table! {
    demons (name) {
        name -> Text,
        position -> Int2,
        requirement -> Int2,
        video -> Nullable<Varchar>,
        description -> Nullable<Text>,
        notes -> Nullable<Text>,
        verifier -> Int4,
        publisher -> Int4,
    }
}

table! {
    demon_publisher_verifier_join (vid, pid) {
        vname -> Text,
        vid -> Int4,
        vbanned -> Bool,
        pname -> Text,
        pid -> Int4,
        pbanned -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::bitstring::BitString;

    members (member_id) {
        member_id -> Int4,
        name -> Text,
        display_name -> Nullable<Text>,
        youtube_channel -> Nullable<Varchar>,
        password_hash -> Bytea,
        password_salt -> Bytea,
        permissions -> BitString,
    }
}

table! {
    players (id) {
        id -> Int4,
        name -> Text,
        banned -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;

    records (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        player -> Int4,
        submitter -> Int4,
        demon -> Text,
    }
}

table! {
    submitters (submitter_id) {
        submitter_id -> Int4,
        ip_address -> Inet,
        banned -> Bool,
    }
}

joinable!(creators -> demons (demon));
joinable!(creators -> players (creator));
joinable!(records -> demons (demon));
joinable!(records -> players (player));
joinable!(records -> submitters (submitter));

allow_tables_to_appear_in_same_query!(
    creators,
    demons,
    members,
    players,
    records,
    submitters,
    demon_publisher_verifier_join
);

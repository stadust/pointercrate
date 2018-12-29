table! {
    use diesel::sql_types::*;
    use crate::model::audit::Audit_operation;

    audit_log (id) {
        id -> Int4,
        operation -> Audit_operation,
        target -> Nullable<Varchar>,
        old_value -> Nullable<Varchar>,
        new_value -> Nullable<Varchar>,
        time_ -> Timestamp,
        list_mod -> Nullable<Text>,
        demon -> Nullable<Text>,
        record -> Nullable<Int4>,
        player -> Nullable<Int4>,
    }
}

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
    demon_publisher_verifier_join (pid, vid) {
        pname -> Text,
        pid -> Int4,
        pbanned -> Bool,
        vname -> Text,
        vid -> Int4,
        vbanned -> Bool,
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

joinable!(audit_log -> demons (demon));
joinable!(audit_log -> players (player));
joinable!(audit_log -> records (record));
joinable!(creators -> demons (demon));
joinable!(creators -> players (creator));
joinable!(records -> demons (demon));
joinable!(records -> players (player));
joinable!(records -> submitters (submitter));

allow_tables_to_appear_in_same_query!(
    audit_log,
    creators,
    demons,
    members,
    players,
    records,
    submitters,
    demon_publisher_verifier_join
);

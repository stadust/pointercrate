table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    creators (demon, creator) {
        demon -> CiText,
        creator -> Int4,
    }
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    demons (name) {
        name -> CiText,
        position -> Int2,
        requirement -> Int2,
        video -> Nullable<Varchar>,
        verifier -> Int4,
        publisher -> Int4,
    }
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    demon_verifier_publisher_join (vid, pid) {
        vname -> CiText,
        vid -> Int4,
        vbanned -> Bool,
        pname -> CiText,
        pid -> Int4,
        pbanned -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::bitstring::BitString;
    use crate::citext::CiText;

    members (member_id) {
        member_id -> Int4,
        name -> Text,
        display_name -> Nullable<Text>,
        youtube_channel -> Nullable<Varchar>,
        password_hash -> Bytea,
        permissions -> BitString,
    }
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    players (id) {
        id -> Int4,
        name -> CiText,
        banned -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::CiText;

    records (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        player -> Int4,
        submitter -> Int4,
        demon -> CiText,
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
    demon_verifier_publisher_join
);

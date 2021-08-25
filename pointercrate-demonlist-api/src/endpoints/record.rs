#[rocket::get("/")]
pub async fn paginate() {
    todo!()
}

#[rocket::post("/")]
pub async fn submit() {
    todo!()
}

#[rocket::get("/<record_id>")]
pub async fn get() {
    todo!()
}

#[rocket::get("/<record_id>/audit")]
pub async fn audit() {
    todo!()
}

#[rocket::patch("/<record_id>")]
pub async fn patch() {
    todo!()
}

#[rocket::delete("/<record_id>")]
pub async fn delete() {
    todo!()
}

#[rocket::post("/<record_id>/notes")]
pub async fn add_note() {
    todo!()
}

#[rocket::patch("/<record_id>/notes/<note_id>")]
pub async fn patch_note() {
    todo!()
}

#[rocket::delete("/<record_id>/notes/<note_id>")]
pub async fn delete_note() {
    todo!()
}

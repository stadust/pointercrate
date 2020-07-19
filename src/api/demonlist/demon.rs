use crate::{
    extractor::auth::TokenAuth,
    model::demonlist::demon::{FullDemon, PostDemon},
    permissions::Permissions,
    state::PointercrateState,
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{web::Json, HttpResponse};
use actix_web_codegen::post;

#[post("/")]
pub async fn post(TokenAuth(user): TokenAuth, state: PointercrateState, data: Json<PostDemon>) -> ApiResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListModerator)?;

    let mut connection = state.audited_transaction(&user).await?;

    let demon = FullDemon::create_from(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Created().json_with_etag(&demon))
}

pub mod v1 {
    use crate::{
        extractor::{auth::TokenAuth, if_match::IfMatch},
        model::demonlist::{
            creator::{Creator, PostCreator},
            demon::{Demon, DemonPositionPagination, FullDemon, PatchDemon},
            player::DatabasePlayer,
        },
        permissions::Permissions,
        state::PointercrateState,
        util::HttpResponseBuilderExt,
        ApiResult,
    };
    use actix_web::{
        web::{Json, Path, Query},
        HttpResponse,
    };
    use actix_web_codegen::{delete, get, patch, post};

    #[get("/")]
    pub async fn paginate(state: PointercrateState, mut pagination: Query<DemonPositionPagination>) -> ApiResult<HttpResponse> {
        let mut connection = state.connection().await?;

        let mut demons = pagination.page(&mut connection).await?;
        let max_position = Demon::max_position(&mut connection).await?;

        pagination_response!(
            "/api/v1/demons/",
            demons,
            pagination,
            1,
            max_position,
            before_position,
            after_position,
            base.position
        )
    }

    #[get("/{position}/")]
    pub async fn get(state: PointercrateState, position: Path<i16>) -> ApiResult<HttpResponse> {
        let mut connection = state.connection().await?;

        let demon = FullDemon::by_position(position.into_inner(), &mut connection).await?;

        Ok(HttpResponse::Ok().json_with_etag(&demon))
    }

    #[patch("/{position}/")]
    pub async fn patch(
        TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, patch: Json<PatchDemon>, position: Path<i16>,
    ) -> ApiResult<HttpResponse> {
        user.inner().require_permissions(Permissions::ListModerator)?;

        let mut connection = state.audited_transaction(&user).await?;
        let demon = FullDemon::by_position(position.into_inner(), &mut connection).await?;

        // FIXME(lost updates)

        if_match.require_etag_match(&demon)?;

        let demon = demon.apply_patch(patch.into_inner(), &mut connection).await?;

        connection.commit().await?;

        Ok(HttpResponse::Ok().json_with_etag(&demon))
    }

    #[post("/{position}/creators/")]
    pub async fn post_creator(
        TokenAuth(user): TokenAuth, state: PointercrateState, position: Path<i16>, creator: Json<PostCreator>,
    ) -> ApiResult<HttpResponse> {
        user.inner().require_permissions(Permissions::ListModerator)?;

        let mut connection = state.audited_connection(&user).await?;

        let demon = Demon::by_position(position.into_inner(), &mut connection).await?;
        let player = DatabasePlayer::by_name_or_create(creator.creator.as_ref(), &mut connection).await?;

        Creator::insert(&demon.base, &player, &mut connection).await?;

        Ok(HttpResponse::Created()
            .header(
                "Location",
                format!("/api/v1/demons/{}/creators/{}/", demon.base.position, player.id),
            )
            .finish())
    }

    #[delete("/{position}/creators/{player_id}/")]
    pub async fn delete_creator(TokenAuth(user): TokenAuth, state: PointercrateState, path: Path<(i16, i32)>) -> ApiResult<HttpResponse> {
        user.inner().require_permissions(Permissions::ListModerator)?;

        let mut connection = state.audited_connection(&user).await?;

        let (position, player_id) = path.into_inner();

        let demon = Demon::by_position(position, &mut connection).await?;
        let player = DatabasePlayer::by_id(player_id, &mut connection).await?;

        Creator::get(&demon.base, &player, &mut connection)
            .await?
            .delete(&mut connection)
            .await?;

        Ok(HttpResponse::NoContent().finish())
    }
}

pub mod v2 {
    use crate::{
        extractor::{auth::TokenAuth, if_match::IfMatch},
        model::demonlist::{
            creator::{Creator, PostCreator},
            demon::{Demon, DemonIdPagination, DemonPositionPagination, FullDemon, PatchDemon},
            player::DatabasePlayer,
        },
        permissions::Permissions,
        state::PointercrateState,
        util::HttpResponseBuilderExt,
        ApiResult,
    };
    use actix_web::{
        web::{Json, Path, Query},
        HttpResponse,
    };
    use actix_web_codegen::{delete, get, patch, post};

    #[get("/")]
    pub async fn paginate(state: PointercrateState, mut pagination: Query<DemonIdPagination>) -> ApiResult<HttpResponse> {
        let mut connection = state.connection().await?;

        let mut demons = pagination.page(&mut connection).await?;
        let (max_id, min_id) = Demon::extremal_demon_ids(&mut connection).await?;

        pagination_response!("/api/v2/demons/", demons, pagination, min_id, max_id, before_id, after_id, base.id)
    }

    // Same as /api/v1/demons/
    #[get("/listed/")]
    pub async fn paginate_listed(state: PointercrateState, mut pagination: Query<DemonPositionPagination>) -> ApiResult<HttpResponse> {
        let mut connection = state.connection().await?;

        let mut demons = pagination.page(&mut connection).await?;
        let max_position = Demon::max_position(&mut connection).await?;

        pagination_response!(
            "/api/v2/demons/listed/",
            demons,
            pagination,
            1,
            max_position,
            before_position,
            after_position,
            base.position
        )
    }

    #[get("/{demon_id}/")]
    pub async fn get(state: PointercrateState, id: Path<i32>) -> ApiResult<HttpResponse> {
        let mut connection = state.connection().await?;

        let demon = FullDemon::by_id(id.into_inner(), &mut connection).await?;

        Ok(HttpResponse::Ok().json_with_etag(&demon))
    }

    #[patch("/{demon_id}/")]
    pub async fn patch(
        TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, patch: Json<PatchDemon>, id: Path<i32>,
    ) -> ApiResult<HttpResponse> {
        user.inner().require_permissions(Permissions::ListModerator)?;

        let mut connection = state.audited_transaction(&user).await?;
        let demon = FullDemon::by_id(id.into_inner(), &mut connection).await?;

        // FIXME(lost updates)

        if_match.require_etag_match(&demon)?;

        let demon = demon.apply_patch(patch.into_inner(), &mut connection).await?;

        connection.commit().await?;

        Ok(HttpResponse::Ok().json_with_etag(&demon))
    }

    #[post("/{demon_id}/creators/")]
    pub async fn post_creator(
        TokenAuth(user): TokenAuth, state: PointercrateState, id: Path<i32>, creator: Json<PostCreator>,
    ) -> ApiResult<HttpResponse> {
        user.inner().require_permissions(Permissions::ListModerator)?;

        let mut connection = state.audited_connection(&user).await?;

        let demon = Demon::by_id(id.into_inner(), &mut connection).await?;
        let player = DatabasePlayer::by_name_or_create(creator.creator.as_ref(), &mut connection).await?;

        Creator::insert(&demon.base, &player, &mut connection).await?;

        Ok(HttpResponse::Created()
            .header(
                "Location",
                format!("/api/v1/demons/{}/creators/{}/", demon.base.position, player.id),
            )
            .finish())
    }

    #[delete("/{demon_id}/creators/{player_id}/")]
    pub async fn delete_creator(TokenAuth(user): TokenAuth, state: PointercrateState, path: Path<(i32, i32)>) -> ApiResult<HttpResponse> {
        user.inner().require_permissions(Permissions::ListModerator)?;

        let mut connection = state.audited_connection(&user).await?;

        let (id, player_id) = path.into_inner();

        let demon = Demon::by_id(id, &mut connection).await?;
        let player = DatabasePlayer::by_id(player_id, &mut connection).await?;

        Creator::get(&demon.base, &player, &mut connection)
            .await?
            .delete(&mut connection)
            .await?;

        Ok(HttpResponse::NoContent().finish())
    }
}

use crate::auth::TokenAuth;
use pointercrate_core::{config, permission::PermissionsManager};
use pointercrate_core_api::response::Page;
use pointercrate_user_pages::{
    account::{AccountPage, AccountPageConfig},
    login::LoginPage,
};
use rocket::{response::Redirect, State};

#[rocket::get("/login", rank = 0)]
pub async fn login_page_authorized(auth: TokenAuth) -> Redirect {
    Redirect::to(rocket::uri!(account_page))
}

#[rocket::get("/login", rank = 1)]
pub async fn login_page() -> Page<LoginPage> {
    Page(LoginPage)
}

#[rocket::get("/account", rank = 1)]
pub async fn account_page_unauthorized() -> Redirect {
    Redirect::to(rocket::uri!(login_page))
}

#[rocket::get("/account", rank = 0)]
pub async fn account_page(
    mut auth: TokenAuth, permissions: &State<PermissionsManager>, tabs: &State<AccountPageConfig>,
) -> Page<AccountPage> {
    let csrf_token = auth.user.generate_csrf_token(&config::secret());

    Page(
        tabs.account_page(csrf_token, auth.user.into_inner(), permissions, &mut auth.connection)
            .await,
    )
}

use maud::{html, Markup};
use pointercrate_core::{
    permission::PermissionsManager,
    view::{PageFragment, PreRenderedFragment},
};
use pointercrate_user::{
    sqlx::{PgConnection, Postgres, Transaction},
    User,
};
use std::sync::{Arc, RwLock};

mod profile;
mod users;

#[async_trait::async_trait]
pub trait AccountPageTab {
    fn should_display_for(&self, user: &User, permissions: &PermissionsManager) -> bool;
    fn additional_scripts(&self) -> Vec<String>;

    fn tab(&self) -> Markup;
    async fn page(&self, user: &User, permissions: &PermissionsManager, connection: &mut PgConnection) -> Markup;
}

pub struct AccountPageConfig {
    tabs: Arc<RwLock<Vec<Box<dyn AccountPageTab>>>>,
}

pub struct AccountPage {
    config: AccountPageConfig,
    user: User,
    permissions: PermissionsManager,
    connection: Transaction<'static, Postgres>,
    csrf_token: String,
}

impl AccountPage {
    pub async fn pre_render(&mut self) -> PreRenderedFragment {
        PreRenderedFragment {
            scripts: self.additional_scripts(),
            stylesheets: vec![],
            fragment: self.fragment().await,
        }
    }

    fn additional_scripts(&self) -> Vec<String> {
        self.config
            .tabs
            .read()
            .unwrap()
            .iter()
            .filter(|page| page.should_display_for(&self.user, &self.permissions))
            .flat_map(|page| page.additional_scripts())
            .collect()
    }

    async fn fragment(&mut self) -> Markup {
        let tabs = ;

        let fucking = self.config.tabs.read().unwrap()
            .iter()
            .filter(|tab| tab.should_display_for(&self.user, &self.permissions))
            .enumerate();

        html! {
            span#chicken-salad-red-fish style = "display:none" {(self.csrf_token)}
            div.tab-display#account-tabber {
                div.tab-selection.flex.wrap.m-center.fade style="text-align: center;" {
                    @for (i, page) in tabs.iter().filter(|tab| tab.should_display_for(&self.user, &self.permissions)).enumerate() {
                        @if i == 1 {
                            div.tab.tab-active.button.white.hover.no-shadow data-tab-id=(i) {
                                (page.tab())
                            }
                        }
                        @else {
                            div.tab.button.white.hover.no-shadow data-tab-id=(i) {
                                (page.tab())
                            }
                        }
                    }
                }

                @for (i, page) in tabs.iter().filter(|tab| tab.should_display_for(&self.user, &self.permissions)).enumerate() {
                    @if i == 1 {
                        div.m-center.flex.tab-content.tab-content-active.container data-tab-id = (i){
                            (page.page(&self.user, &self.permissions, &mut self.connection).await)
                        }
                    }
                    @else {
                        div.m-center.flex.tab-content.container data-tab-id = (i){
                            (page.page(&self.user, &self.permissions, &mut self.connection).await)
                        }
                    }
                }
            }
        }
    }
}

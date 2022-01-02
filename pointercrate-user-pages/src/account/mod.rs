use maud::{html, Markup, PreEscaped};
use pointercrate_core::{etag::Taggable, permission::PermissionsManager};
use pointercrate_core_pages::{PageFragment, Script};
use pointercrate_user::{sqlx::PgConnection, User};

pub mod profile;
pub mod users;

#[async_trait::async_trait]
pub trait AccountPageTab {
    fn should_display_for(&self, user: &User, permissions: &PermissionsManager) -> bool;
    fn additional_scripts(&self) -> Vec<Script>;

    fn tab_id(&self) -> u8;
    fn tab(&self) -> Markup;
    async fn content(&self, user: &User, permissions: &PermissionsManager, connection: &mut PgConnection) -> Markup;
}

pub struct AccountPageConfig {
    tabs: Vec<Box<dyn AccountPageTab + Send + Sync + 'static>>,
}

impl AccountPageConfig {
    pub fn new() -> Self {
        AccountPageConfig { tabs: Vec::new() }
    }

    pub fn with_page(mut self, page: impl AccountPageTab + Send + Sync + 'static) -> Self {
        self.tabs.push(Box::new(page));
        self
    }

    pub async fn account_page(
        &self, csrf_token: String, user: User, permissions: &PermissionsManager, connection: &mut PgConnection,
    ) -> AccountPage {
        let mut page = AccountPage {
            user,
            scripts: vec![],
            tabs: vec![],
            csrf_token,
        };

        for tab_config in &self.tabs {
            if tab_config.should_display_for(&page.user, &permissions) {
                let tab = tab_config.tab();
                let content = tab_config.content(&page.user, permissions, connection).await;

                page.scripts.extend(tab_config.additional_scripts());
                page.tabs.push((tab, content, tab_config.tab_id()));
            }
        }

        page
    }
}

pub struct AccountPage {
    user: User,
    scripts: Vec<Script>,
    tabs: Vec<(Markup, Markup, u8)>,
    csrf_token: String,
}

impl PageFragment for AccountPage {
    fn title(&self) -> String {
        format!("Account - {}", self.user.name)
    }

    fn description(&self) -> String {
        String::new()
    }

    fn additional_scripts(&self) -> Vec<Script> {
        let mut scripts = self.scripts.clone();
        scripts.push(Script::module("/static/js/staff.js"));
        scripts
    }

    fn additional_stylesheets(&self) -> Vec<String> {
        vec!["/static/css/account.css".to_string(), "/static/css/sidebar.css".to_string()]
    }

    fn head_fragment(&self) -> Markup {
        html! {
            (PreEscaped(
                format!(r#"<script>window.username='{}'; window.etag='{}'; window.permissions='{}'</script>"#, self.user.name, self.user.etag_string(), self.user.permissions)
            ))
        }
    }

    fn body_fragment(&self) -> Markup {
        html! {
            span#chicken-salad-red-fish style = "display:none" {(self.csrf_token)}
            div.tab-display#account-tabber {
                div.tab-selection.flex.wrap.m-center.fade style="text-align: center;" {
                    @for (i, (tab, _, id)) in self.tabs.iter().enumerate() {
                        @if i == 0 {
                            div.tab.tab-active.button.white.hover.no-shadow data-tab-id=(id) {
                                (*tab)
                            }
                        }
                        @else {
                            div.tab.button.white.hover.no-shadow data-tab-id=(id) {
                                (*tab)
                            }
                        }
                    }
                }

                @for (i, (_, content, id)) in self.tabs.iter().enumerate() {
                    @if i == 0 {
                        div.m-center.flex.tab-content.tab-content-active.container data-tab-id = (id){
                            (*content)
                        }
                    }
                    @else {
                        div.m-center.flex.tab-content.container data-tab-id = (id){
                            (*content)
                        }
                    }
                }
            }
        }
    }
}

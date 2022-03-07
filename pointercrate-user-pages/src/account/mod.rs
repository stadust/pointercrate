use maud::{html, Markup, PreEscaped};
use pointercrate_core::{etag::Taggable, permission::PermissionsManager};
use pointercrate_core_pages::{PageFragment, Script};
use pointercrate_user::{sqlx::PgConnection, User};

pub mod profile;
pub mod users;

#[async_trait::async_trait]
pub trait AccountPageTab {
    fn should_display_for(&self, user: &User, permissions: &PermissionsManager) -> bool;
    fn initialization_script(&self) -> String;
    fn additional_scripts(&self) -> Vec<Script> {
        vec![]
    }

    fn tab_id(&self) -> u8;
    fn tab(&self) -> Markup;
    async fn content(&self, user: &User, permissions: &PermissionsManager, connection: &mut PgConnection) -> Markup;
}

pub struct AccountPageConfig {
    tabs: Vec<Box<dyn AccountPageTab + Send + Sync + 'static>>,
}

impl Default for AccountPageConfig {
    fn default() -> Self {
        AccountPageConfig { tabs: Vec::new() }
    }
}

impl AccountPageConfig {
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
            if tab_config.should_display_for(&page.user, permissions) {
                let tab = tab_config.tab();
                let content = tab_config.content(&page.user, permissions, connection).await;

                page.scripts.extend(tab_config.additional_scripts());
                page.scripts.push(Script::module(tab_config.initialization_script()));
                page.tabs
                    .push((tab, content, tab_config.initialization_script(), tab_config.tab_id()));
            }
        }

        page
    }
}

pub struct AccountPage {
    user: User,
    scripts: Vec<Script>,
    tabs: Vec<(Markup, Markup, String, u8)>,
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
        self.scripts.clone()
    }

    fn additional_stylesheets(&self) -> Vec<String> {
        vec![
            "/static/user/css/account.css".to_string(),
            "/static/core/css/sidebar.css".to_string(),
        ]
    }

    fn head_fragment(&self) -> Markup {
        html! {
            (PreEscaped(
                format!(r#"<script>window.username='{}'; window.etag='{}'; window.permissions='{}'</script><script type="module">{}</script>"#, self.user.name, self.user.etag_string(), self.user.permissions, self.initialization_script())
            ))
        }
    }

    fn body_fragment(&self) -> Markup {
        html! {
            span#chicken-salad-red-fish style = "display:none" {(self.csrf_token)}
            div.tab-display#account-tabber {
                div.tab-selection.flex.wrap.m-center.fade style="text-align: center;" {
                    @for (i, (tab, _, _, id)) in self.tabs.iter().enumerate() {
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

                @for (i, (_, content, _, id)) in self.tabs.iter().enumerate() {
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

impl AccountPage {
    fn initialization_script(&self) -> String {
        let mut imports = r#"
import { TabbedPane } from "/static/core/js/modules/tab.js";
        "#
        .to_owned();
        let mut initialization_states = String::new();
        let mut initializations = String::new();

        for (_, _, script, i) in &self.tabs {
            imports.push_str(&format!(r#"import {{ initialize as initialize{} }} from "{}";"#, i, script));

            initialization_states.push_str(&format!("let initialized{} = false;", i));
            initializations.push_str(&format!(
                r#"
accountTabber.addSwitchListener("{0}", () => {{
if (!initialized{0}) {{
  initialize{0}(csrfToken, accountTabber);

  initialized{0} = true;
}}
}});
            "#,
                i
            ));
        }

        format!(
            r#"
        {}
        {}
        
$(document).ready(function () {{
    var csrfTokenSpan = document.getElementById("chicken-salad-red-fish");
    var csrfToken = csrfTokenSpan.innerHTML;
    
    csrfTokenSpan.remove();
    
    let accountTabber = new TabbedPane(
    document.getElementById("account-tabber"),
    "account-tab-selection"
    );
    
    {}
}});
        "#,
            imports, initialization_states, initializations
        )
    }
}

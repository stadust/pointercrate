use std::str::FromStr;

use maud::{html, PreEscaped};
use pointercrate_core::error::CoreError;
use pointercrate_core_pages::PageFragment;
use pointercrate_demonlist::list::List;
use rocket::{
    form::FromFormField,
    http::uri::fmt::{FromUriParam, Path, UriDisplay},
    request::FromParam,
};

pub struct ClientList(pub List);

impl<'a> FromParam<'a> for ClientList {
    type Error = CoreError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match List::from_str(param) {
            Ok(list) => Ok(ClientList(list)),
            Err(err) => Err(err),
        }
    }
}

impl<'a> FromUriParam<Path, &'a str> for ClientList {
    type Target = ClientList;

    fn from_uri_param(param: &'a str) -> Self::Target {
        ClientList(List::from_str(param).unwrap_or_default())
    }
}

impl UriDisplay<Path> for ClientList {
    fn fmt(&self, f: &mut rocket::http::uri::fmt::Formatter<'_, Path>) -> std::fmt::Result {
        f.write_value(&self.0.as_str())
    }
}

impl<'a> FromFormField<'a> for ClientList {
    fn from_value(field: rocket::form::ValueField<'a>) -> rocket::form::Result<'a, Self> {
        Ok(ClientList(
            List::from_str(field.value).map_err(|_| rocket::form::Error::validation("Failed to deserialize list"))?,
        ))
    }
}

/// A utility method for inserting <head> scripts to pages which are bound to a particular list
/// only used in stats viewer pages, demon pages, and demons overview i think rn
pub fn inject_list_context(list: &List, fragment: impl Into<PageFragment>) -> PageFragment {
    let fragment: PageFragment = fragment.into();
    let fragment = fragment.head(html! {
        (PreEscaped(format!(r#"
                <script>
                    window.active_list = "{0}";
                </script>
                <script defer>
                    document.documentElement.dataset.list = "{0}";
                </script>
            "#, list.as_str())))
    });

    fragment
}

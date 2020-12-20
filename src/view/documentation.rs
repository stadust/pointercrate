use super::Page;
use crate::{error::PointercrateError, extractor::auth::TokenAuth, permissions::Permissions, state::PointercrateState, Result, ViewResult};
use actix_web::{web::Path, HttpResponse};
use actix_web_codegen::get;
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
pub struct Documentation<'a> {
    toc: &'a str,
    content: &'a str,
    page: &'a str,
}

impl<'a> Documentation<'a> {
    pub fn api_documentation(state: &'a PointercrateState, page: &'a str) -> Result<Documentation<'a>> {
        let content = match state.documentation_topics.get(page) {
            Some(cnt) => cnt,
            _ => return Err(PointercrateError::NotFound),
        };

        Ok(Documentation {
            toc: &*state.documentation_toc,
            content,
            page,
        })
    }

    pub fn guidelines(state: &'a PointercrateState, page: &'a str) -> Result<Documentation<'a>> {
        let content = match state.guidelines_topics.get(page) {
            Some(cnt) => cnt,
            _ => return Err(PointercrateError::NotFound),
        };

        Ok(Documentation {
            toc: &*state.guidelines_toc,
            content,
            page,
        })
    }
}

// actix complains if these aren't async, although they don't not have to be
#[get("/documentation/")]
pub async fn index(state: PointercrateState) -> ViewResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(Documentation::api_documentation(&state, "index")?.render().0))
}

#[get("/documentation/{topic}/")]
pub async fn topic(state: PointercrateState, topic: Path<String>) -> ViewResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(Documentation::api_documentation(&state, &topic.into_inner())?.render().0))
}

// actix complains if these aren't async, although they don't not have to be
#[get("/guidelines/")]
pub async fn guildelines_index(TokenAuth(user): TokenAuth, state: PointercrateState) -> ViewResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListHelper)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(Documentation::guidelines(&state, "index")?.render().0))
}

// cannot have multiple parameters with the same name in the same field it seems because actix_web
// generates a unit struct for them.
#[get("/guidelines/{gtopic}/")]
pub async fn guidelines_topic(TokenAuth(user): TokenAuth, state: PointercrateState, gtopic: Path<String>) -> ViewResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListHelper)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(Documentation::guidelines(&state, &gtopic.into_inner())?.render().0))
}

impl<'a> Page for Documentation<'a> {
    fn title(&self) -> String {
        format!("API Documentation - {}", self.page)
    }

    fn description(&self) -> String {
        "The pointercrate API, which allows you to programmatically interface with the Demonlist".to_owned()
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/sidebar.css", "css/doc.css"]
    }

    fn body(&self) -> Markup {
        html! {
            div class="m-center flex container" {
                div.left {
                    (PreEscaped(self.content))
                }
                div.right {
                    (PreEscaped(self.toc))
                }
            }
            (PreEscaped(r#"
                <script>
                // you know, this might be the most ugly solution to a problem I have ever thought of
                $(document).ready(function() {
                  for(let header of document.getElementsByTagName("h1")) {
                    header.innerHTML += '<a class="fa fa-link fa-3 link-anchor" aria-hidden="true" title="Permanent link to this topic" href = #' + header.parentNode.id + '></a>';
                    header.innerHTML = '<i class="fa fa-link fa-3 link-anchor" style="visibility:hidden" aria-hidden="true"></i>' + header.innerHTML;
                  }
                })
                </script>
            "#))
        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(r#"
<script type="application/ld+json">
  {
    "@context": "http://schema.org",
    "@type": "WebPage",
    "breadcrumb": {
      "@type": "BreadcrumbList",
      "itemListElement": [{
        "@type": "ListItem",
        "position": 1,
        "item": {
          "@id": "https://pointercrate.com/",
          "name": "pointercrate"
        }
      },{
        "@type": "ListItem",
        "position": 2,
        "item": {
          "@id": "https://pointercrate.com/documentation/",
          "name": "documentation"
        }
      },{
        "@type": "ListItem",
        "position": 3,
        "item": {
            "@id": "https://pointercrate.com/documentation/account/",
            "name": "account"
        }
      }]
    },
    "name": "API Documentation",
    "description": "The pointercrate API, which allows you to programmatically interface with the Demonlist",
    "url": "https://pointercrate.com/documentation/account/",
    "dateCreated": "2017-04-08"
  }
</script>
            "#))
        }]
    }
}

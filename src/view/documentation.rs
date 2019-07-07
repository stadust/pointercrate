use super::Page;
use crate::{error::PointercrateError, state::PointercrateState, Result};
use actix_web::HttpRequest;
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
pub struct Documentation<'a> {
    toc: &'a str,
    content: &'a str,
    page: String,
}

impl<'a> Documentation<'a> {
    pub fn new(state: &'a PointercrateState, page: String) -> Result<Documentation<'a>> {
        let content = match state.documentation_topics.get(&page) {
            Some(cnt) => cnt,
            _ => return Err(PointercrateError::NotFound),
        };

        Ok(Documentation {
            toc: &*state.documentation_toc,
            content,
            page,
        })
    }
}

impl<'a> Page for Documentation<'a> {
    fn title(&self) -> String {
        format!("API Documentation - {}", self.page)
    }

    fn description(&self) -> String {
        "The pointercrate API, which allows you to programmatically interface with the Demonlist"
            .to_owned()
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/sidebar.css", "css/doc.css"]
    }

    fn body(&self, _: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            div class="m-center flex container" {
                div.left {
                    (PreEscaped(self.content))
                }
                div.right {
                    (PreEscaped(self.toc))
                }
            }
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
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
<script>
// you know, this might be the most ugly solution to a problem I have ever thought of
$(document).ready(function() {
  $("h1").append(
    '<a class="fa fa-link fa-3 link-anchor" aria-hidden="true" title="Permanent link to this topic"></a>'
  );
  $("h1").prepend(
    '<i class="fa fa-link fa-3 link-anchor" style="visibility:hidden" aria-hidden="true"></i>'
  );
  $(".link-anchor").each((idx, elem) =>
    $(elem).attr("href", '#' + $(elem).parent()[0].id)
  );
})
</script>
            "#))
        }]
    }
}

use crate::features::auth;
use topcoat::{
    Result,
    context::Cx,
    htmx::{HxRedirect, hx_request},
    router::{
        error::see_other,
        response::{IntoResponse, Response},
        route,
    },
};

#[route(POST)]
pub async fn logout(cx: &Cx) -> Result<Response> {
    auth::clear_session(cx);
    if hx_request(cx) {
        (HxRedirect::from("/"), ()).into_response(cx)
    } else {
        see_other("/").into_response(cx)
    }
}

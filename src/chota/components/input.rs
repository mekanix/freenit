use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, class, component, view},
};

/// A styled input using Chota's native form styling.
///
/// Pass input attributes and event handlers through `attrs`. Extra classes are
/// added to the input's classes. Chota fills the container by default.
///
/// ```
/// # let cx = topcoat::context::Cx::new();
/// # use crate::chota::components::input::input;
/// # use topcoat::view::{attributes, view};
/// # let _ =
/// view! {
///     input(attrs: attributes! { type="email" placeholder="you@example.com" })
/// }
/// # ;
/// ```
#[component]
pub async fn input(cx: &Cx, #[default] mut attrs: Attributes) -> Result<impl View> {
    Ok(view! { <input class=(class!(attrs.remove("class"))) (attrs)> })
}

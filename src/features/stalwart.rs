use topcoat::{Result, context::Cx, router::error::internal_server_error};

pub use crate::config::StalwartConfig;
use crate::config::Config;

pub fn stalwart_config(cx: &Cx) -> StalwartConfig {
    topcoat::context::context::<Config>(cx).stalwart.clone()
}

pub fn protocol<T>(result: Result<T, String>) -> Result<T> {
    result.map_err(|error| {
        tracing::error!("Stalwart backend error: {error}");
        internal_server_error(std::io::Error::other(error)).into()
    })
}

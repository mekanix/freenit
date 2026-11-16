mod app;
pub mod chota;

use freenit::{config, db, features};

#[tokio::main]
async fn main() {
    let config = config::Config::load().unwrap();
    let log_level = config
        .logging
        .level
        .parse::<tracing::Level>()
        .unwrap_or_else(|error| {
            eprintln!("invalid [logging].level: {error}");
            std::process::exit(2);
        });
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init()
        .unwrap_or_else(|error| {
            eprintln!("failed to initialize logging: {error}");
            std::process::exit(1);
        });
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap_or_else(|_| {
            tracing::error!("failed to install the Rustls ring crypto provider");
            std::process::exit(1);
        });
    let _connection = db::connect(&config).await.unwrap_or_else(|error| {
        tracing::error!("failed to connect configured backends: {error}");
        std::process::exit(1);
    });

    #[cfg(feature = "sql")]
    {
        // `cargo run -- toasty migration <command>` manages the database schema.
        let mut args = config::Config::args_without_config().into_iter().peekable();
        if args.peek().is_some_and(|arg| arg == "toasty") {
            let Some(backend) = _connection.sql else {
                tracing::error!("SQL storage is unavailable for migrations");
                std::process::exit(1);
            };
            if let Err(error) = toasty_cli::ToastyCli::new(backend).parse_from(args).await {
                tracing::error!("{error:#}");
                std::process::exit(1);
            }
            return;
        }
    }

    let router = app::router();
    #[cfg(feature = "sql")]
    let router = if let Some(backend) = _connection.sql {
        router.with(backend)
    } else {
        router
    };
    let router = router.with(config.clone());
    #[cfg(feature = "mail")]
    let router = router.with(features::mail::mailer(&config.mail).unwrap());

    topcoat::start(router.build()).await.unwrap();
}

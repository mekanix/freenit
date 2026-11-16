use freenit::{config::Config, db, features::user::User};
use topcoat::context::Cx;

#[tokio::main]
async fn main() {
    let email = Config::args_without_config()
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --bin make-admin -- [--config FILE] <email>");
            std::process::exit(2);
        });
    if let Err(error) = make_admin(&email).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn make_admin(email: &str) -> Result<(), String> {
    let config = Config::load().map_err(|error| error.to_string())?;
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| "failed to install the Rustls ring crypto provider".to_owned())?;
    let _connection = db::connect(&config)
        .await
        .map_err(|error| error.to_string())?;
    let cx = Cx::new();
    #[cfg(feature = "sql")]
    let cx = if let Some(backend) = _connection.sql {
        cx.with(backend)
    } else {
        cx
    };
    let cx = cx.with(config);
    let user = User::find_by_email(&cx, email)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("user not found: {email}"))?;
    User::set_admin(&cx, &user.id, true)
        .await
        .map_err(|error| error.to_string())?;
    println!("{} is now an administrator", user.email);
    Ok(())
}

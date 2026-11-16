use topcoat::{
    Result,
    context::Cx,
    mail::{Mail, Mailbox, Mailer, SmtpTransport, send},
};

use crate::config::{Config, MailConfig};

pub fn mailer(config: &MailConfig) -> Result<Mailer> {
    let builder = if config.tls {
        SmtpTransport::starttls(&config.server)?
    } else {
        SmtpTransport::unencrypted(&config.server)
    };
    let transport = builder
        .port(config.port)
        .credentials(&config.user, &config.password)
        .build();
    Ok(Mailer::new(transport))
}

pub async fn send_verification(cx: &Cx, user: &crate::features::user::User) -> Result<()> {
    let token =
        crate::features::auth::issue_token(cx, user, crate::features::auth::TokenPurpose::Verify)?;
    println!("{token}");
    send_link(
        cx,
        &user.email,
        "Freenit registration",
        "confirm your registration",
        &format!("verify/{token}"),
    )
    .await
}

pub async fn send_recovery(cx: &Cx, user: &crate::features::user::User) -> Result<()> {
    let token =
        crate::features::auth::issue_token(cx, user, crate::features::auth::TokenPurpose::Reset)?;
    send_link(
        cx,
        &user.email,
        "Freenit password recovery",
        "reset your password",
        &format!("reset-password/{token}"),
    )
    .await
}

async fn send_link(cx: &Cx, address: &str, subject: &str, action: &str, path: &str) -> Result<()> {
    let config = topcoat::context::context::<Config>(cx);
    let headers = topcoat::router::request::headers(cx);
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:3000");
    let scheme = if config.auth.secure_cookies {
        "https"
    } else {
        "http"
    };
    let link = format!("{scheme}://{host}/{path}");
    let mail = Mail::builder()
        .from(Mailbox::new(&config.mail.from)?)
        .to([Mailbox::new(address)?])
        .subject(subject)
        .text(format!(
            "Hello,\n\nPlease {action} by following this link:\n\n{link}\n\nRegards,\nFreenit\n"
        ))
        .build()?;
    if let Err(error) = send(cx, mail).await {
        tracing::error!("could not deliver the mail: {error:#}");
    }
    Ok(())
}

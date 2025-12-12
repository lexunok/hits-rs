use std::env;
use anyhow::{Error, Ok};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType, transport::smtp::authentication::Credentials};
use tera::{Context, Tera};
use crate::models::smtp::{CodeEmailContext, Notification};

pub async fn send_code_to_reset_password(code: String, email: String) -> Result<(), Error> {
    let subject = "Код для изменения пароля".to_string();
    let code_email_context = CodeEmailContext {
        code,
        email: email.clone(),
        subject: subject.clone(),
        text: "Вы изменяете пароль на вашем аккаунте. Необходимо ввести код для подтверждения изменения".to_string()
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("code_email_context", &code_email_context);
    let html = tera.render("verification_code.html", &ctx)?;

    send_message_to_email(email, html, subject).await?;

    Ok(())
}
pub async fn send_invitation(id: String, url: String, first_name:String, last_name:String, email: String) -> Result<(), Error> {
    let subject = "Приглашение на регистрацию".to_string();
    let link = format!(
            "{}/auth/registration?code={}",
            url,
            id
    );
    let invitation_text = format!(
        "Вас пригласил(-а) зарегистрироваться на портал HITS {} {} \
        в качестве пользователя. Для регистрации на сервисе \
        перейдите по данной ссылке и заполните все поля.",
        first_name, last_name
    );

    let notification = Notification{
        email: email.clone(),
        title: subject.clone(),
        message: invitation_text,
        link,
        button_name: "Зарегистрироваться".to_string()
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("notification", &notification);
    let html = tera.render("notification.html", &ctx)?;

    send_message_to_email(email, html, subject).await?;

    Ok(())
}
pub async fn send_message_to_email(email: String, html: String, subject: String) -> Result<(), Error> {
    let smtp_host = env::var("SMTP_HOST")?;
    let smtp_from = env::var("SMTP_FROM")?;

    let mailer: AsyncSmtpTransport<Tokio1Executor> = if cfg!(debug_assertions) {
        let smtp_user = env::var("SMTP_USER")?;
        let smtp_password = env::var("SMTP_PASSWORD")?;

        let creds = Credentials::new(smtp_user, smtp_password);

        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_host).build()
    };
    
    let message = Message::builder()
        .from(smtp_from.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(subject.clone())
        .header(ContentType::TEXT_HTML)
        .body(html)?;

    mailer.send(message).await?;
    
    tracing::debug!(
        "Отправляем письмо {} на {}",
        subject,
        email,
    );

    Ok(())
}
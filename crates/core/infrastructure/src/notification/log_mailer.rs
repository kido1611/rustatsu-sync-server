use async_trait::async_trait;
use core_domain::{
    notification::{mailer::Mailer, model::MailEnvelope},
    shared::error::DomainError,
};
use tracing::{Level, info, instrument};

pub struct LogMailer {}

#[async_trait]
impl Mailer for LogMailer {
    #[instrument(name = "notification::send_mail", skip_all, fields(to = %envelope.to), level = Level::DEBUG, err(level = Level::ERROR))]
    async fn send(&self, envelope: MailEnvelope) -> Result<(), DomainError> {
        info!(
            r#"SENDING MAIL
            --------------
            to: {}
            subject: {}
            body: {}"#,
            envelope.to, envelope.subject, envelope.body
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core_domain::notification::{mailer::Mailer, model::MailEnvelope};

    use crate::notification::log_mailer::LogMailer;

    #[tokio::test]
    async fn should_be_ok_when_sending_mail() {
        let mailer = LogMailer {};

        let res = mailer
            .send(MailEnvelope {
                to: "test@email.com".to_string(),
                subject: "subject".to_string(),
                body: "body".to_string(),
            })
            .await;

        assert!(res.is_ok());
    }
}

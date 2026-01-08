use core_domain::notification::model::MailEnvelope;

pub struct ForgotPasswordMail {
    pub url: String,
}

impl ForgotPasswordMail {
    pub fn to_mail_envelope(self, to: String) -> MailEnvelope {
        MailEnvelope {
            to,
            subject: "Reset Your Password".to_string(),
            body: self.generate_body(),
        }
    }

    fn generate_body(&self) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Forgot password</title>
</head>
<body style="font-family:-apple-system,'Segoe UI',Roboto,Arial,sans-serif;">
<main>
    <p>We received a request to reset your password.</p>
    <p>You can reset your password at <a href="{0}">{0}</a></p>
    <p>If you didn't make this request, please ignore this email.</p>
</main>
</body>
</html>
            "#,
            self.url
        )
    }
}

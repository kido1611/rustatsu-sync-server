use axum::{
    extract::{Query, State},
    response::Html,
};
use serde::Deserialize;

use crate::{error::Error, state::SharedAppState};

#[derive(Deserialize)]
pub struct ResetPasswordQuery {
    pub token: String,
}

#[tracing::instrument(name = "request::deep_link_reset_password", skip_all)]
pub async fn reset_password(
    State(app_state): State<SharedAppState>,
    query: Query<ResetPasswordQuery>,
) -> Result<Html<String>, Error> {
    let deep_link = format!(
        "kotatsu://reset-password?base_url={}&token={}",
        app_state.config.application.get_base_url(),
        query.token
    );

    let page = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Reset password</title>
    <script type="text/javascript">
        window.location.href = "{0}";
    </script>
</head>
<body style="font-family:-apple-system,'Segoe UI',Roboto,Arial,sans-serif;">
<main>
    <p>If you are not redirected automatically, <a href="{}">click here</a>.</p>
</main>
</body>
</html>
        "#,
        deep_link
    );

    Ok(Html(page))
}

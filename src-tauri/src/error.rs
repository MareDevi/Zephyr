use serde::Serialize;
use specta::Type;

pub type AppResult<T> = anyhow::Result<T>;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        let message = value.to_string();
        let code = if message.contains("state lock poisoned") {
            "state_poisoned"
        } else if message.contains("D-Bus") {
            "dbus_error"
        } else {
            "internal_error"
        };
        tracing::error!(code = code, message = %message, "api error");
        Self { code, message }
    }
}

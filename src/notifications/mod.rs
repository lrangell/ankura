use notify_rust::{Notification, Timeout};
use tracing::error;

pub struct NotificationManager {
    app_name: String,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            app_name: "Karabiner-Pkl".to_string(),
        }
    }

    pub fn send_success(&self, message: &str) {
        self.send_notification("✅ Success", message, false);
    }

    pub fn send_error(&self, message: &str) {
        self.send_notification("❌ Error", message, true);
    }

    #[allow(dead_code)]
    pub fn send_info(&self, message: &str) {
        self.send_notification("ℹ️ Info", message, false);
    }

    fn send_notification(&self, title: &str, message: &str, is_error: bool) {
        let result = Notification::new()
            .appname(&self.app_name)
            .summary(title)
            .body(message)
            .timeout(if is_error {
                Timeout::Never
            } else {
                Timeout::Milliseconds(3000)
            })
            .show();

        if let Err(e) = result {
            error!("Failed to send notification: {}", e);
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
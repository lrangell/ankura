use crate::cli::{merge_configurations, write_karabiner_config};
use crate::compiler::Compiler;
use crate::error::{KarabinerPklError, Result};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use notify_rust::{Notification, Timeout};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct Daemon {
    config_path: PathBuf,
    compiler: Arc<Compiler>,
    notification_manager: Arc<NotificationManager>,
    is_running: Arc<RwLock<bool>>,
    watcher: Arc<RwLock<Option<Debouncer<RecommendedWatcher>>>>,
}

impl Daemon {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let compiler = Arc::new(Compiler::new()?);
        let notification_manager = Arc::new(NotificationManager::new());

        Ok(Self {
            config_path,
            compiler,
            notification_manager,
            is_running: Arc::new(RwLock::new(false)),
            watcher: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                return Err(KarabinerPklError::DaemonError {
                    message: "Daemon is already running".to_string(),
                });
            }
            *is_running = true;
        }

        info!("Starting ankura daemon");
        debug!("Watching: {}", self.config_path.display());

        self.compile_and_notify(None).await;

        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_secs(5), tx)
            .map_err(|e| KarabinerPklError::WatchError { source: e })?;

        let config_dir = self.config_path.parent().unwrap_or(&self.config_path);
        debouncer
            .watcher()
            .watch(config_dir, RecursiveMode::Recursive)
            .map_err(|e| KarabinerPklError::WatchError { source: e })?;

        {
            let mut watcher_guard = self.watcher.write().await;
            *watcher_guard = Some(debouncer);
        }

        let compiler = self.compiler.clone();
        let notification_manager = self.notification_manager.clone();
        let config_path = self.config_path.clone();
        let is_running = self.is_running.clone();
        let config_file_name = config_path.file_name().map(OsString::from);
        let watcher = self.watcher.clone();

        tokio::spawn(async move {
            enum WatchLoopExit {
                ChannelClosed(std::sync::mpsc::RecvError),
                Stopped,
            }

            let exit_reason = loop {
                match rx.recv() {
                    Ok(Ok(events)) => {
                        let should_compile = events.iter().any(|event| {
                            let is_target = if let Some(file_name) = &config_file_name {
                                event
                                    .path
                                    .file_name()
                                    .map_or(false, |name| name == file_name)
                            } else {
                                event.path == config_path
                            };
                            let is_settled = event.kind == DebouncedEventKind::Any;
                            is_target && is_settled
                        });

                        if should_compile {
                            debug!("Configuration file changed, recompiling...");
                            Self::compile_with_notification(
                                &compiler,
                                &notification_manager,
                                &config_path,
                                None,
                            )
                            .await;
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Watch error: {:?}", e);
                    }
                    Err(e) => break WatchLoopExit::ChannelClosed(e),
                }

                if !*is_running.read().await {
                    break WatchLoopExit::Stopped;
                }
            };

            match exit_reason {
                WatchLoopExit::ChannelClosed(e) => {
                    if *is_running.read().await {
                        error!("File watcher channel closed unexpectedly: {:?}", e);
                    } else {
                        debug!("File watcher channel closed");
                    }
                }
                WatchLoopExit::Stopped => {
                    debug!("File watcher loop stopping");
                }
            }

            let mut watcher_guard = watcher.write().await;
            watcher_guard.take();
        });

        info!("Daemon started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping ankura daemon");
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        let mut watcher_guard = self.watcher.write().await;
        watcher_guard.take();
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn compile_once(
        &self,
        profile_name: Option<&str>,
        _output_path: Option<&str>,
    ) -> Result<()> {
        self.compile_and_notify(profile_name).await;
        Ok(())
    }

    async fn compile_and_notify(&self, profile_name: Option<&str>) {
        Self::compile_with_notification(
            &self.compiler,
            &self.notification_manager,
            &self.config_path,
            profile_name,
        )
        .await;
    }

    async fn compile_with_notification(
        compiler: &Arc<Compiler>,
        notification_manager: &Arc<NotificationManager>,
        config_path: &Path,
        profile_name: Option<&str>,
    ) {
        match compiler.compile(config_path, profile_name).await {
            Ok(config) => {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                let output_path = home.join(".config/karabiner/karabiner.json");

                let final_config = if output_path.exists() {
                    match merge_configurations(&output_path, config) {
                        Ok(merged) => merged,
                        Err(e) => {
                            error!("Failed to merge configurations: {:?}", e);
                            notification_manager.send_error(&format!("Merge failed: {e}"));
                            return;
                        }
                    }
                } else {
                    config
                };

                match write_karabiner_config(&output_path, &final_config) {
                    Ok(_) => {
                        info!("Successfully compiled configuration");
                        notification_manager.send_success("Karabiner configuration updated");
                    }
                    Err(e) => {
                        error!("Failed to write configuration: {:?}", e);
                        notification_manager.send_error(&format!("Write failed: {e}"));
                    }
                }
            }
            Err(e) => {
                error!("Compilation failed: {:?}", e);
                let error_msg = format!("Compilation failed: {e}");
                notification_manager.send_error(&error_msg);
            }
        }
    }
}

struct NotificationManager {
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

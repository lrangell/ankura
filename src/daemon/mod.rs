use crate::compiler::Compiler;
use crate::error::{KarabinerPklError, Result};
use crate::notifications::NotificationManager;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info};

pub struct Daemon {
    config_path: PathBuf,
    compiler: Arc<Compiler>,
    notification_manager: Arc<NotificationManager>,
    is_running: Arc<RwLock<bool>>,
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

        info!("Starting karabiner-pkl daemon");
        info!("Watching: {}", self.config_path.display());

        self.compile_and_notify().await;

        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(300), tx)
            .map_err(|e| KarabinerPklError::WatchError { source: e })?;

        let config_dir = self.config_path.parent().unwrap_or(&self.config_path);
        debouncer
            .watcher()
            .watch(config_dir, RecursiveMode::Recursive)
            .map_err(|e| KarabinerPklError::WatchError { source: e })?;

        let compiler = self.compiler.clone();
        let notification_manager = self.notification_manager.clone();
        let config_path = self.config_path.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            for res in rx {
                match res {
                    Ok(events) => {
                        for event in events {
                            if event.kind == DebouncedEventKind::Any
                                && event.path.ends_with("karabiner.pkl")
                            {
                                info!("Configuration file changed, recompiling...");
                                Self::compile_with_notification(
                                    &compiler,
                                    &notification_manager,
                                    &config_path,
                                )
                                .await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {:?}", e);
                    }
                }

                if !*is_running.read().await {
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping karabiner-pkl daemon");
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    pub async fn compile_once(&self) -> Result<()> {
        self.compile_and_notify().await;
        Ok(())
    }

    async fn compile_and_notify(&self) {
        Self::compile_with_notification(
            &self.compiler,
            &self.notification_manager,
            &self.config_path,
        )
        .await;
    }

    async fn compile_with_notification(
        compiler: &Arc<Compiler>,
        notification_manager: &Arc<NotificationManager>,
        config_path: &Path,
    ) {
        match compiler.compile(config_path).await {
            Ok(_) => {
                info!("Successfully compiled configuration");
                notification_manager.send_success("Karabiner configuration updated");
            }
            Err(e) => {
                error!("Compilation failed: {:?}", e);
                let error_msg = format!("Compilation failed: {}", e);
                notification_manager.send_error(&error_msg);
            }
        }
    }
}
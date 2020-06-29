use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures::FutureExt;
use snafu::ResultExt;
use structopt::StructOpt;
use tokio::{runtime::Runtime, sync::Mutex};

use clipcat::{ClipboardManager, ClipboardMonitor};

use crate::{
    config::{Config, ConfigError},
    error::{self, Error},
    history::HistoryManager,
    lifecycle::{self, LifecycleManager},
};

#[derive(StructOpt, Clone)]
#[structopt(name = clipcat::DAEMON_PROGRAM_NAME)]
pub struct Command {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,

    #[structopt(long = "no-daemon", help = "Does not run as daemon")]
    no_daemon: bool,

    #[structopt(long = "replace", short = "r", help = "Tries to replace existing daemon")]
    replace: bool,

    #[structopt(long = "config", short = "c", help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[structopt(long = "history-file", help = "Specifies a history file")]
    history_file_path: Option<PathBuf>,

    #[structopt(long = "grpc-host", help = "Specifies gRPC host address")]
    grpc_host: Option<IpAddr>,

    #[structopt(long = "grpc-port", help = "Specifies gRPC port number")]
    grpc_port: Option<u16>,
}

#[derive(StructOpt, Clone)]
pub enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },

    #[structopt(about = "Outputs default configuration")]
    DefaultConfig,
}

impl Command {
    pub fn new() -> Command { StructOpt::from_args() }

    fn load_config(&self) -> Result<Config, ConfigError> {
        let config_file = &self.config_file.clone().unwrap_or_else(Config::default_path);
        let mut config = Config::load(&config_file)?;

        config.daemonize = !self.no_daemon;

        if let Some(history_file_path) = &self.history_file_path {
            config.history_file_path = history_file_path.clone();
        }

        if let Some(host) = self.grpc_host {
            config.grpc.host = host;
        }

        if let Some(port) = self.grpc_port {
            config.grpc.port = port;
        }

        Ok(config)
    }

    pub fn run(self) -> Result<(), Error> {
        match self.subcommand {
            Some(SubCommand::Version) => {
                Command::clap()
                    .write_long_version(&mut std::io::stdout())
                    .expect("failed to write to stdout");
                return Ok(());
            }
            Some(SubCommand::Completions { shell }) => {
                Command::clap().gen_completions_to(
                    clipcat::DAEMON_PROGRAM_NAME,
                    shell,
                    &mut std::io::stdout(),
                );
                return Ok(());
            }
            Some(SubCommand::DefaultConfig) => {
                use std::io::Write;
                let config_text =
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable");
                std::io::stdout()
                    .write_all(&config_text.as_bytes())
                    .expect("failed to write to stdout");
                return Ok(());
            }
            None => {}
        }

        let config = self.load_config().context(error::LoadConfig)?;
        run_clipcatd(config, self.replace)
    }
}

#[inline]
fn kill_other(pid: u64) -> Result<(), Error> {
    let ret = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    if ret != 0 {
        return Err(Error::SendSignalTerminal { pid });
    }
    Ok(())
}

fn run_clipcatd(config: Config, replace: bool) -> Result<(), Error> {
    let pid_file = PidFile::from(config.pid_file);
    if config.daemonize {
        if pid_file.exists() && replace {
            let pid = pid_file.try_load()?;
            kill_other(pid)?;

            // sleep for a while
            std::thread::sleep(Duration::from_millis(200));
        }

        let daemonize = daemonize::Daemonize::new().pid_file(pid_file.clone_path());
        if let Err(err) = daemonize.start() {
            return Err(Error::Daemonize { source: err });
        }
    }

    syslog::init(syslog::Facility::LOG_USER, config.log_level, None)
        .context(error::InitializeSyslog)?;

    info!("{} is initializing, pid: {}", clipcat::DAEMON_PROGRAM_NAME, std::process::id());

    let (mut lifecycle_manager, internal_shutdown_signal) = LifecycleManager::new();

    let grpc_addr = format!("{}:{}", config.grpc.host, config.grpc.port)
        .parse()
        .context(error::ParseSockAddr)?;

    let (clipboard_manager, history_manager) = {
        let file_path = config.history_file_path;

        info!("History file path: {:?}", file_path);
        let history_manager =
            HistoryManager::new(&file_path).context(error::CreateHistoryManager)?;

        info!("Load history from {:?}", history_manager.path());
        let history_clips = history_manager.load().context(error::LoadHistoryManager)?;
        let clip_count = history_clips.len();
        info!("{} clip(s) loaded", clip_count);

        info!("Initialize ClipboardManager with capacity {}", config.max_history);
        let mut clipboard_manager = ClipboardManager::with_capacity(config.max_history);

        info!("Import {} clip(s) into ClipboardManager", clip_count);
        clipboard_manager.import(&history_clips);

        (Arc::new(Mutex::new(clipboard_manager)), Arc::new(Mutex::new(history_manager)))
    };

    let clipboard_monitor_fut = {
        use clipcat::{ClipboardData, ClipboardType};

        let (shutdown_signal, mut shutdown_slot) = lifecycle::shutdown_handle();
        let monitor_opts = config.monitor.into();
        let mut clipboard_monitor =
            ClipboardMonitor::new(monitor_opts).context(error::CreateClipboardMonitor)?;
        lifecycle_manager
            .register("Clipboard Monitor", Box::new(move || shutdown_signal.shutdown()));
        let clipboard_manager = clipboard_manager.clone();

        async move {
            loop {
                let event = futures::select! {
                    event = clipboard_monitor.recv().fuse() => event,
                    _ = shutdown_slot.wait().fuse() => break,
                };

                match event {
                    Some(event) => {
                        match event.clipboard_type {
                            ClipboardType::Clipboard => info!("Clipboard [{:?}]", event.data),
                            ClipboardType::Primary => info!("Primary [{:?}]", event.data),
                        }

                        let data = ClipboardData::from(event);
                        clipboard_manager.lock().await.insert(data.clone());
                        let _ = history_manager.lock().await.put(&data);
                    }
                    None => {
                        info!("ClipboardMonitor is closing, no further values will be received");
                        drop(clipboard_monitor);
                        info!("Internal shutdown signal is sent");
                        internal_shutdown_signal.shutdown();
                        break;
                    }
                }
            }

            let (clips, history_capacity) = {
                let cm = clipboard_manager.lock().await;
                (cm.list(), cm.capacity())
            };

            {
                let mut hm = history_manager.lock().await;

                info!("Save history and shrink to capacity {}", history_capacity);
                if let Err(err) = hm.save_and_shrink_to(&clips, history_capacity) {
                    warn!("Failed to save history, error: {:?}", err);
                }
            }
            info!("ClipboardMonitor is down");
        }
    };

    let grpc_server_fut = {
        use clipcat::grpc::{GrpcServer, GrpcService};

        let (shutdown_signal, mut shutdown_slot) = lifecycle::shutdown_handle();
        lifecycle_manager.register("gRPC Server", Box::new(move || shutdown_signal.shutdown()));

        let grpc_service = GrpcService::new(clipboard_manager);

        async move {
            let server =
                tonic::transport::Server::builder().add_service(GrpcServer::new(grpc_service));

            info!("gRPC service listening on {}", grpc_addr);
            futures::select! {
                _ = server.serve(grpc_addr).fuse() => {},
                _ = shutdown_slot.wait().fuse() => {},
            }
            info!("gRPC service is down");
        }
    };

    let mut runtime = Runtime::new().context(error::InitializeTokioRuntime)?;
    runtime.spawn(clipboard_monitor_fut);
    runtime.block_on(lifecycle_manager.block_on(grpc_server_fut));

    if config.daemonize {
        pid_file.remove()?;
    }

    info!("{} is shutdown", clipcat::DAEMON_PROGRAM_NAME);
    Ok(())
}

struct PidFile {
    path: PathBuf,
}

impl PidFile {
    #[inline]
    fn exists(&self) -> bool { self.path.exists() }

    #[inline]
    fn clone_path(&self) -> PathBuf { self.path().to_path_buf() }

    #[inline]
    fn path(&self) -> &Path { &self.path }

    fn try_load(&self) -> Result<u64, Error> {
        let pid_data = std::fs::read_to_string(&self)
            .context(error::ReadPidFile { filename: self.clone_path() })?;
        let pid = pid_data.trim().parse().context(error::ParseProcessId { value: pid_data })?;
        Ok(pid)
    }

    #[inline]
    fn remove(self) -> Result<(), Error> {
        info!("Remove PID file: {:?}", self.path);
        std::fs::remove_file(&self.path).context(error::RemovePidFile { pid_file: self.path })?;
        Ok(())
    }
}

impl From<PathBuf> for PidFile {
    fn from(path: PathBuf) -> PidFile { PidFile { path } }
}

impl AsRef<Path> for PidFile {
    fn as_ref(&self) -> &Path { &self.path }
}

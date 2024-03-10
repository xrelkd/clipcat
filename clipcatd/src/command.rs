use std::{io::Write, net::IpAddr, path::PathBuf, time::Duration};

use clap::{CommandFactory, Parser, Subcommand};
use snafu::ResultExt;
use tokio::runtime::Runtime;

use crate::{
    config::Config,
    error::{self, Error},
    pid_file::PidFile,
    shadow,
};

#[derive(Parser)]
#[command(
    name = clipcat_base::DAEMON_PROGRAM_NAME,
    author,
    version,
    long_version = shadow::CLAP_LONG_VERSION,
    about,
    long_about = None
)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Option<Commands>,

    #[clap(long = "no-daemon", help = "Do not run as daemon")]
    no_daemon: bool,

    #[clap(long = "replace", short = 'r', help = "Try to replace existing daemon")]
    replace: bool,

    #[clap(
        long = "config",
        short = 'c',
        env = "CLIPCATD_CONFIG_FILE_PATH",
        help = "Specify a configuration file"
    )]
    config_file: Option<PathBuf>,

    #[clap(
        long = "history-file",
        env = "CLIPCATD_HISTORY_FILE_PATH",
        help = "Specify a history file"
    )]
    history_file_path: Option<PathBuf>,

    #[clap(long = "grpc-host", env = "CLIPCATD_GRPC_HOST", help = "Specify gRPC host address")]
    grpc_host: Option<IpAddr>,

    #[clap(long = "grpc-port", env = "CLIPCATD_GRPC_PORT", help = "Specify gRPC port number")]
    grpc_port: Option<u16>,

    #[clap(
        long = "grpc-socket-path",
        env = "CLIPCATD_GRPC_SOCKET_PATH",
        help = "Specify gRPC local socket path"
    )]
    grpc_socket_path: Option<PathBuf>,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    #[clap(about = "Print version information")]
    Version,

    #[clap(about = "Output shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[clap(about = "Output default configuration")]
    DefaultConfig,
}

impl Default for Cli {
    #[inline]
    fn default() -> Self { Self::parse() }
}

impl Cli {
    pub fn run(self) -> Result<(), Error> {
        match self.subcommand {
            Some(Commands::Version) => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("failed to write to stdout");
                Ok(())
            }
            Some(Commands::Completions { shell }) => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                Ok(())
            }
            Some(Commands::DefaultConfig) => {
                let config_text =
                    toml::to_string_pretty(&Config::default()).expect("`Config` is serializable");
                std::io::stdout()
                    .write_all(config_text.as_bytes())
                    .expect("failed to write to stdout");
                Ok(())
            }
            None => {
                let config = self.load_config()?;
                run_clipcatd(config, self.replace)
            }
        }
    }

    fn load_config(&self) -> Result<Config, Error> {
        let config_file = &self.config_file.clone().unwrap_or_else(Config::default_path);
        let mut config = Config::load(config_file)?;

        config.daemonize = !self.no_daemon;

        if let Some(history_file_path) = &self.history_file_path {
            config.history_file_path = history_file_path.clone();
        }

        if let Some(host) = self.grpc_host {
            config.grpc.enable_http = true;
            config.grpc.host = host;
        }

        if let Some(port) = self.grpc_port {
            config.grpc.port = port;
        }

        if let Some(path) = &self.grpc_socket_path {
            config.grpc.enable_local_socket = true;
            config.grpc.local_socket = path.clone();
        }

        if !config.grpc.enable_http && !config.grpc.enable_local_socket {
            tracing::warn!(
                "gRPC over HTTP and gRPC over local socket are disabled, force enable gRPC over \
                 local socket",
            );
            config.grpc.enable_local_socket = true;
            config.grpc.local_socket = clipcat_base::config::default_unix_domain_socket();
        }

        Ok(config)
    }
}

#[allow(clippy::cognitive_complexity)]
fn run_clipcatd(config: Config, replace: bool) -> Result<(), Error> {
    config.log.registry();

    let pid_file = PidFile::from(config.pid_file.clone());
    if pid_file.exists() {
        let pid = pid_file.try_load()?;
        if replace {
            if let Err(err) = kill_other(pid) {
                tracing::warn!(
                    "Error occurs while trying to terminate another instance, error: {err}"
                );
            };

            let polling_interval = Duration::from_millis(200);
            while pid_file.exists() {
                tracing::warn!(
                    "PID file `{path}` exists, another instance (PID: {pid}) is still running, \
                     sleep for {dur}ms",
                    path = pid_file.path().display(),
                    dur = polling_interval.as_millis()
                );
                // sleep for a while
                std::thread::sleep(polling_interval);
            }
        } else {
            tracing::warn!(
                "Another instance (PID: {pid}) is running, please terminate `{pid}` first"
            );
            return Ok(());
        }
    }

    if config.daemonize {
        daemonize::Daemonize::new().pid_file(pid_file.path()).start()?;
    } else {
        pid_file.create()?;
    }

    let config = clipcat_server::Config::from(config);

    tracing::info!(
        "{} is initializing, pid: {}",
        clipcat_base::DAEMON_PROGRAM_NAME,
        std::process::id()
    );

    tracing::info!("Initializing Tokio runtime");

    let exit_status = match Runtime::new().context(error::InitializeTokioRuntimeSnafu) {
        Ok(runtime) => {
            runtime.block_on(clipcat_server::serve_with_shutdown(config)).map_err(Error::from)
        }
        Err(err) => Err(err),
    };

    if let Err(err) = pid_file.remove() {
        tracing::error!("{err}");
    }

    tracing::info!("{} is shutdown", clipcat_base::DAEMON_PROGRAM_NAME);
    exit_status
}

#[allow(unsafe_code)]
#[inline]
fn kill_other(pid: libc::pid_t) -> Result<(), Error> {
    tracing::info!("Try to terminate another instance (PID: {pid})");
    let ret = unsafe { libc::kill(pid, libc::SIGTERM) };
    if ret != 0 {
        match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ESRCH) => {
                tracing::warn!("Previous instance (PID: {pid}) did not remove its PID file");
            }
            _ => return Err(Error::SendSignalTermination { pid }),
        }
    }
    Ok(())
}

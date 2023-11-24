use std::{io::Write, net::IpAddr, path::PathBuf, time::Duration};

use clap::{CommandFactory, Parser, Subcommand};
use snafu::ResultExt;
use tokio::runtime::Runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    error::{self, Error},
    pid_file::PidFile,
};

#[derive(Parser)]
#[command(name = clipcat::DAEMON_PROGRAM_NAME, author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Option<Commands>,

    #[clap(long = "no-daemon", help = "Do not run as daemon")]
    no_daemon: bool,

    #[clap(long = "replace", short = 'r', help = "Try to replace existing daemon")]
    replace: bool,

    #[clap(long = "config", short = 'c', help = "Specify a configuration file")]
    config_file: Option<PathBuf>,

    #[clap(long = "history-file", help = "Specify a history file")]
    history_file_path: Option<PathBuf>,

    #[clap(long = "grpc-host", help = "Specify gRPC host address")]
    grpc_host: Option<IpAddr>,

    #[clap(long = "grpc-port", help = "Specify gRPC port number")]
    grpc_port: Option<u16>,
}

impl Default for Cli {
    #[inline]
    fn default() -> Self { Self::parse() }
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
            config.grpc.host = host;
        }

        if let Some(port) = self.grpc_port {
            config.grpc.port = port;
        }

        Ok(config)
    }
}

fn run_clipcatd(config: Config, replace: bool) -> Result<(), Error> {
    let daemonize = config.daemonize;
    let pid_file = PidFile::from(config.pid_file.clone());
    if daemonize {
        if pid_file.exists() && replace {
            let pid = pid_file.try_load()?;
            kill_other(pid)?;

            // sleep for a while
            std::thread::sleep(Duration::from_millis(200));
        }

        daemonize::Daemonize::new().pid_file(pid_file.clone_path()).start()?;
    }

    init_tracing(config.log_level);
    let config = clipcat_server::Config::from(config);

    tracing::info!("{} is initializing, pid: {}", clipcat::DAEMON_PROGRAM_NAME, std::process::id());

    tracing::info!("Initializing Tokio runtime");

    let exit_status = match Runtime::new().context(error::InitializeTokioRuntimeSnafu) {
        Ok(runtime) => {
            runtime.block_on(clipcat_server::serve_with_shutdown(config)).map_err(Error::from)
        }
        Err(err) => Err(err),
    };

    if daemonize {
        if let Err(err) = pid_file.remove() {
            tracing::error!("{err}");
        }
    }

    tracing::info!("{} is shutdown", clipcat::DAEMON_PROGRAM_NAME);
    exit_status
}

fn init_tracing(log_level: tracing::Level) {
    // filter
    let filter_layer = tracing_subscriber::filter::LevelFilter::from_level(log_level);

    // format
    let fmt_layer =
        tracing_subscriber::fmt::layer().pretty().with_thread_ids(true).with_thread_names(true);

    // subscriber
    let registry = tracing_subscriber::registry().with(filter_layer).with(fmt_layer);
    match tracing_journald::layer() {
        Ok(layer) => registry.with(layer).init(),
        Err(_err) => {
            registry.init();
        }
    }
}

#[allow(unsafe_code)]
#[inline]
fn kill_other(pid: libc::pid_t) -> Result<(), Error> {
    let ret = unsafe { libc::kill(pid, libc::SIGTERM) };
    if ret != 0 {
        return Err(Error::SendSignalTermination { pid });
    }
    Ok(())
}

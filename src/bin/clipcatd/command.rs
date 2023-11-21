use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use snafu::ResultExt;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use crate::{
    config::{Config, ConfigError},
    error::{self, Error},
    worker,
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
                    .write_all(config_text.as_bytes())
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
    let daemonize = config.daemonize;
    let pid_file = PidFile::from(config.pid_file.clone());
    if daemonize {
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

    {
        use tracing_subscriber::prelude::*;

        let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
        let level_filter = tracing_subscriber::filter::LevelFilter::from_level(config.log_level);

        let registry = tracing_subscriber::registry().with(level_filter).with(fmt_layer);
        match tracing_journald::layer() {
            Ok(layer) => registry.with(layer).init(),
            Err(_err) => {
                registry.init();
            }
        }
    }

    tracing::info!("{} is initializing, pid: {}", clipcat::DAEMON_PROGRAM_NAME, std::process::id());

    let runtime = Runtime::new().context(error::InitializeTokioRuntime)?;
    runtime.block_on(worker::start(config))?;

    if daemonize {
        pid_file.remove()?;
    }

    tracing::info!("{} is shutdown", clipcat::DAEMON_PROGRAM_NAME);
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
        let pid_data = std::fs::read_to_string(self)
            .context(error::ReadPidFile { filename: self.clone_path() })?;
        let pid = pid_data.trim().parse().context(error::ParseProcessId { value: pid_data })?;
        Ok(pid)
    }

    #[inline]
    fn remove(self) -> Result<(), Error> {
        tracing::info!("Remove PID file: {:?}", self.path);
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

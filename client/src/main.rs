mod client;

#[cfg(windows)]
mod windows_service;

use clap::{Parser, Subcommand};
use std::fs;

#[cfg(unix)]
use daemonize::Daemonize;
#[cfg(unix)]
use std::fs::File;

#[derive(Parser)]
#[command(name = "client", version, about = "RFRP Client - 反向代理客户端")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 前台运行客户端
    Start {
        /// Controller 地址（例如 http://controller:3100）
        #[arg(long)]
        controller_url: String,

        /// 客户端 Token
        #[arg(long)]
        token: String,
    },

    /// 停止运行中的守护进程
    Stop {
        /// PID 文件路径
        #[cfg(unix)]
        #[arg(long, default_value = "/var/run/rfrp-client.pid")]
        pid_file: String,

        /// PID 文件路径
        #[cfg(windows)]
        #[arg(long, default_value = "rfrp-client.pid")]
        pid_file: String,
    },

    /// 以守护进程模式运行
    Daemon {
        /// Controller 地址（例如 http://controller:3100）
        #[arg(long)]
        controller_url: String,

        /// 客户端 Token
        #[arg(long)]
        token: String,

        /// PID 文件路径
        #[cfg(unix)]
        #[arg(long, default_value = "/var/run/rfrp-client.pid")]
        pid_file: String,

        /// 日志文件路径
        #[cfg(unix)]
        #[arg(long, default_value = "/var/log/rfrp-client.log")]
        log_file: String,

        /// PID 文件路径
        #[cfg(windows)]
        #[arg(long, default_value = "rfrp-client.pid")]
        pid_file: String,

        /// 日志文件路径
        #[cfg(windows)]
        #[arg(long, default_value = "rfrp-client.log")]
        log_file: String,
    },

    /// 安装为 Windows 服务（仅 Windows 系统）
    #[cfg(windows)]
    InstallService {
        /// Controller 地址（例如 http://controller:3100）
        #[arg(long)]
        controller_url: String,

        /// 客户端 Token
        #[arg(long)]
        token: String,
    },

    /// 卸载 Windows 服务（仅 Windows 系统）
    #[cfg(windows)]
    UninstallService,

    /// 以 Windows 服务模式运行（由 SCM 调用，用户不应直接使用）
    #[cfg(windows)]
    #[command(hide = true)]
    Service {
        /// Controller 地址
        #[arg(long)]
        controller_url: Option<String>,

        /// 客户端 Token
        #[arg(long)]
        token: Option<String>,
    },

    /// 更新到最新版本
    Update,
}

// ─── Unix 入口 ───────────────────────────────────────────

#[cfg(not(windows))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            controller_url,
            token,
        } => {
            client::run_client(controller_url, token).await?;
        }

        Command::Stop { pid_file } => {
            stop_daemon_unix(&pid_file)?;
        }

        Command::Daemon {
            controller_url,
            token,
            pid_file,
            log_file,
        } => {
            println!("启动守护进程模式...");
            println!("PID 文件: {}", pid_file);
            println!("日志文件: {}", log_file);

            let stdout =
                File::create(&log_file).expect("无法创建日志文件");
            let stderr =
                File::create(format!("{}.err", log_file)).expect("无法创建错误日志文件");

            let daemonize = Daemonize::new()
                .pid_file(&pid_file)
                .working_directory(".")
                .stdout(stdout)
                .stderr(stderr);

            match daemonize.start() {
                Ok(_) => println!("守护进程已启动"),
                Err(e) => {
                    eprintln!("启动守护进程失败: {}", e);
                    std::process::exit(1);
                }
            }

            client::run_client(controller_url, token).await?;
        }

        Command::Update => {
            update_binary()?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn stop_daemon_unix(pid_file: &str) -> anyhow::Result<()> {
    let pid_str = fs::read_to_string(pid_file)
        .map_err(|e| anyhow::anyhow!("无法读取 PID 文件 {}: {}", pid_file, e))?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("PID 文件内容无效: {}", e))?;

    let ret = unsafe { libc::kill(pid, libc::SIGTERM) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        // ESRCH = no such process — already stopped
        if err.raw_os_error() == Some(libc::ESRCH) {
            println!("进程 (PID: {}) 已不存在", pid);
        } else {
            return Err(anyhow::anyhow!("停止进程失败 (PID: {}): {}", pid, err));
        }
    } else {
        println!("已发送停止信号到守护进程 (PID: {})", pid);
    }

    fs::remove_file(pid_file).ok();
    Ok(())
}

// ─── Windows 入口 ────────────────────────────────────────

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            controller_url,
            token,
        } => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async { client::run_client(controller_url, token).await })
        }

        Command::Stop { pid_file } => stop_daemon_windows(&pid_file),

        Command::Daemon {
            controller_url,
            token,
            pid_file,
            log_file,
        } => start_daemon_windows(&controller_url, &token, &pid_file, &log_file),

        Command::InstallService {
            controller_url,
            token,
        } => windows_service::install_service(&controller_url, &token),

        Command::UninstallService => windows_service::uninstall_service(),

        Command::Service { .. } => windows_service::run_service(),

        Command::Update => update_binary(),
    }
}

#[cfg(windows)]
fn start_daemon_windows(
    controller_url: &str,
    token: &str,
    pid_file: &str,
    log_file: &str,
) -> anyhow::Result<()> {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let stdout = fs::File::create(log_file)
        .map_err(|e| anyhow::anyhow!("无法创建日志文件 {}: {}", log_file, e))?;
    let stderr = fs::File::create(format!("{}.err", log_file))
        .map_err(|e| anyhow::anyhow!("无法创建错误日志文件: {}", e))?;

    let exe = std::env::current_exe()?;
    let child = std::process::Command::new(&exe)
        .args([
            "start",
            "--controller-url",
            controller_url,
            "--token",
            token,
        ])
        .stdout(stdout)
        .stderr(stderr)
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| anyhow::anyhow!("启动守护进程失败: {}", e))?;

    fs::write(pid_file, child.id().to_string())?;

    println!("守护进程已启动 (PID: {})", child.id());
    println!("PID 文件: {}", pid_file);
    println!("日志文件: {}", log_file);
    println!();
    println!("停止守护进程: client stop --pid-file {}", pid_file);

    Ok(())
}

#[cfg(windows)]
fn stop_daemon_windows(pid_file: &str) -> anyhow::Result<()> {
    let pid_str = fs::read_to_string(pid_file)
        .map_err(|e| anyhow::anyhow!("无法读取 PID 文件 {}: {}", pid_file, e))?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("PID 文件内容无效: {}", e))?;

    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            let err = std::io::Error::last_os_error();
            // ERROR_INVALID_PARAMETER (87) = process does not exist
            if err.raw_os_error() == Some(87) {
                println!("进程 (PID: {}) 已不存在", pid);
                fs::remove_file(pid_file).ok();
                return Ok(());
            }
            return Err(anyhow::anyhow!("无法打开进程 (PID: {}): {}", pid, err));
        }

        let ret = TerminateProcess(handle, 0);
        CloseHandle(handle);

        if ret == 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow::anyhow!("停止进程失败 (PID: {}): {}", pid, err));
        }
    }

    println!("已停止守护进程 (PID: {})", pid);
    fs::remove_file(pid_file).ok();
    Ok(())
}

/// 更新二进制文件到最新版本
fn update_binary() -> anyhow::Result<()> {
    println!("正在检查更新...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("ryunnet")
        .repo_name("rfrp")
        .bin_name("client")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    match status {
        self_update::Status::UpToDate(version) => {
            println!("✓ 已是最新版本: v{}", version);
        }
        self_update::Status::Updated(version) => {
            println!("✓ 成功更新到版本: v{}", version);
            println!("请重启 client 服务以使用新版本");
        }
    }

    Ok(())
}

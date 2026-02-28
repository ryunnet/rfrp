//! Windows 服务支持
//!
//! 提供将 RFRP Client 安装为 Windows 服务的功能

use anyhow::{anyhow, Result};
use std::ffi::OsString;
use std::time::Duration;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "RfrpClient";
const SERVICE_DISPLAY_NAME: &str = "RFRP Client";
const SERVICE_DESCRIPTION: &str = "RFRP 反向代理客户端服务";

define_windows_service!(ffi_service_main, service_main);

/// 安装 Windows 服务
pub fn install_service(controller_url: &str, token: &str, tls_ca_cert: Option<&str>) -> Result<()> {
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service::service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType};

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    // 获取当前可执行文件路径
    let exe_path = std::env::current_exe()?;

    let mut launch_arguments = vec![
        OsString::from("service"),
        OsString::from("--controller-url"),
        OsString::from(controller_url),
        OsString::from("--token"),
        OsString::from(token),
    ];

    if let Some(ca_path) = tls_ca_cert {
        launch_arguments.push(OsString::from("--tls-ca-cert"));
        launch_arguments.push(OsString::from(ca_path));
    }

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: exe_path.clone(),
        launch_arguments,
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;

    // 设置服务描述
    service.set_description(SERVICE_DESCRIPTION)?;

    println!("服务安装成功: {}", SERVICE_DISPLAY_NAME);
    println!("服务名称: {}", SERVICE_NAME);
    println!("启动类型: 自动");
    println!();
    println!("使用以下命令管理服务:");
    println!("  启动服务: sc start {}", SERVICE_NAME);
    println!("  停止服务: sc stop {}", SERVICE_NAME);
    println!("  卸载服务: {} uninstall-service", exe_path.display());

    Ok(())
}

/// 卸载 Windows 服务
pub fn uninstall_service() -> Result<()> {
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service::service::ServiceAccess;

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS)?;

    // 检查服务状态
    let status = service.query_status()?;
    if status.current_state != ServiceState::Stopped {
        return Err(anyhow!(
            "服务正在运行，请先停止服务: sc stop {}",
            SERVICE_NAME
        ));
    }

    service.delete()?;
    println!("服务卸载成功: {}", SERVICE_DISPLAY_NAME);

    Ok(())
}

/// 运行 Windows 服务
pub fn run_service() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .map_err(|e| anyhow!("服务调度失败: {}", e))
}

/// 服务主函数
fn service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service_impl(arguments) {
        // 记录错误到 Windows 事件日志
        eprintln!("服务运行错误: {}", e);
    }
}

/// 服务实现
fn run_service_impl(arguments: Vec<OsString>) -> Result<()> {
    // 解析服务参数
    let mut controller_url = String::new();
    let mut token = String::new();
    let mut tls_ca_cert_path: Option<String> = None;

    let mut i = 0;
    while i < arguments.len() {
        let arg = arguments[i].to_string_lossy();
        match arg.as_ref() {
            "--controller-url" => {
                if i + 1 < arguments.len() {
                    controller_url = arguments[i + 1].to_string_lossy().to_string();
                    i += 1;
                }
            }
            "--token" => {
                if i + 1 < arguments.len() {
                    token = arguments[i + 1].to_string_lossy().to_string();
                    i += 1;
                }
            }
            "--tls-ca-cert" => {
                if i + 1 < arguments.len() {
                    tls_ca_cert_path = Some(arguments[i + 1].to_string_lossy().to_string());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if controller_url.is_empty() || token.is_empty() {
        return Err(anyhow!("缺少必需参数: --controller-url 或 --token"));
    }

    // 加载 CA 证书（如果提供）
    let tls_ca_cert = match tls_ca_cert_path {
        Some(path) => {
            let content = std::fs::read(&path)
                .map_err(|e| anyhow!("读取 CA 证书文件 {} 失败: {}", path, e))?;
            Some(content)
        }
        None => None,
    };

    // 创建事件处理器
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = shutdown_tx.try_send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // 注册服务控制处理器
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // 通知 SCM 服务正在启动
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(1),
        process_id: None,
    })?;

    // 创建 Tokio 运行时
    let runtime = tokio::runtime::Runtime::new()?;

    // 通知 SCM 服务已启动
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // 运行客户端
    runtime.block_on(async {
        tokio::select! {
            result = crate::client::run_client(controller_url, token, tls_ca_cert, None) => {
                if let Err(e) = result {
                    eprintln!("客户端运行错误: {}", e);
                }
            }
            _ = shutdown_rx.recv() => {
                println!("收到停止信号");
            }
        }
    });

    // 通知 SCM 服务已停止
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

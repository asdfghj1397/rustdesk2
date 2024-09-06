use std::fs::OpenOptions;
use std::io::{self, BufRead};
use std::path::Path;

use hbb_common::log;
use url::Url;

// `/etc/hosts` 文件的路径（在 Windows 上可能是 `C:\\Windows\\System32\\drivers\\etc\\hosts`）
const HOSTS_PATH: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";

/// 更新 `/etc/hosts` 文件中的域名解析记录
///
/// # 参数
/// - `domain`: 要查询和修改的域名
/// - `target_ip`: 目标 IP 地址
///
/// # 返回值
/// - `Ok(String)`: 操作成功时，返回操作的结果描述
/// - `Err(io::Error)`: 操作失败时，返回错误信息

pub fn test_socket() {
    use std::net::TcpListener;

    // 绑定地址和端口，创建 TCP 监听器
    let _ = TcpListener::bind("0.0.0.0:17878");
}

pub fn replace_domain_from_hosts(url: &str) -> io::Result<String> {
    log::info!("replace_domain_from_hosts: {}", url);
    let parsed_url = Url::parse(url).expect("无法解析URL");

    if parsed_url.domain() == None {
        return Err(io::Error::new(io::ErrorKind::Unsupported, "url 为 None"));
    }

    // 获取域名
    let domain = parsed_url.domain().unwrap();

    // Step 1: 读取 `/etc/hosts` 文件内容
    let path = Path::new(HOSTS_PATH);
    let file = OpenOptions::new().read(true).open(&path)?;
    let reader = io::BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();

    // Step 2: 遍历每一行，检查是否已经有匹配的域名解析
    for line in reader.lines() {
        let line = line?;
        // 跳过注释行
        if line.starts_with('#') {
            lines.push(line);
            continue;
        }

        // 检查行中是否包含目标域名
        if line.contains(domain) {
            // 将行按空格或tab拆分为 IP 和 域名
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == domain {
                let ip = parts[0];
                let new_url = url.replace(domain, ip);
                return Ok(new_url);
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "未找到服务器IP"))
}

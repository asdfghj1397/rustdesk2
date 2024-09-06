use regex::Regex;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead};
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

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

pub static UPDATED: OnceLock<Mutex<bool>> = OnceLock::new();

fn remove_readonly_attribute(file_path: &str) -> io::Result<()> {
    // 获取文件元数据
    let metadata = fs::metadata(file_path)?;

    // 获取文件属性
    let attributes = metadata.file_attributes();

    // 检查是否设置了只读属性
    if (attributes & 0x00000001) != 0 {
        // 如果有只读属性，则移除它
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(file_path, permissions)?;
        println!("已移除只读属性");
    } else {
        println!("文件没有只读属性");
    }

    Ok(())
}

fn is_valid_ipv4(ip: &str) -> bool {
    let re = Regex::new(r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])$")
        .unwrap();
    re.is_match(ip)
}

pub fn update_hosts_file(domain: &str, target_ip: &str) -> io::Result<String> {
    if (!is_valid_ipv4(target_ip)) {
        return Ok("无效的ip地址".to_owned());
    }

    remove_readonly_attribute(HOSTS_PATH)?;

    // Step 1: 读取 `/etc/hosts` 文件内容
    let path = Path::new(HOSTS_PATH);
    let file = OpenOptions::new().read(true).write(true).open(&path)?;
    let reader = io::BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();
    let mut found = false;
    let mut updated = false;

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
                found = true;
                // 检查是否IP与目标IP一致
                if parts[0] == target_ip {
                    found = true;
                    return Ok(format!(
                        "域名 {} 已经正确解析到 IP {}，无需修改。",
                        domain, target_ip
                    ));
                } else {
                    lines.push(format!("{} {}", target_ip, domain));
                    updated = true;
                    continue;
                }
            }
        }
        lines.push(line);
    }

    // Step 3: 如果未找到指定的域名解析，则添加新的记录
    if !found {
        lines.push(format!("{} {}", target_ip, domain));
        updated = true;
    }

    // Step 4: 如果需要更新文件，则写入修改
    if updated {
        fs::write(HOSTS_PATH, lines.join("\n"))?;
        return Ok(format!(
            "域名 {} 的解析记录已更新为 IP {}。",
            domain, target_ip
        ));
    }

    Ok(format!("域名 {} 的解析记录没有变化。", domain))
}

fn test() -> io::Result<()> {
    // 测试调用
    let domain = "example.com";
    let target_ip = "192.168.1.106";

    match update_hosts_file(domain, target_ip) {
        Ok(message) => println!("{}", message),
        Err(e) => eprintln!("操作失败: {}", e),
    }

    Ok(())
}

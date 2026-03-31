#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_start_chrome() {
        let chrome_path = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
        let port = 9254;
        let user_data_dir = r"C:\chrome\test_rust";
        
        // 创建目录
        std::fs::create_dir_all(user_data_dir).unwrap();
        
        println!("Chrome路径: {}", chrome_path);
        println!("端口: {}", port);
        println!("用户目录: {}", user_data_dir);
        
        // 启动Chrome
        let child = Command::new(chrome_path)
            .args([
                format!("--remote-debugging-port={}", port),
                format!("--user-data-dir={}", user_data_dir),
                "--no-first-run".to_string(),
                "--no-default-browser-check".to_string(),
                "https://www.xiaohongshu.com".to_string(),
            ])
            .spawn();
        
        match child {
            Ok(c) => {
                println!("Chrome启动成功! PID: {}", c.id());
                // 不等待，让Chrome继续运行
                std::mem::forget(c);
            }
            Err(e) => {
                println!("Chrome启动失败: {}", e);
                panic!("Chrome启动失败: {}", e);
            }
        }
        
        // 等待Chrome启动
        std::thread::sleep(std::time::Duration::from_secs(5));
        
        // 检查端口
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{}/json/version", port);
        match client.get(&url).timeout(std::time::Duration::from_secs(5)).send() {
            Ok(resp) => {
                println!("Chrome DevTools: OK, status: {}", resp.status());
                assert!(resp.status().is_success());
            }
            Err(e) => {
                println!("Chrome DevTools: FAILED, error: {}", e);
                panic!("Chrome DevTools连接失败: {}", e);
            }
        }
    }
}

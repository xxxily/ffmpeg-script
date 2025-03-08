#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
use tauri::{command, Manager, Window};

#[command]
fn run_ffmpeg_command(command_type: &str, args: Vec<String>) -> Result<String, String> {
    let result = match command_type {
        "flv2mp4" => {
            let mut cmd = Command::new("node");
            cmd.arg("../bin/flvToMp4.js");  // 修改为正确的相对路径
            for arg in args {
                cmd.arg(arg);
            }
            cmd.output()
        },
        "avm" => {
            let mut cmd = Command::new("node");
            cmd.arg("../bin/audioVideoMerger.js");  // 修改为正确的相对路径
            for arg in args {
                cmd.arg(arg);
            }
            cmd.output()
        },
        _ => return Err("未知命令类型".into()),
    };

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            if !stderr.is_empty() {
                return Err(stderr);
            }
            
            Ok(stdout)
        },
        Err(e) => Err(format!("执行命令失败: {}", e)),
    }
}

#[command]
fn run_ffmpeg_command_realtime(window: Window, command_type: &str, args: Vec<String>) -> Result<(), String> {
    let mut cmd = Command::new("node");
    
    match command_type {
        "flv2mp4" => {
            cmd.arg("../bin/flvToMp4.js");
        },
        "avm" => {
            cmd.arg("../bin/audioVideoMerger.js");
        },
        _ => return Err("未知命令类型".into()),
    };
    
    for arg in args {
        cmd.arg(arg);
    }
    
    // 设置标准输出和标准错误为管道
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => return Err(format!("启动命令失败: {}", e)),
    };
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let window_clone = window.clone();
    
    // 处理标准输出
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = window_clone.emit("command-output", line);
            }
        }
    });
    
    // 处理标准错误
    let window_clone = window.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = window_clone.emit("command-output", format!("错误: {}", line));
            }
        }
    });
    
    // 等待命令执行完成
    thread::spawn(move || {
        match child.wait() {
            Ok(status) => {
                let message = if status.success() {
                    "命令执行完成".to_string()
                } else {
                    format!("命令执行失败，退出码: {:?}", status.code())
                };
                let _ = window.emit("command-output", message);
            },
            Err(e) => {
                let _ = window.emit("command-output", format!("等待命令完成时出错: {}", e));
            }
        }
    });
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_ffmpeg_command, run_ffmpeg_command_realtime])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
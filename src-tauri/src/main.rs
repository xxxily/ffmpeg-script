#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
use std::path::{Path, PathBuf};
use std::fs;
use tauri::{command, Manager, Window};

// 检查 FFmpeg 是否已安装
fn check_ffmpeg_installed() -> Result<String, String> {
  let output = Command::new("ffmpeg")
      .arg("-version")
      .output()
      .map_err(|_| "FFmpeg 未安装".to_string())?;
  
  if output.status.success() {
      // 提取版本信息
      let version_output = String::from_utf8_lossy(&output.stdout).to_string();
      let version_line = version_output.lines().next().unwrap_or("");
      Ok(version_line.to_string())
  } else {
      Err("FFmpeg 未安装或无法运行".to_string())
  }
}

#[command]
fn get_ffmpeg_version() -> Result<String, String> {
    check_ffmpeg_installed()
}

// FLV 转 MP4 功能
fn flv_to_mp4(file_path: &str) -> Result<(), String> {
  let file_path = Path::new(file_path);
  let file_info = file_path.parent().ok_or("无法获取文件目录")?;
  
  let output_path = file_path.with_extension("mp4");
  
  let mut cmd = Command::new("ffmpeg");
  cmd.args(&[
      "-y", 
      "-i", 
      file_path.to_str().ok_or("文件路径转换失败")?, 
      "-vcodec", 
      "copy", 
      "-acodec", 
      "copy", 
      output_path.to_str().ok_or("输出路径转换失败")?
  ]);
  
  let output = cmd.output().map_err(|e| format!("执行命令失败: {}", e))?;
  
  if !output.status.success() {
      return Err(String::from_utf8_lossy(&output.stderr).to_string());
  }
  
  Ok(())
}

// 音视频合并功能
fn audio_video_merger(audio_file_path: &str, video_file_path: &str) -> Result<(), String> {
  let video_path = Path::new(video_file_path);
  let file_info = video_path.parent().ok_or("无法获取文件目录")?;
  
  let output_path = video_path.to_str().ok_or("路径转换失败")?
      .replace("_video.", ".");
  
  let mut cmd = Command::new("ffmpeg");
  cmd.args(&[
      "-i", 
      video_file_path, 
      "-i", 
      audio_file_path, 
      "-vcodec", 
      "copy", 
      "-acodec", 
      "copy", 
      &output_path
  ]);
  
  let output = cmd.output().map_err(|e| format!("执行命令失败: {}", e))?;
  
  if !output.status.success() {
      return Err(String::from_utf8_lossy(&output.stderr).to_string());
  }
  
  Ok(())
}

// 处理 FLV 转 MP4 的主要逻辑
fn handle_flv_to_mp4(
  cwd: &str, 
  output_dir: &str, 
  watch: bool, 
  archive: bool, 
  remove: bool, 
  debug: bool,
  timeout: u64,
  window: Option<&Window>
) -> Result<(), String> {
  let input_dir = Path::new(cwd);
  let output_dir = Path::new(output_dir);
  
  // 确保输出目录存在
  if !output_dir.exists() {
      fs::create_dir_all(output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
      let msg = format!("[flv-to-mp4] 转换结果存放目录创建成功：{}", output_dir.display());
      if let Some(window) = window {
          let _ = window.emit("command-output", &msg);
      } else {
          println!("{}", msg);
      }
  }
  
  // 获取所有 flv 文件
  let entries = fs::read_dir(input_dir).map_err(|e| format!("读取目录失败: {}", e))?;
  let mut flv_files = Vec::new();
  
  for entry in entries {
      if let Ok(entry) = entry {
          let path = entry.path();
          if path.is_file() && path.extension().map_or(false, |ext| ext == "flv") {
              flv_files.push(path);
          }
      }
  }
  
  if flv_files.is_empty() {
      let msg = format!("[flv-to-mp4] {} 当前目录下未发现flv文件", input_dir.display());
      if let Some(window) = window {
          let _ = window.emit("command-output", &msg);
      } else {
          println!("{}", msg);
      }
      return Ok(());
  }
  
  // 获取已转换的 mp4 文件名
  let mut mp4_file_names = Vec::new();
  if output_dir.exists() {
      let entries = fs::read_dir(output_dir).map_err(|e| format!("读取输出目录失败: {}", e))?;
      for entry in entries {
          if let Ok(entry) = entry {
              let path = entry.path();
              if path.is_file() && path.extension().map_or(false, |ext| ext == "mp4") {
                  if let Some(name) = path.file_stem() {
                      mp4_file_names.push(name.to_string_lossy().to_string());
                  }
              }
          }
      }
  }
  
  // 处理每个 flv 文件
  for flv_file in flv_files {
      let file_name = flv_file.file_stem().ok_or("无法获取文件名")?
          .to_string_lossy().to_string();
      
      // 检查是否已经转换过
      if mp4_file_names.contains(&file_name) {
          let msg = format!("[flv-to-mp4] {}的mp4版本的文件已存在", file_name);
          if let Some(window) = window {
              let _ = window.emit("command-output", &msg);
          } else {
              println!("{}", msg);
          }
          continue;
      }
      
      // 如果是监视模式，检查文件是否正在被修改
      if watch {
          if let Ok(metadata) = fs::metadata(&flv_file) {
              if let Ok(modified) = metadata.modified() {
                  let now = std::time::SystemTime::now();
                  if let Ok(duration) = now.duration_since(modified) {
                      if duration.as_secs() < 60 {
                          if debug {
                              let msg = format!("[flv-to-mp4] {} 文件内容最近仍在修改，可能还未录制结束，暂时跳过", file_name);
                              if let Some(window) = window {
                                  let _ = window.emit("command-output", &msg);
                              } else {
                                  println!("{}", msg);
                              }
                          }
                          continue;
                      }
                  }
              }
          }
      }
      
      // 开始转换
      let msg = format!("[flv-to-mp4] 正在转换：{}", flv_file.display());
      if let Some(window) = window {
          let _ = window.emit("command-output", &msg);
      } else {
          println!("{}", msg);
      }
      
      let start_time = std::time::Instant::now();
      
      match flv_to_mp4(flv_file.to_str().ok_or("文件路径转换失败")?) {
          Ok(_) => {
              let mp4_file_path = flv_file.with_extension("mp4");
              let mut dest_path = PathBuf::from(output_dir);
              
              // 如果需要归档
              if archive {
                if let Ok(metadata) = fs::metadata(&flv_file) {
                    if let Ok(modified) = metadata.modified() {
                        // 使用 chrono 的正确方法将 SystemTime 转换为 DateTime
                        let datetime = chrono::DateTime::<chrono::Local>::from(modified);
                        let date_str = format!("{}-{}.{}", 
                            datetime.format("%Y"), 
                            datetime.format("%m"), 
                            datetime.format("%d")
                        );
                        dest_path = dest_path.join(date_str);
                        fs::create_dir_all(&dest_path).map_err(|e| format!("创建归档目录失败: {}", e))?;
                  }
                }
              }
              
              dest_path = dest_path.join(format!("{}.mp4", file_name));
              
              // 移动文件到目标位置
              fs::rename(&mp4_file_path, &dest_path).map_err(|e| format!("移动文件失败: {}", e))?;
              
              let duration = start_time.elapsed().as_secs_f32();
              let msg = format!("[flv-to-mp4] 转换成功，耗时：{:.2}s", duration);
              if let Some(window) = window {
                  let _ = window.emit("command-output", &msg);
              } else {
                  println!("{}", msg);
              }
              
              // 如果需要删除源文件
              if remove {
                  if let Err(e) = fs::remove_file(&flv_file) {
                      let msg = format!("[flv-to-mp4] 删除源文件失败: {}", e);
                      if let Some(window) = window {
                          let _ = window.emit("command-output", &msg);
                      } else {
                          println!("{}", msg);
                      }
                  }
              }
          },
          Err(e) => {
              let msg = format!("[flv-to-mp4] {}转换失败：\n{}", file_name, e);
              if let Some(window) = window {
                  let _ = window.emit("command-output", &msg);
              } else {
                  println!("{}", msg);
              }
          }
      }
  }
  
  Ok(())
}

// 处理音视频合并的主要逻辑
fn handle_audio_video_merger(cwd: &str, window: Option<&Window>) -> Result<(), String> {
  let cwd_path = Path::new(cwd);
  let output_dir = cwd_path.join("audio-video-merger");
  
  // 确保输出目录存在
  if !output_dir.exists() {
      fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
  }
  
  // 获取所有音频和视频文件
  let entries = fs::read_dir(cwd_path).map_err(|e| format!("读取目录失败: {}", e))?;
  let mut audio_files = Vec::new();
  let mut video_files = Vec::new();
  
  for entry in entries {
      if let Ok(entry) = entry {
          let path = entry.path();
          if path.is_file() {
              let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
              if file_name.contains("_audio.") {
                  audio_files.push(path);
              } else if file_name.contains("_video.") {
                  video_files.push(path);
              }
          }
      }
  }
  
  if audio_files.is_empty() {
      let msg = format!("[Audio-Video-Merger] {} 当前目录下未发现可合并的音视频文件", cwd_path.display());
      if let Some(window) = window {
          let _ = window.emit("command-output", &msg);
      } else {
          println!("{}", msg);
      }
      return Ok(());
  }
  
  // 处理每个音频文件
  for audio_file in audio_files {
      let file_info = audio_file.file_stem().ok_or("无法获取文件名")?
          .to_string_lossy().to_string();
      let audio_file_name = file_info.replace("_audio", "");
      
      // 查找匹配的视频文件
      let mut matching_video_files = Vec::new();
      for video_file in &video_files {
          let video_file_name = video_file.to_string_lossy().to_string();
          if video_file_name.contains(&format!("{}_video.", audio_file_name)) {
              matching_video_files.push(video_file.clone());
          }
      }
      
      if matching_video_files.is_empty() {
          let msg = format!("[Audio-Video-Merger] 未找到【{}】对应的视频文件", audio_file.display());
          if let Some(window) = window {
              let _ = window.emit("command-output", &msg);
          } else {
              println!("{}", msg);
          }
          continue;
      }
      
      let start_time = std::time::Instant::now();
      let video_file = &matching_video_files[0];
      let video_file_info = video_file.file_name().ok_or("无法获取视频文件名")?
          .to_string_lossy().to_string();
      let result_video_file_name = video_file_info.replace("_video", "");
      let result_video_file_path = output_dir.join(&result_video_file_name);
      
      // 检查输出文件是否已存在
      if result_video_file_path.exists() {
          let msg = format!("[Audio-Video-Merger] 【{}】的合并文件已存在", audio_file_name);
          if let Some(window) = window {
              let _ = window.emit("command-output", &msg);
          } else {
              println!("{}", msg);
          }
          continue;
      }
      
      // 移除可能存在的旧文件
      let old_result_path = video_file.parent().unwrap_or(Path::new("")).join(&result_video_file_name);
      if old_result_path.exists() {
          if let Err(e) = fs::remove_file(&old_result_path) {
              let msg = format!("[Audio-Video-Merger] 删除旧文件失败: {}", e);
              if let Some(window) = window {
                  let _ = window.emit("command-output", &msg);
              } else {
                  println!("{}", msg);
              }
          }
      }
      
        // 开始合并
        let msg = format!("[Audio-Video-Merger] 正在合并：{}", audio_file_name);
        if let Some(window) = window {
            let _ = window.emit("command-output", &msg);
        } else {
            println!("{}", msg);
        }
        
        match audio_video_merger(
            audio_file.to_str().ok_or("音频文件路径转换失败")?,
            video_file.to_str().ok_or("视频文件路径转换失败")?
        ) {
            Ok(_) => {
                // 移动合并后的文件到目标位置
                let result_path = video_file.parent().unwrap_or(Path::new("")).join(&result_video_file_name);
                if let Err(e) = fs::rename(&result_path, &result_video_file_path) {
                    let msg = format!("[Audio-Video-Merger] 移动合并文件失败: {}", e);
                    if let Some(window) = window {
                        let _ = window.emit("command-output", &msg);
                    } else {
                        println!("{}", msg);
                    }
                    continue;
                }
                
                let duration = start_time.elapsed().as_secs_f32();
                let msg = format!("[Audio-Video-Merger] 合并成功，耗时：{:.2}s", duration);
                if let Some(window) = window {
                    let _ = window.emit("command-output", &msg);
                } else {
                    println!("{}", msg);
                }
            },
            Err(e) => {
                let msg = format!("[Audio-Video-Merger] {}合并失败：\n{}", audio_file_name, e);
                if let Some(window) = window {
                    let _ = window.emit("command-output", &msg);
                } else {
                    println!("{}", msg);
                }
            }
        }
    }
    
    Ok(())
}

#[command]
fn run_ffmpeg_command(command_type: &str, args: Vec<String>) -> Result<String, String> {
    match command_type {
        "flv2mp4" => {
            // 解析参数
            let mut cwd = std::env::current_dir()
                .map_err(|e| format!("获取当前目录失败: {}", e))?
                .to_string_lossy().to_string();
            let mut output_dir = String::new();
            let mut watch = false;
            let mut archive = false;
            let mut remove = false;
            let mut debug = false;
            let mut timeout = 30;
            
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "-c" => {
                        if i + 1 < args.len() {
                            cwd = args[i + 1].clone();
                            i += 1;
                        }
                    },
                    "-o" => {
                        if i + 1 < args.len() {
                            output_dir = args[i + 1].clone();
                            i += 1;
                        }
                    },
                    "-w" => watch = true,
                    "-a" => archive = true,
                    "-r" => remove = true,
                    "-d" => debug = true,
                    "-t" => {
                        if i + 1 < args.len() {
                            timeout = args[i + 1].parse().unwrap_or(30);
                            i += 1;
                        }
                    },
                    _ => {}
                }
                i += 1;
            }
            
            // 如果没有指定输出目录，使用默认值
            if output_dir.is_empty() {
                output_dir = format!("{}/flv-to-mp4", cwd);
            }
            
            // 执行转换
            let mut output = String::new();
            match handle_flv_to_mp4(&cwd, &output_dir, watch, archive, remove, debug, timeout, None) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
            
            Ok(output)
        },
        "avm" => {
            // 解析参数
            let cwd = if !args.is_empty() {
                args[0].clone()
            } else {
                std::env::current_dir()
                    .map_err(|e| format!("获取当前目录失败: {}", e))?
                    .to_string_lossy().to_string()
            };
            
            // 执行合并
            let mut output = String::new();
            match handle_audio_video_merger(&cwd, None) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
            
            Ok(output)
        },
        _ => Err("未知命令类型".into()),
    }
}

#[command]
fn run_ffmpeg_command_realtime(window: Window, command_type: &str, args: Vec<String>) -> Result<(), String> {
    match command_type {
        "flv2mp4" => {
            // 解析参数
            let mut cwd = std::env::current_dir()
                .map_err(|e| format!("获取当前目录失败: {}", e))?
                .to_string_lossy().to_string();
            let mut output_dir = String::new();
            let mut watch = false;
            let mut archive = false;
            let mut remove = false;
            let mut debug = false;
            let mut timeout = 30;
            
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "-c" => {
                        if i + 1 < args.len() {
                            cwd = args[i + 1].clone();
                            i += 1;
                        }
                    },
                    "-o" => {
                        if i + 1 < args.len() {
                            output_dir = args[i + 1].clone();
                            i += 1;
                        }
                    },
                    "-w" => watch = true,
                    "-a" => archive = true,
                    "-r" => remove = true,
                    "-d" => debug = true,
                    "-t" => {
                        if i + 1 < args.len() {
                            timeout = args[i + 1].parse().unwrap_or(30);
                            i += 1;
                        }
                    },
                    _ => {}
                }
                i += 1;
            }
            
            // 如果没有指定输出目录，使用默认值
            if output_dir.is_empty() {
                output_dir = format!("{}/flv-to-mp4", cwd);
            }
            
            // 在新线程中执行转换，以便实时输出
            let window_clone = window.clone();
            thread::spawn(move || {
                if watch {
                    // 如果是监视模式，需要循环执行
                    let mut watch_count = 0;
                    loop {
                        match handle_flv_to_mp4(&cwd, &output_dir, watch, archive, remove, debug, timeout, Some(&window_clone)) {
                            Ok(_) => {},
                            Err(e) => {
                                let _ = window_clone.emit("command-output", format!("执行出错: {}", e));
                            }
                        }
                        
                        watch_count += 1;
                        let msg = format!("[flv-to-mp4][Watching][{}]=>[{}] 已执行 {} 次", cwd, output_dir, watch_count);
                        let _ = window_clone.emit("command-output", msg);
                        
                        // 等待指定时间后再次执行
                        std::thread::sleep(std::time::Duration::from_secs(timeout));
                    }
                } else {
                    // 单次执行
                    match handle_flv_to_mp4(&cwd, &output_dir, watch, archive, remove, debug, timeout, Some(&window_clone)) {
                        Ok(_) => {
                            let _ = window_clone.emit("command-output", "命令执行完成");
                        },
                        Err(e) => {
                            let _ = window_clone.emit("command-output", format!("执行出错: {}", e));
                        }
                    }
                }
            });
            
            Ok(())
        },
        "avm" => {
            // 解析参数
            let cwd = if !args.is_empty() {
                args[0].clone()
            } else {
                std::env::current_dir()
                    .map_err(|e| format!("获取当前目录失败: {}", e))?
                    .to_string_lossy().to_string()
            };
            
            // 在新线程中执行合并，以便实时输出
            let window_clone = window.clone();
            thread::spawn(move || {
                match handle_audio_video_merger(&cwd, Some(&window_clone)) {
                    Ok(_) => {
                        let _ = window_clone.emit("command-output", "命令执行完成");
                    },
                    Err(e) => {
                        let _ = window_clone.emit("command-output", format!("执行出错: {}", e));
                    }
                }
            });
            
            Ok(())
        },
        _ => Err("未知命令类型".into()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
          run_ffmpeg_command, 
          run_ffmpeg_command_realtime,
          get_ffmpeg_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
<template>
  <div class="container">
    <h1>FFmpeg Script 管理界面</h1>
    
    <!-- 添加 FFmpeg 版本信息显示 -->
    <div class="ffmpeg-info" v-if="ffmpegVersion">
      <span>FFmpeg 版本: {{ ffmpegVersion }}</span>
    </div>
    <div class="ffmpeg-warning" v-if="ffmpegError">
      <span>{{ ffmpegError }}</span>
      <p>请安装 FFmpeg 后再使用本工具。安装命令：</p>
      <pre>brew install ffmpeg</pre>
    </div>
    
    <div class="tabs">
      <button 
        :class="{ active: activeTab === 'flv2mp4' }" 
        @click="activeTab = 'flv2mp4'"
      >
        FLV 转 MP4
      </button>
      <button 
        :class="{ active: activeTab === 'avm' }" 
        @click="activeTab = 'avm'"
      >
        音视频合并
      </button>
    </div>
    
    <div class="tab-content">
      <!-- FLV 转 MP4 面板 -->
      <div v-if="activeTab === 'flv2mp4'" class="panel">
        <div class="form-group">
          <label>工作目录:</label>
          <div class="input-group">
            <input v-model="flv2mp4.cwd" type="text" placeholder="请选择工作目录" />
            <button @click="selectDirectory('flv2mp4', 'cwd')">浏览...</button>
          </div>
        </div>
        
        <div class="form-group">
          <label>输出目录:</label>
          <div class="input-group">
            <input v-model="flv2mp4.output" type="text" placeholder="请选择输出目录" />
            <button @click="selectDirectory('flv2mp4', 'output')">浏览...</button>
          </div>
        </div>
        
        <div class="options">
          <label>
            <input type="checkbox" v-model="flv2mp4.watch" />
            持续检查需要转换的文件
          </label>
          
          <label>
            <input type="checkbox" v-model="flv2mp4.archive" />
            自动归档（按日期）
          </label>
          
          <label>
            <input type="checkbox" v-model="flv2mp4.remove" />
            转换完成后删除源文件
          </label>
          
          <label>
            <input type="checkbox" v-model="flv2mp4.debug" />
            输出调试信息
          </label>
        </div>
        
        <div v-if="flv2mp4.watch" class="form-group">
          <label>检查间隔 (秒):</label>
          <input v-model="flv2mp4.timeout" type="number" min="1" />
        </div>
        
        <button class="primary" @click="runCommand('flv2mp4')" :disabled="flv2mp4.isRunning">
          {{ flv2mp4.isRunning ? '正在执行...' : '开始转换' }}
        </button>
        
        <!-- FLV 转 MP4 输出日志 -->
        <div class="output-panel">
          <h3>FLV 转 MP4 输出日志</h3>
          <div class="output-content" ref="flv2mp4OutputContent">
            <pre>{{ flv2mp4Output }}</pre>
          </div>
          <button @click="clearOutput('flv2mp4')">清空日志</button>
        </div>
      </div>
      
      <!-- 音视频合并面板 -->
      <div v-if="activeTab === 'avm'" class="panel">
        <div class="form-group">
          <label>工作目录:</label>
          <div class="input-group">
            <input v-model="avm.cwd" type="text" placeholder="请选择工作目录" />
            <button @click="selectDirectory('avm', 'cwd')">浏览...</button>
          </div>
        </div>
        
        <button class="primary" @click="runCommand('avm')" :disabled="avm.isRunning">
          {{ avm.isRunning ? '正在执行...' : '开始合并' }}
        </button>
        
        <!-- 音视频合并输出日志 -->
        <div class="output-panel">
          <h3>音视频合并输出日志</h3>
          <div class="output-content" ref="avmOutputContent">
            <pre>{{ avmOutput }}</pre>
          </div>
          <button @click="clearOutput('avm')">清空日志</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event'

export default {
  setup() {
    const activeTab = ref('flv2mp4')
    const flv2mp4Output = ref('')
    const avmOutput = ref('')
    const flv2mp4OutputContent = ref(null)
    const avmOutputContent = ref(null)
    const ffmpegVersion = ref('')
    const ffmpegError = ref('')
    
    const flv2mp4 = ref({
      cwd: '',
      output: '',
      watch: false,
      archive: false,
      remove: false,
      debug: false,
      timeout: 30,
      isRunning: false
    })
    
    const avm = ref({
      cwd: '',
      isRunning: false
    })
    
    // 监听输出变化，自动滚动到底部
    watch(flv2mp4Output, () => {
      setTimeout(() => {
        if (flv2mp4OutputContent.value) {
          flv2mp4OutputContent.value.scrollTop = flv2mp4OutputContent.value.scrollHeight
        }
      }, 100)
    })
    
    watch(avmOutput, () => {
      setTimeout(() => {
        if (avmOutputContent.value) {
          avmOutputContent.value.scrollTop = avmOutputContent.value.scrollHeight
        }
      }, 100)
    })
    
    // 监听来自Rust后端的实时输出事件
    onMounted(async () => {
      await listen('command-output', (event) => {
        // 根据命令类型将输出添加到对应的日志区域
        const payload = event.payload
        if (typeof payload === 'string') {
          // 根据输出内容前缀判断命令类型
          if (payload.includes('[flv-to-mp4]')) {
            appendOutput('flv2mp4', payload)
          } else if (payload.includes('[Audio-Video-Merger]')) {
            appendOutput('avm', payload)
          } else {
            // 如果无法判断，则根据当前正在运行的命令类型添加
            if (flv2mp4.value.isRunning) {
              appendOutput('flv2mp4', payload)
            } else if (avm.value.isRunning) {
              appendOutput('avm', payload)
            } else {
              // 如果都没有运行，则添加到当前活动标签
              appendOutput(activeTab.value, payload)
            }
          }
          
          // 检查输出中是否包含命令执行完成的信息
          if (payload.includes('命令执行完成')) {
            if (payload.includes('[flv-to-mp4]') || 
                (flv2mp4.value.isRunning && !payload.includes('[Audio-Video-Merger]'))) {
              flv2mp4.value.isRunning = false
            } else if (payload.includes('[Audio-Video-Merger]') || 
                      (avm.value.isRunning && !payload.includes('[flv-to-mp4]'))) {
              avm.value.isRunning = false
            }
          }
        }
      })
      
      // 监听命令执行完成事件
      try {
        await listen('command-complete', (event) => {
          const { commandType } = event.payload || {}
          if (commandType === 'flv2mp4') {
            flv2mp4.value.isRunning = false
          } else if (commandType === 'avm') {
            avm.value.isRunning = false
          }
        })
      } catch (err) {
        console.error('监听命令完成事件失败:', err)
      }
      
      // 检查 FFmpeg
      await checkFFmpeg()
    })
    
    const selectDirectory = async (tab, field) => {
      try {
        const selected = await open({
          directory: true,
          multiple: false,
          title: '选择目录'
        })
        
        if (selected) {
          if (tab === 'flv2mp4') {
            flv2mp4.value[field] = selected
          } else if (tab === 'avm') {
            avm.value[field] = selected
          }
        }
      } catch (err) {
        appendOutput(tab, `选择目录出错: ${err}`)
      }
    }
    
    const appendOutput = (commandType, text) => {
      if (commandType === 'flv2mp4') {
        flv2mp4Output.value += text + '\n'
      } else if (commandType === 'avm') {
        avmOutput.value += text + '\n'
      }
    }
    
    const clearOutput = (commandType) => {
      if (commandType === 'flv2mp4') {
        flv2mp4Output.value = ''
      } else if (commandType === 'avm') {
        avmOutput.value = ''
      }
    }
    
    const runCommand = async (commandType) => {
      try {
        // 检查对应标签页的运行状态
        if (commandType === 'flv2mp4' && flv2mp4.value.isRunning) {
          appendOutput('flv2mp4', '已有 FLV 转 MP4 命令正在执行，请等待完成...')
          return
        } else if (commandType === 'avm' && avm.value.isRunning) {
          appendOutput('avm', '已有音视频合并命令正在执行，请等待完成...')
          return
        }
        
        // 设置对应标签页的运行状态
        if (commandType === 'flv2mp4') {
          flv2mp4.value.isRunning = true
          appendOutput('flv2mp4', `正在执行 ${commandType} 命令...`)
        } else if (commandType === 'avm') {
          avm.value.isRunning = true
          appendOutput('avm', `正在执行 ${commandType} 命令...`)
        }
        
        const args = []
        if (commandType === 'flv2mp4') {
          if (flv2mp4.value.cwd) args.push('-c', flv2mp4.value.cwd)
          if (flv2mp4.value.output) args.push('-o', flv2mp4.value.output)
          if (flv2mp4.value.watch) args.push('-w')
          if (flv2mp4.value.archive) args.push('-a')
          if (flv2mp4.value.remove) args.push('-r')
          if (flv2mp4.value.debug) args.push('-d')
          if (flv2mp4.value.watch && flv2mp4.value.timeout) {
            args.push('-t', flv2mp4.value.timeout.toString())
          }
        } else if (commandType === 'avm') {
          if (avm.value.cwd) args.push(avm.value.cwd)
        }
        
        // 使用新的实时输出命令
        await invoke('run_ffmpeg_command_realtime', {
          commandType,
          args
        })
        
        // 如果不是监视模式，则在命令启动后立即重置状态
        // 监视模式下的状态重置由后端通过事件通知
        if (commandType === 'flv2mp4' && !flv2mp4.value.watch) {
          setTimeout(() => {
            flv2mp4.value.isRunning = false
          }, 1000)
        } else if (commandType === 'avm') {
          // 音视频合并命令执行后重置状态
          setTimeout(() => {
            avm.value.isRunning = false
          }, 1000)
        }
      } catch (error) {
        const errorMsg = `执行出错: ${error}`
        appendOutput(commandType, errorMsg)
        
        // 重置运行状态
        if (commandType === 'flv2mp4') {
          flv2mp4.value.isRunning = false
        } else if (commandType === 'avm') {
          avm.value.isRunning = false
        }
      }
    }
    
    // 检查 FFmpeg 是否已安装
    const checkFFmpeg = async () => {
      try {
        const version = await invoke('get_ffmpeg_version')
        ffmpegVersion.value = version
        ffmpegError.value = ''
      } catch (error) {
        ffmpegError.value = error
        ffmpegVersion.value = ''
        appendOutput('flv2mp4', `FFmpeg 检查失败: ${error}`)
      }
    }
    
    // 在组件挂载时检查 FFmpeg
    onMounted(async () => {
      // 检查 FFmpeg
      await checkFFmpeg()
    })
    
    return {
      activeTab,
      flv2mp4,
      avm,
      flv2mp4Output,
      avmOutput,
      flv2mp4OutputContent,
      avmOutputContent,
      ffmpegVersion,
      ffmpegError,
      selectDirectory,
      runCommand,
      clearOutput
    }
  }
}
</script>

<style>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
}

h1 {
  text-align: center;
  color: #333;
  margin-bottom: 30px;
}

.tabs {
  display: flex;
  margin-bottom: 20px;
  border-bottom: 1px solid #ddd;
}

.tabs button {
  padding: 10px 20px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  border-bottom: 2px solid transparent;
}

.tabs button.active {
  border-bottom: 2px solid #4CAF50;
  color: #4CAF50;
}

.panel {
  background: #f9f9f9;
  padding: 20px;
  border-radius: 5px;
  margin-bottom: 20px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.input-group {
  display: flex;
}

.input-group input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px 0 0 4px;
}

.input-group button {
  padding: 8px 15px;
  background: #eee;
  border: 1px solid #ddd;
  border-left: none;
  border-radius: 0 4px 4px 0;
  cursor: pointer;
}

.options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 15px;
}

.options label {
  display: flex;
  align-items: center;
  gap: 5px;
}

button.primary {
  background: #4CAF50;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  width: 100%;
}

button.primary:hover {
  background: #45a049;
}

button.primary:disabled {
  background: #cccccc;
  cursor: not-allowed;
}

.output-panel {
  margin-top: 30px;
  border: 1px solid #ddd;
  border-radius: 5px;
  overflow: hidden;
}

.output-panel h3 {
  margin: 0;
  padding: 10px;
  background: #f5f5f5;
  border-bottom: 1px solid #ddd;
}

.output-content {
  height: 300px;
  overflow-y: auto;
  background: #f8f8f8;
  padding: 10px;
}

.output-content pre {
  margin: 0;
  white-space: pre-wrap;
  font-family: monospace;
}

.output-panel button {
  width: 100%;
  padding: 10px;
  background: #f5f5f5;
  border: none;
  border-top: 1px solid #ddd;
  cursor: pointer;
}

.output-panel button:hover {
  background: #e5e5e5;
}

.ffmpeg-info {
  background-color: #e7f7e7;
  padding: 10px;
  margin-bottom: 15px;
  border-radius: 4px;
  border-left: 4px solid #4CAF50;
}

.ffmpeg-warning {
  background-color: #fff3e0;
  padding: 10px;
  margin-bottom: 15px;
  border-radius: 4px;
  border-left: 4px solid #ff9800;
}

.ffmpeg-warning pre {
  background: #f5f5f5;
  padding: 8px;
  border-radius: 4px;
  margin-top: 5px;
}
</style>
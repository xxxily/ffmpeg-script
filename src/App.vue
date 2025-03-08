<template>
  <div class="container">
    <h1>FFmpeg Script 管理界面</h1>
    
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
        
        <button class="primary" @click="runCommand('flv2mp4')">开始转换</button>
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
        
        <button class="primary" @click="runCommand('avm')">开始合并</button>
      </div>
    </div>
    
    <div class="output-panel">
      <h3>输出日志</h3>
      <div class="output-content" ref="outputContent">
        <pre>{{ output }}</pre>
      </div>
      <button @click="clearOutput">清空日志</button>
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
    const output = ref('')
    const outputContent = ref(null)
    const isRunning = ref(false)
    
    const flv2mp4 = ref({
      cwd: '',
      output: '',
      watch: false,
      archive: false,
      remove: false,
      debug: false,
      timeout: 30
    })
    
    const avm = ref({
      cwd: ''
    })
    
    // 监听输出变化，自动滚动到底部
    watch(output, () => {
      setTimeout(() => {
        if (outputContent.value) {
          outputContent.value.scrollTop = outputContent.value.scrollHeight
        }
      }, 100)
    })
    
    // 监听来自Rust后端的实时输出事件
    onMounted(async () => {
      await listen('command-output', (event) => {
        appendOutput(event.payload)
      })
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
        appendOutput(`选择目录出错: ${err}`)
      }
    }
    
    const appendOutput = (text) => {
      output.value += text + '\n'
    }
    
    const clearOutput = () => {
      output.value = ''
    }
    
    const runCommand = async (commandType) => {
      try {
        if (isRunning.value) {
          appendOutput('已有命令正在执行，请等待完成...')
          return
        }
        
        isRunning.value = true
        appendOutput(`正在执行 ${commandType} 命令...`)
        
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
      } catch (error) {
        appendOutput(`执行出错: ${error}`)
        isRunning.value = false
      }
    }
    
    return {
      activeTab,
      flv2mp4,
      avm,
      output,
      outputContent,
      isRunning,
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
</style>
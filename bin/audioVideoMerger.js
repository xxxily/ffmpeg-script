#!/usr/bin/env node
const execSync = require('child_process').execSync
const path = require('path')
const fs = require('fs-extra')
const fg = require('fast-glob')
const tipsHead = '[Audio-Video-Merger]'

/**
 * 使用ffmpeg将音频和视频合并在一起，注意该操作为同步操作，会阻塞脚本的运行
 * 参考：https://blog.csdn.net/Gary__123456/article/details/112310092
 * ffmpeg -i out.mp4 -i out.aac -vcodec copy -acodec copy new.mp4
 * @param {*} filePath
 */
function audioVideoMerger (audioFilePath, videoFilePath) {
  const fileInfo = path.parse(videoFilePath)
  const command = `ffmpeg -i "${videoFilePath}" -i "${audioFilePath}" -vcodec copy -acodec copy "${videoFilePath.replace('_video.', '.')}"`

  return execSync(command, {
    cwd: fileInfo.dir
  })
}

async function main (cwd) {
  cwd = cwd || __dirname

  const globOpts = {
    cwd,

    // 输出绝对地址
    absolute: true
  }

  const audioFiles = await fg(['*_audio.*'], globOpts)
  const videoFiles = await fg(['*_video.*'], globOpts)

  if(!audioFiles.length){
    console.log(`${tipsHead} ${cwd}`)
    console.log(`${tipsHead} 当前目录下未发现可合并的音视频文件`)
    return false
  }

  fs.ensureDirSync(path.join(cwd, './audio-video-merger'))

  function getMatchVideoFiles(audioFileName){
    const matchVideoFiles = []

    videoFiles.forEach(videoFile => {
      if(videoFile.includes(`${audioFileName}_video.`)) {
        matchVideoFiles.push(videoFile)
      }
    })

    return matchVideoFiles
  }

  audioFiles.forEach((audioFile) => {
    const fileInfo = path.parse(audioFile)
    const audioFileName = fileInfo.name.replace('_audio', '')
    const videoFiles = getMatchVideoFiles(audioFileName)

    if (!videoFiles.length) {
      console.log(`${tipsHead} 未找到【${audioFile}】对应的视频文件`)
      return true
    }

    const startTime = Date.now()
    const videoFile = videoFiles[0]
    const videoFileInfo = path.parse(videoFile)
    const resultVideoFileName = `${videoFileInfo.name.replace(/_video/gi, '')}${videoFileInfo.ext}`
    const resultVideoFilePath = path.join(cwd, `./audio-video-merger/${resultVideoFileName}`)

    if (fs.existsSync(resultVideoFilePath)) {
      console.log(`${tipsHead} 【${audioFileName}】的合并文件已存在`)
      return true
    }

    /* 移除合并出错或没合并完成的旧文件 */
    if (fs.existsSync(path.join(videoFileInfo.dir, resultVideoFileName))) {
      fs.removeSync(path.join(videoFileInfo.dir, resultVideoFileName))
    }

    try {
      console.log(`${tipsHead} 正在合并：${audioFileName}`)
      
      audioVideoMerger(audioFile, videoFile)

      fs.moveSync(path.join(videoFileInfo.dir, resultVideoFileName), resultVideoFilePath)

      const duration = ((Date.now() - startTime) / 1000).toFixed(2)
      console.log(`${tipsHead} 合并成功，耗时：${duration}s`)
    } catch (e) {
      console.error(`${tipsHead} ${audioFile}合并失败：\n`, e)
    }
  })
}

main(process.cwd())

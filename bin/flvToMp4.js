const execSync = require('child_process').execSync
const path = require('path')
const fs = require('fs-extra')
const fg = require('fast-glob')
const tipsHead = '[flv-to-mp4]'

/**
 * 使用ffmpeg将flv文件转换成mp4文件，注意该操作为同步操作，会阻塞脚本的运行
 * 参考：https://juejin.cn/post/6844903757503807495
 * ffmpeg -i input.flv -vcodec copy -acodec copy output.mp4
 * @param {*} filePath
 */
function flvToMp4 (filePath) {
  const fileInfo = path.parse(filePath)
  const command = `ffmpeg -i "${filePath}" -vcodec copy -acodec copy "${filePath.replace('.flv', '.mp4')}"`

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

  const flvFiles = await fg(['*.flv'], globOpts)
  const mp4Files = await fg(['flv-to-mp4/*.mp4'], globOpts)
  const mp4FilesName = mp4Files.map((mp4File) => path.parse(mp4File).name)

  if(!flvFiles.length){
    console.log(`${tipsHead} ${cwd}`)
    console.log(`${tipsHead} 当前目录下未发现flv文件`)
    return false
  }

  fs.ensureDirSync(path.join(cwd, './flv-to-mp4'))

  flvFiles.forEach((flvFile) => {
    const fileInfo = path.parse(flvFile)
    const flvFileName = fileInfo.name
    if (mp4FilesName.includes(flvFileName)) {
      console.log(`${tipsHead} ${flvFileName}的mp4版本的文件已存在`)
      return true
    }

    try {
      console.log(`${tipsHead} 正在转换：${flvFile}`)
      const startTime = Date.now()
      const mp4FilePath = flvFile.replace('.flv', '.mp4')

      /* 移除转换出错或没转换完成的旧文件 */
      if (fs.existsSync(mp4FilePath)) {
        fs.removeSync(mp4FilePath)
      }

      flvToMp4(flvFile)
      fs.moveSync(mp4FilePath, path.join(cwd, `./flv-to-mp4/${fileInfo.name}.mp4`))

      const duration = ((Date.now() - startTime) / 1000).toFixed(2)
      console.log(`${tipsHead} 转换成功，耗时：${duration}s`)
    } catch (e) {
      console.error(`${tipsHead} ${flvFileName}转换失败：\n`, e)
    }
  })
}

main(process.cwd())

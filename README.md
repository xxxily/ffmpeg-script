# ffmpeg-script

> 基于ffmpeg的一键脚本命令，实现特定需求的一键操作

## 使用说明

使用前先确保你电脑已经安装了：[ffmpeg](http://ffmpeg.org/)  

基本工作原理为执行ffmpeg的相关命令，例如：

```sh
# 将音视频文件进行快速的合并操作
ffmpeg -i out.mp4 -i out.aac -vcodec copy -acodec copy new.mp4

# 将flv文件快速转换成mp4文件
ffmpeg -i input.flv -vcodec copy -acodec copy output.mp4
```

## 特性

- 支持批量处理
- 不重新编码，快速处理
- 自动检测是否存在相关文件
- 自动跳过已处理完成的文件
- 可随时终止，无需担心出错
- 一个场景一条命令，简单快捷

### 安装脚本

```sh
# npm
npm install ffmpeg-script --global

# yarn

yarn global add ffmpeg-script
```

中国大陆用户可使用阿里源进行加速安装

```sh
# npm
npm install ffmpeg-script --global --registry=https://registry.npmmirror.com

# yarn

yarn global add ffmpeg-script --registry=https://registry.npmmirror.com
```

### 使用脚本

去到存在相关文件的目录，直接运行对应命令即可

```sh
# 批量音视频文件合并
audioVideoMerger
# 或者
avm

# 批量将flv文件转换成mp4文件
flv2mp4
```

### 批量音视频文件合并

脚本支持对存在音视频文件的目录下的文件进行批量的自动合并，主要用于合并通过[h5player for tampermonkey](https://github.com/xxxily/h5player) 脚本下载回来的音视频文件  

也支持其它符合命名规则的音视频文件，当目录存在如下规则的文件，即可使用命令进行批量合并：

- 基础文件名一致
- 音频文件以`_audio.xxx`(后缀格式不限)
- 视频文件以`_video.xxx`(后缀格式不限)

如：

```txt
demo1_audio.mp4
demo1_video.mp4

demo2_audio.webm
demo2_video.webm

demo3_audio.mp3
demo3_video.mp4
```

合并命令：

```sh
# 批量音视频文件合并
audioVideoMerger

# 为了方便，也支持命令的简写
avm
```

### 批量将flv文件转换成mp4文件

脚本支持对下载或录制回来的flv文件批量转换成mp4文件，

转换命令：

```sh
flv2mp4

# 或者
flvtomp4
```

## 本地调试

将当前项目安装到全局

```sh
npm i -g .
```

将当前项目从全局中移除

```sh
npm uninstall -g .
```

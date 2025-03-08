#!/bin/bash

# 创建图标目录
mkdir -p src-tauri/icons

# 创建一个简单的基础图标 (512x512 像素)
convert -size 512x512 xc:none -fill "#3178c6" -draw "roundrectangle 50,50,462,462,80,80" \
  -fill white -font Arial -pointsize 240 -gravity center -annotate 0 "FS" \
  src-tauri/icons/app-icon.png

# 生成不同尺寸的图标
convert src-tauri/icons/app-icon.png -resize 32x32 src-tauri/icons/32x32.png
convert src-tauri/icons/app-icon.png -resize 128x128 src-tauri/icons/128x128.png
convert src-tauri/icons/app-icon.png -resize 256x256 src-tauri/icons/128x128@2x.png

# 生成 Windows 图标
convert src-tauri/icons/app-icon.png -define icon:auto-resize=16,32,48,64,128,256 src-tauri/icons/icon.ico

# 生成 macOS 图标
mkdir -p src-tauri/icons/icon.iconset
convert src-tauri/icons/app-icon.png -resize 16x16 src-tauri/icons/icon.iconset/icon_16x16.png
convert src-tauri/icons/app-icon.png -resize 32x32 src-tauri/icons/icon.iconset/icon_16x16@2x.png
convert src-tauri/icons/app-icon.png -resize 32x32 src-tauri/icons/icon.iconset/icon_32x32.png
convert src-tauri/icons/app-icon.png -resize 64x64 src-tauri/icons/icon.iconset/icon_32x32@2x.png
convert src-tauri/icons/app-icon.png -resize 128x128 src-tauri/icons/icon.iconset/icon_128x128.png
convert src-tauri/icons/app-icon.png -resize 256x256 src-tauri/icons/icon.iconset/icon_128x128@2x.png
convert src-tauri/icons/app-icon.png -resize 256x256 src-tauri/icons/icon.iconset/icon_256x256.png
convert src-tauri/icons/app-icon.png -resize 512x512 src-tauri/icons/icon.iconset/icon_256x256@2x.png
convert src-tauri/icons/app-icon.png -resize 512x512 src-tauri/icons/icon.iconset/icon_512x512.png
convert src-tauri/icons/app-icon.png -resize 1024x1024 src-tauri/icons/icon.iconset/icon_512x512@2x.png

# 使用 iconutil 将 iconset 转换为 icns (仅在 macOS 上有效)
iconutil -c icns src-tauri/icons/icon.iconset -o src-tauri/icons/icon.icns

echo "图标文件已创建完成！"
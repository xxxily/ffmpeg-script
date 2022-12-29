#!/usr/bin/env node
const pkg = require("../package.json");
const { Command } = require("commander");
const inquirer = require("inquirer");
const execSync = require("child_process").execSync;
const path = require("path");
const fs = require("fs-extra");
const fg = require("fast-glob");
const tipsHead = "[flv-to-mp4]";

/**
 * commander的使用说明
 * https://github.com/tj/commander.js/blob/master/Readme_zh-CN.md
 */
const program = new Command();
program.version(pkg.version);
program
  .option("-d, --debug", "是否输出相关调试信息", false)
  .option("-q, --inquirer", "是否使用inquirer来引导输入相关参数", false)
  .option("-w, --watch", "是否持续检查有需要转换的文件", false)
  .option(
    "-t, --timeout <type>",
    "指定重复执行转换检查的时间间隔，默认30秒",
    30
  )
  .option("-a, --archive", "是否自动归档（暂支持按日期归档）", false)
  .option("-r, --remove", "转换完成后，是否删除flv源文件", false)
  .option("--no-skip", "是否自动跳过不符合转换条件的文件", true)
  .option("-c, --cwd <type>", "指定命令行的工作目录", process.cwd())
  .option("-o, --output <type>", "指定转换成功后的输出目录");

program.parse(process.argv);
const cmdOpts = program.opts();
cmdOpts.debug && console.log("Command选项：", cmdOpts);

/**
 * 使用ffmpeg将flv文件转换成mp4文件，注意该操作为同步操作，会阻塞脚本的运行
 * 参考：https://juejin.cn/post/6844903757503807495
 * ffmpeg -i input.flv -vcodec copy -acodec copy output.mp4
 * @param {*} filePath
 */
function flvToMp4(filePath) {
  const fileInfo = path.parse(filePath);
  const command = `ffmpeg -y -i "${filePath}" -vcodec copy -acodec copy "${filePath.replace(
    ".flv",
    ".mp4"
  )}"`;

  return execSync(command, {
    cwd: fileInfo.dir,
  });
}

const convertSucFileList = [];
const convertFailFileList = [];

async function convertHandler(inputDir, outputDir, opts = {}) {
  const flvFiles = await fg(["*.flv"], { cwd: inputDir, absolute: true });
  const mp4Files = await fg(["*.mp4", "**/*.mp4"], {
    cwd: outputDir,
    absolute: true,
  });
  const mp4FilesName = mp4Files.map((mp4File) => path.parse(mp4File).name);

  if (!flvFiles.length) {
    console.log(`${tipsHead} ${inputDir}`);
    console.log(`${tipsHead} 当前目录下未发现flv文件`);
    return false;
  }

  flvFiles.forEach((flvFile) => {
    const fileInfo = path.parse(flvFile);
    const flvFileName = fileInfo.name;

    if (convertSucFileList.includes(flvFileName) && opts.watch) {
      opts.debug && console.log(`${tipsHead} ${flvFileName} 文件已转换过`);
      return false;
    }

    if (mp4FilesName.includes(flvFileName)) {
      console.log(`${tipsHead} ${flvFileName}的mp4版本的文件已存在`);
      return true;
    }

    if (convertFailFileList.includes(flvFileName)) {
      console.log(`${tipsHead} ${flvFileName} 转换异常文件，已跳过`);
      return false;
    }

    const flvInfo = fs.statSync(flvFile);

    if (opts.watch && Date.now() - flvInfo.mtime.getTime() < 1000 * 60) {
      opts.debug &&
        console.log(
          `${tipsHead} ${flvFileName} 文件内容最近仍在修改，可能还未录制结束，暂时跳过`
        );
      return false;
    }

    try {
      console.log(`${tipsHead} 正在转换：${flvFile}`);
      const startTime = Date.now();
      const mp4FilePath = flvFile.replace(".flv", ".mp4");

      /* 移除转换出错或没转换完成的旧文件 */
      if (fs.existsSync(mp4FilePath)) {
        fs.removeSync(mp4FilePath);
      }

      flvToMp4(flvFile);

      let destPath = path.join(outputDir, `${fileInfo.name}.mp4`);

      /* 创建归档目录 */
      if (opts.archive) {
        const fileDate = `${flvInfo.mtime.getFullYear()}-${
          flvInfo.mtime.getMonth() + 1
        }.${flvInfo.mtime.getDate()}`;
        const destDir = path.join(outputDir, fileDate);

        fs.ensureDirSync(destDir);
        destPath = path.join(destDir, `${fileInfo.name}.mp4`);
      }

      fs.moveSync(mp4FilePath, destPath);

      const duration = ((Date.now() - startTime) / 1000).toFixed(2);
      console.log(`${tipsHead} 转换成功，耗时：${duration}s`);

      /* 如果包含remove选项，则尝试对转换成功的文件进行移除 */
      if (opts.remove) {
        setTimeout(() => {
          fs.remove(flvFile);
        }, 500);
      }

      convertSucFileList.push(flvFileName);
    } catch (e) {
      convertFailFileList.push(flvFileName);
      console.error(`${tipsHead} ${flvFileName}转换失败：\n`, e);
    }
  });
}

function inquirerHandler() {
  const inputDir = path.resolve(cmdOpts.cwd || process.cwd());
  const outputDir = path.resolve(
    cmdOpts.output || path.join(path.resolve(inputDir), "./flv-to-mp4")
  );

  function optsHandler(optName, defaultVal = false) {
    const opts = cmdOpts || {};
    return typeof opts[optName] !== "undefined" ? opts[optName] : defaultVal;
  }

  const optsChoices = [
    {
      name: "(--watch) 是否持续检查有需要转换的文件",
      checked: optsHandler("watch"),
      keyName: "watch",
    },
    {
      name: "(--archive) 是否自动归档（暂支持按日期归档）",
      checked: optsHandler("archive"),
      keyName: "archive",
    },
    {
      name: "(--remove) 转换完成后，是否删除flv源文件",
      checked: optsHandler("remove"),
      keyName: "remove",
    },
    {
      name: "(--skip) 是否自动跳过不符合转换条件的文件",
      checked: optsHandler("skip", true),
      keyName: "skip",
    },
    {
      name: "(--debug) 是否输出相关调试信息",
      checked: optsHandler("debug"),
      keyName: "debug",
    },
  ];

  const questions = [
    {
      type: "input",
      name: "cwd",
      message: "指定命令行的工作目录（可以是相对目录）",
      default() {
        return inputDir;
      },
    },
    {
      type: "input",
      name: "output",
      message: "指定转换成功后的输出目录（可以是相对目录）",
      default() {
        return outputDir;
      },
    },
    {
      type: "checkbox",
      message: "相关选项",
      name: "opts",
      choices: optsChoices,
    },
  ];

  function optsAnswersHandler(optsAnswers) {
    const result = {};

    optsChoices.forEach((opt) => {
      result[opt["keyName"]] = optsAnswers.includes(opt.name);
    });

    return result;
  }

  function answersHandler(answers) {
    const inquirerOpt = { ...answers, ...optsAnswersHandler(answers.opts) };
    delete inquirerOpt.opts;

    /* 将inquirerOpt获得选项同步给cmdOpts */
    Object.keys(inquirerOpt).forEach((keyName) => {
      cmdOpts[keyName] = inquirerOpt[keyName];
    });

    cmdOpts.debug && console.log("[inquirer][answers]", answers, cmdOpts);
  }

  function watchTimeoutInquirer() {
    return new Promise((resolve, reject) => {
      inquirer
        .prompt([
          {
            type: "input",
            name: "timeout",
            message: "指定重复执行转换检查的时间间隔，默认30秒",
            default() {
              return cmdOpts.timeout || 30;
            },
          },
        ])
        .then((answers) => {
          answers.timeout && (cmdOpts.timeout = answers.timeout);
          resolve(true);
        })
        .catch((err) => {
          reject(err);
        });
    });
  }

  /**
   * https://github.com/SBoudrias/Inquirer.js
   * https://github.com/SBoudrias/Inquirer.js/tree/master/packages/inquirer/examples
   */
  inquirer
    .prompt(questions)
    .then(async (answers) => {
      answersHandler(answers);
      cmdOpts.watch && (await watchTimeoutInquirer());
      main();
    })
    .catch((error) => {
      if (error.isTtyError) {
        console.error(`Prompt couldn't be rendered in the current environment`);
      }

      throw error;
    });
}

let watchCount = 0;
async function watch(inputDir, outputDir, cmdOpts) {
  try {
    await convertHandler(inputDir, outputDir, cmdOpts);
  } catch (error) {
    console.error("[convertHandler][error]", error);
  }

  watchCount++;
  console.log(`${tipsHead}[Watching] 已执行 ${watchCount} 次`);

  /* 进行循环监听 */
  const timeout = Math.abs(Number(cmdOpts.timeout) || 30) * 1000;
  setTimeout(() => {
    watch(inputDir, outputDir, cmdOpts);
  }, timeout);
}

async function main() {
  const inputDir = path.resolve(cmdOpts.cwd || process.cwd());
  const outputDir = path.resolve(
    cmdOpts.output || path.join(path.resolve(inputDir), "./flv-to-mp4")
  );

  cmdOpts.debug && console.log("输入输出目录：", inputDir, outputDir);

  if (!fs.existsSync(inputDir)) {
    const errMsg = `${tipsHead} 工作目录不存在：${inputDir}`;

    if (cmdOpts.debug) {
      throw new Error(errMsg);
    } else {
      console.error(errMsg);
      return false;
    }
  }

  if (!fs.existsSync(outputDir)) {
    fs.ensureDirSync(outputDir);
    console.log(`${tipsHead} 转换结果存放目录创建成功：${outputDir}`);
  }

  if (cmdOpts.watch) {
    watch(inputDir, outputDir, cmdOpts);
  } else {
    await convertHandler(inputDir, outputDir, cmdOpts);
  }
}

if (cmdOpts.inquirer) {
  inquirerHandler();
} else {
  main();
}

# OpenClaw 离线打包说明

本目录用于存放 OpenClaw 便携版运行时。

## 打包步骤（在有网络环境执行）：

1. 下载 Node.js 22.x Windows x64 二进制：
   https://nodejs.org/dist/v22.x.x/node-v22.x.x-win-x64.zip
   解压到本目录下的 node/ 文件夹

2. 安装 OpenClaw：
   在联网环境运行: npm install -g openclaw@latest
   然后运行: npm pack openclaw
   将生成的 tarball 解压到本目录下的 openclaw/ 文件夹

3. 在 openclaw 目录运行:
   npm install --production

4. 首次运行后会在 data/ 目录生成配置文件

## 目录结构：
resources/
└── openclaw/
    ├── node/           # Node.js 运行时
    ├── openclaw/       # OpenClaw 代码 + node_modules
    ├── data/           # OpenClaw 工作目录
    └── README.txt      # 本文件
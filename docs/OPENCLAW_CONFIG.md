# OpenClaw 配置说明

**项目名称**: Shine Helper - OpenClaw 配置
**日期**: 2026-03-13
**状态**: 已更新

---

## 一、环境变量配置

启动脚本 `start.sh` 中设置了以下 OpenClaw 环境变量：

| 环境变量 | 值 | 说明 |
|----------|-----|------|
| `OPENCLAW_HOME` | `$SCRIPT_DIR/resources/openclaw/openclaw` | OpenClaw 应用程序主目录 |
| `OPENCLAW_STATE_DIR` | `$HOME/Desktop/workspace` | OpenClaw 状态目录（工作区） |
| `OPENCLAW_CONFIG_PATH` | `$SCRIPT_DIR/resources/openclaw/data/openclaw.json` | OpenClaw 配置文件路径 |

---

## 二、目录结构

```
shine-helper/
├── start.sh                          # 启动脚本
├── resources/
│   └── openclaw/
│       ├── node/                     # Node.js 运行时
│       │   └── bin/
│       │       └── node
│       ├── openclaw/                 # OpenClaw 应用程序 (OPENCLAW_HOME)
│       │   ├── package.json
│       │   └── openclaw.mjs
│       └── data/
│           └── openclaw.json         # 配置文件 (OPENCLAW_CONFIG_PATH)
└── ...
```

---

## 三、工作目录

OpenClaw 的工作目录设置在用户桌面的 `workspace` 文件夹：

- **路径**: `$HOME/Desktop/workspace`
- **用途**: 存储 OpenClaw 的项目文件、临时文件、会话数据等
- **自动创建**: 启动脚本会在首次运行时自动创建该目录

---

## 四、启动流程

1. 检查运行环境（Node.js、配置文件、可执行文件）
2. 创建桌面 workspace 目录（如不存在）
3. 设置环境变量：
   - `OPENCLAW_HOME`
   - `OPENCLAW_STATE_DIR`
   - `OPENCLAW_CONFIG_PATH`
4. 检查端口 18789 是否被占用
5. 启动 OpenClaw Gateway 服务
6. 启动 Shine Helper 桌面应用

---

## 五、配置文件说明

配置文件 `resources/openclaw/data/openclaw.json` 包含：

| 配置项 | 说明 |
|--------|------|
| `models.providers` | AI 模型提供商配置（阿里云 GLM-4-7） |
| `agents.defaults.model.primary` | 默认模型 |
| `agents.defaults.workspace` | 默认工作区路径 |
| `gateway.port` | Gateway 服务端口（18789） |
| `gateway.auth` | 认证配置 |

---

## 六、修改记录

| 日期 | 修改内容 |
|------|----------|
| 2026-03-13 | 更新启动脚本，使用环境变量指定配置和工作目录 |
| 2026-03-13 | 更新配置文件中的 workspace 路径为桌面目录 |

---

*文档创建：2026-03-13*

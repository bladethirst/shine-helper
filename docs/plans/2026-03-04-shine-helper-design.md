# Shine Helper 设计文档

**项目名称**: Shine Helper
**版本**: v1.2.0
**日期**: 2026-03-13
**状态**: ✅ 核心功能已完成

---

## 1. 项目概述

### 1.1 产品定位

面向电力行业非技术用户的桌面 AI 助手，帮助领导和员工通过 OpenClaw 解决日常问题（整理报表、系统数据获取、邮件发送等）。

### 1.2 目标用户

- 电力行业非技术背景员工
- 电力行业管理层领导
- 需要简化 AI 使用流程的企业用户

### 1.3 核心功能

1. **对话功能** - 与 OpenClaw 自然对话，获取 AI 辅助
2. **Skills 市场** - 浏览、安装、管理企业 Skills
3. **配置管理** - 可视化配置 OpenClaw 连接和偏好

---

## 2. 技术架构

### 2.1 技术栈

| 层级 | 技术选择 |
|------|----------|
| 桌面框架 | Tauri 2.x (Rust + Web) |
| 前端框架 | Vue 3 + TypeScript |
| 状态管理 | Pinia |
| 样式方案 | Tailwind CSS |
| 本地存储 | SQLite (via rusqlite) |
| HTTP 客户端 | reqwest (Rust) |

### 2.2 架构图

```
┌─────────────────────────────────────────────────┐
│                  Vue 3 前端                      │
│  ┌─────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ 对话模块 │  │Skills市场 │  │   配置面板    │  │
│  └────┬────┘  └─────┬────┘  └───────┬───────┘  │
└───────┼────────────┼───────────────┼───────────┘
        │            │               │
        ▼            ▼               ▼
┌───────────────────────────────────────────────────┐
│              Tauri Commands (IPC 桥接)             │
│  ┌─────────────┐  ┌────────────┐  ┌────────────┐ │
│  │ chat_proxy  │  │skills_mgr  │  │ config_mgr │ │
│  └─────────────┘  └────────────┘  └────────────┘ │
└───────────────────────────────────────────────────┘
        │                    ▲
        ▼                    │
┌─────────────────────┐      │
│   OpenClaw REST     │      │ 可选远程
│   (本地/远程)       │──────┘
└─────────────────────┘

┌─────────────────────┐
│  内网 Skills 市场    │
│  (REST API)         │
└─────────────────────┘
```

### 2.3 模块职责

| 模块 | 职责 |
|------|------|
| chat_proxy | OpenClaw API 代理、流式响应处理、对话历史管理 |
| skills_mgr | Skills 市场交互、下载、安装、版本管理 |
| config_mgr | 配置读取/写入、凭据管理、应用偏好 |

---

## 3. 功能详细设计

### 3.1 对话功能

#### 3.1.1 数据流

```
用户输入 ──▶ Vue 前端 ──▶ Tauri IPC ──▶ Rust chat_proxy
                                              │
                                              ▼
                                        OpenClaw API
                                        (POST /chat)
                                              │
                                              ▼
                                        Rust 接收 SSE 流
                                              │
                                              ▼
                                        Vue 前端渲染
                                        (流式输出)
```

#### 3.1.2 核心接口

```typescript
// Tauri Commands
create_session() -> SessionId
send_message(sessionId: string, message: string) -> Stream<Response>
get_history(sessionId: string) -> Message[]
delete_session(sessionId: string)
list_sessions() -> Session[]
```

#### 3.1.3 数据模型

```typescript
interface Session {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
}

interface Message {
  id: string;
  sessionId: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}
```

### 3.2 Skills 市场

#### 3.2.1 数据流

```
用户浏览 ──▶ Vue 前端 ──▶ Tauri IPC ──▶ Rust skills_mgr
                                              │
                                              ▼
                                        内网 Skills 市场 API
                                        (GET /skills)
                                              │
                                              ▼
                                        本地 Skills 目录
                                        (~/.shine_helper/skills/)
```

#### 3.2.2 核心接口

```typescript
list_market_skills() -> Skill[]
install_skill(skillId: string) -> Result
uninstall_skill(skillId: string) -> Result
get_local_skills() -> Skill[]
toggle_skill(skillId: string, enabled: boolean)
check_updates() -> SkillUpdate[]
```

#### 3.2.3 数据模型

```typescript
interface Skill {
  id: string;
  name: string;
  description: string;
  version: string;
  author: 'official' | 'community';
  icon?: string;
  installed: boolean;
  enabled: boolean;
  installedVersion?: string;
}
```

### 3.3 配置管理

#### 3.3.1 配置项

| 配置项 | 说明 | 存储方式 |
|--------|------|----------|
| OpenClaw URL | 本地/远程服务地址 | 明文 |
| OpenClaw API Key | 认证凭据 | 系统密钥库 |
| 市场地址 | 内网 Skills 市场 URL | 明文 |
| 市场认证 Token | API Token | 系统密钥库 |
| 主题 | light / dark / system | 配置文件 |
| 语言 | zh-CN / en-US | 配置文件 |

#### 3.3.2 核心接口

```typescript
get_openclaw_config() -> OpenClawConfig
set_openclaw_config(config: OpenClawConfig)
test_connection() -> boolean
get_market_config() -> MarketConfig
set_market_config(config: MarketConfig)
get_app_preferences() -> Preferences
set_app_preferences(prefs: Preferences)
```

---

## 4. 安全策略

| 场景 | 策略 |
|------|------|
| API 认证 | API Key 存储在系统密钥库（Windows: Credential Manager） |
| 内网市场认证 | API Token，支持配置 |
| 数据传输 | HTTPS 强制（远程模式），本地使用 Unix Socket |
| Skill 隔离 | 每个 Skill 运行在独立沙箱环境 |
| 敏感操作 | 发送邮件/导出数据需要二次确认 |

---

## 5. UI/UX 设计

### 5.1 整体布局

```
┌─────────────────────────────────────────────────────────┐Logo] Shine Helper
│  [          [⚙️] [👤]               │
├────────────┬────────────────────────────────────────────┤
│            │                                            │
│  💬 对话   │          主内容区域                         │
│            │                                            │
│  🛒 Skills │   (根据左侧导航显示对应内容)                 │
│            │                                            │
│  ⚙️ 配置   │                                            │
│            │                                            │
└────────────┴────────────────────────────────────────────┘
```

### 5.2 设计原则

- 简洁直观，降低学习成本
- 左侧固定导航，右侧内容区
- 重要操作提供引导提示
- 电力行业风格：专业、稳重、易用

---

## 6. 验收标准

### 6.1 功能验收

- [x] 能够成功连接本地 OpenClaw 服务并对话
- [x] 能够连接远程 OpenClaw 服务（可配置）
- [x] 能够浏览内网 Skills 市场
- [x] 能够安装/卸载 Skills
- [x] 能够修改和保存配置
- [x] 对话历史正确持久化

### 6.2 性能验收

- [x] 冷启动时间 < 3 秒
- [x] 对话响应延迟 < 1 秒（不计 OpenClaw 处理时间）
- [x] 内存占用 < 200MB

### 6.3 安全验收

- [x] API Key 不以明文存储
- [x] 敏感操作有确认提示
- [x] 网络请求支持 HTTPS

---

## 7. 后续工作

1. **实现计划** - 详细的任务拆分和时间安排
2. **技术验证** - 原型验证关键功能
3. **迭代开发** - 按优先级实现功能

---

*本文档由 Sisyphus AI 生成，日期: 2026-03-04*

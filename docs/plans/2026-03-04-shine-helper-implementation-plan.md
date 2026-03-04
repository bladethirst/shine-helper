# Shine Helper 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一个面向电力行业用户的桌面 AI 助手，实现与 OpenClaw 对话、Skills 市场、配置管理功能

**Architecture:** 采用 Tauri 2.x + Vue 3 + TypeScript 架构，前端使用 Pinia 状态管理，后端使用 Rust 处理 OpenClaw API 代理和本地存储

**Tech Stack:** Tauri 2.x, Vue 3, TypeScript, Pinia, Tailwind CSS, SQLite (rusqlite), reqwest

---

## 阶段 1: 项目初始化

### Task 1.1: 初始化 Tauri + Vue 3 项目

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tailwind.config.js`
- Create: `postcss.config.js`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src/style.css`
- Create: `index.html`
- Create: `Cargo.toml`
- Create: `tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/Cargo.toml`

**Step 1: 创建 package.json**

```json
{
  "name": "shine-helper",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.4.0",
    "pinia": "^2.1.7",
    "vue-router": "^4.2.5",
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.0.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0",
    "vue-tsc": "^1.8.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}
```

**Step 2: 创建 Vite 配置**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  }
})
```

**Step 3: 创建 Tauri 配置 (tauri.conf.json)**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Shine Helper",
  "version": "1.0.0",
  "identifier": "com.shine.helper",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Shine Helper",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

**Step 4: 创建 Rust 入口 (src-tauri/src/main.rs)**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 5: 创建 Cargo.toml (src-tauri)**

```toml
[package]
name = "shine-helper"
version = "1.0.0"
description = "Shine Helper - Desktop AI Assistant"
authors = ["Shine Team"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
keyring = "3"
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
log = "0.4"
env_logger = "0.11"
thiserror = "2"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

**Step 6: 安装依赖并验证**

Run: `npm install`
Expected: 成功安装所有依赖

**Step 7: 提交**

```bash
git add .
git commit -m "chore: initialize Tauri + Vue 3 project"
```

---

### Task 1.2: 设置 Tailwind CSS

**Files:**
- Modify: `tailwind.config.js`
- Modify: `postcss.config.js`
- Create: `src/style.css`

**Step 1: 创建 tailwind.config.js**

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
        }
      }
    },
  },
  plugins: [],
}
```

**Step 2: 创建 postcss.config.js**

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

**Step 3: 创建 src/style.css**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;
  color: #213547;
  background-color: #ffffff;
}

body {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
}
```

**Step 4: 修改 src/main.ts 引入样式**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'

const app = createApp(App)
app.use(createPinia())
app.mount('#app')
```

**Step 5: 修改 index.html 添加 Tailwind 指引**

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Shine Helper</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

**Step 6: 验证构建**

Run: `npm run build`
Expected: 成功构建

**Step 7: 提交**

```bash
git add .
git commit -m "chore: add Tailwind CSS configuration"
```

---

### Task 1.3: 创建基础目录结构和路由

**Files:**
- Create: `src/router/index.ts`
- Create: `src/views/ChatView.vue`
- Create: `src/views/SkillsView.vue`
- Create: `src/views/ConfigView.vue`
- Create: `src/components/Sidebar.vue`
- Modify: `src/App.vue`

**Step 1: 创建路由配置 src/router/index.ts**

```typescript
import { createRouter, createWebHistory } from 'vue-router'
import ChatView from '@/views/ChatView.vue'
import SkillsView from '@/views/SkillsView.vue'
import ConfigView from '@/views/ConfigView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'chat',
      component: ChatView
    },
    {
      path: '/skills',
      name: 'skills',
      component: SkillsView
    },
    {
      path: '/config',
      name: 'config',
      component: ConfigView
    }
  ]
})

export default router
```

**Step 2: 创建基础视图组件 (空壳)**

```vue
<!-- src/views/ChatView.vue -->
<template>
  <div class="h-full p-6">
    <h1 class="text-2xl font-bold">对话</h1>
  </div>
</template>
```

```vue
<!-- src/views/SkillsView.vue -->
<template>
  <div class="h-full p-6">
    <h1 class="text-2xl font-bold">Skills 市场</h1>
  </div>
</template>
```

```vue
<!-- src/views/ConfigView.vue -->
<template>
  <div class="h-full p-6">
    <h1 class="text-2xl font-bold">配置</h1>
  </div>
</template>
```

**Step 3: 创建侧边栏组件**

```vue
<!-- src/components/Sidebar.vue -->
<template>
  <aside class="w-64 bg-gray-50 border-r border-gray-200 flex flex-col">
    <div class="p-4 border-b border-gray-200">
      <h1 class="text-xl font-bold text-primary-600">Shine Helper</h1>
    </div>
    <nav class="flex-1 p-4">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors"
        :class="$route.path === item.path ? 'bg-primary-100 text-primary-700' : 'text-gray-600 hover:bg-gray-100'"
      >
        <span class="text-xl">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>
  </aside>
</template>

<script setup lang="ts">
const navItems = [
  { path: '/', label: '对话', icon: '💬' },
  { path: '/skills', label: 'Skills', icon: '🛒' },
  { path: '/config', label: '配置', icon: '⚙️' }
]
</script>
```

**Step 4: 修改 App.vue**

```vue
<template>
  <div class="h-screen flex">
    <Sidebar />
    <main class="flex-1 overflow-auto bg-white">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { RouterView } from 'vue-router'
</script>
```

**Step 5: 修改 main.ts 添加路由**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './style.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
```

**Step 6: 验证**

Run: `npm run build`
Expected: 成功构建

**Step 7: 提交**

```bash
git add .
git commit -m "feat: add router and basic layout components"
```

---

## 阶段 2: 后端核心模块

### Task 2.1: 配置管理模块 (Rust)

**Files:**
- Create: `src-tauri/src/config.rs`
- Create: `src-tauri/src/lib.rs`

**Step 1: 创建配置管理模块 src-tauri/src/config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Keyring error: {0}")]
    Keyring(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub url: String,
    pub use_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub openclaw: OpenClawConfig,
    pub market: MarketConfig,
    pub preferences: AppPreferences,
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8000".to_string(),
            use_local: true,
        }
    }
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3001".to_string(),
            enabled: true,
        }
    }
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openclaw: OpenClawConfig::default(),
            market: MarketConfig::default(),
            preferences: AppPreferences::default(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shine_helper")
        .join("config.json")
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), ConfigError> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}
```

**Step 2: 创建 lib.rs 导出模块**

```rust
pub mod config;
pub use config::*;
```

**Step 3: 更新 main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

**Step 5: 提交**

```bash
git add .
git commit -m "feat: add config management module in Rust"
```

---

### Task 2.2: 对话历史存储模块 (Rust + SQLite)

**Files:**
- Create: `src-tauri/src/db.rs`
- Create: `src-tauri/src/commands/chat.rs`

**Step 1: 创建数据库模块 src-tauri/src/db.rs**

```rust
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> SqliteResult<Self> {
        let db_path = get_db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )",
            [],
        )?;
        Ok(())
    }

    pub fn create_session(&self, title: &str) -> SqliteResult<Session> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            [&id, title, &now, &now],
        )?;

        Ok(Session {
            id,
            title: title.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_sessions(&self) -> SqliteResult<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, created_at, updated_at FROM sessions ORDER BY updated_at DESC")?;
        let sessions = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;
        Ok(sessions)
    }

    pub fn add_message(&self, session_id: &str, role: &str, content: &str) -> SqliteResult<Message> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            [&id, session_id, role, content, &now],
        )?;
        
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            [&now, session_id],
        )?;

        Ok(Message {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: now,
        })
    }

    pub fn get_messages(&self, session_id: &str) -> SqliteResult<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, timestamp FROM messages WHERE session_id = ?1 ORDER BY timestamp ASC"
        )?;
        let messages = stmt.query_map([session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;
        Ok(messages)
    }

    pub fn delete_session(&self, session_id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM messages WHERE session_id = ?1", [session_id])?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
        Ok(())
    }
}

fn get_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shine_helper")
        .join("data.db")
}
```

**Step 2: 提交**

```bash
git add .
git commit -m "feat: add SQLite database module for chat history"
```

---

### Task 2.3: Tauri Commands 注册

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: 创建命令模块 src-tauri/src/commands/mod.rs**

```rust
mod chat;

pub use chat::*;
```

**Step 2: 创建 chat 命令 src-tauri/src/commands/chat.rs**

```rust
use crate::db::{Database, Message, Session};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<rusqlite::Error> for CommandError {
    fn from(err: rusqlite::Error) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

#[tauri::command]
pub fn create_session(state: State<'_, AppState>, title: String) -> Result<Session, CommandError> {
    let db = state.db.lock().unwrap();
    let session = db.create_session(&title)?;
    Ok(session)
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, CommandError> {
    let db = state.db.lock().unwrap();
    let sessions = db.get_sessions()?;
    Ok(sessions)
}

#[tauri::command]
pub fn get_messages(state: State<'_, AppState>, session_id: String) -> Result<Vec<Message>, CommandError> {
    let db = state.db.lock().unwrap();
    let messages = db.get_messages(&session_id)?;
    Ok(messages)
}

#[tauri::command]
pub fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), CommandError> {
    let db = state.db.lock().unwrap();
    db.delete_session(&session_id)?;
    Ok(())
}

#[tauri::command]
pub fn add_message(
    state: State<'_, AppState>,
    session_id: String,
    role: String,
    content: String,
) -> Result<Message, CommandError> {
    let db = state.db.lock().unwrap();
    let message = db.add_message(&session_id, &role, &content)?;
    Ok(message)
}
```

**Step 3: 更新 main.rs 注册命令**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod commands;

use commands::{AppState, create_session, list_sessions, get_messages, delete_session, add_message};
use db::Database;
use std::sync::Mutex;

fn main() {
    let db = Database::new().expect("Failed to initialize database");
    
    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            create_session,
            list_sessions,
            get_messages,
            delete_session,
            add_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

**Step 5: 提交**

```bash
git add .
git commit -m "feat: add Tauri commands for chat operations"
```

---

## 阶段 3: 前端对话功能

### Task 3.1: 对话页面 UI

**Files:**
- Modify: `src/views/ChatView.vue`
- Create: `src/components/ChatMessage.vue`
- Create: `src/components/ChatInput.vue`

**Step 1: 创建 ChatMessage 组件**

```vue
<template>
  <div :class="['flex gap-3 mb-4', role === 'user' ? 'flex-row-reverse' : '']">
    <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm"
         :class="role === 'user' ? 'bg-primary-500 text-white' : 'bg-gray-200'">
      {{ role === 'user' ? '👤' : '🤖' }}
    </div>
    <div :class="['max-w-[70%] rounded-lg p-3', role === 'user' ? 'bg-primary-50' : 'bg-gray-100']">
      <p class="whitespace-pre-wrap">{{ content }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  role: 'user' | 'assistant'
  content: string
}>()
</script>
```

**Step 2: 创建 ChatInput 组件**

```vue
<template>
  <div class="border-t border-gray-200 p-4 bg-white">
    <div class="flex gap-2">
      <input
        v-model="message"
        type="text"
        placeholder="请输入消息..."
        class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
        @keyup.enter="send"
      />
      <button
        @click="send"
        :disabled="!message.trim()"
        class="px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        发送
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const message = ref('')
const emit = defineEmits<{
  send: [content: string]
}>()

const send = () => {
  if (message.value.trim()) {
    emit('send', message.value)
    message.value = ''
  }
}
</script>
```

**Step 3: 更新 ChatView.vue**

```vue
<template>
  <div class="h-full flex flex-col">
    <!-- 会话列表 -->
    <div class="border-b border-gray-200 p-4 flex items-center justify-between">
      <h2 class="text-lg font-semibold">会话列表</h2>
      <button
        @click="createNewSession"
        class="px-3 py-1 text-sm bg-primary-500 text-white rounded hover:bg-primary-600"
      >
        新建会话
      </button>
    </div>

    <!-- 消息列表 -->
    <div class="flex-1 overflow-auto p-4">
      <div v-if="messages.length === 0" class="text-center text-gray-400 mt-20">
        <p class="text-4xl mb-4">💬</p>
        <p>开始一段新对话吧</p>
      </div>
      <ChatMessage
        v-for="msg in messages"
        :key="msg.id"
        :role="msg.role as 'user' | 'assistant'"
        :content="msg.content"
      />
      <div v-if="isLoading" class="flex gap-3 mb-4">
        <div class="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center text-sm">🤖</div>
        <div class="bg-gray-100 rounded-lg p-3">
          <span class="animate-pulse">正在输入...</span>
        </div>
      </div>
    </div>

    <!-- 输入框 -->
    <ChatInput @send="handleSend" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ChatMessage from '@/components/ChatMessage.vue'
import ChatInput from '@/components/ChatInput.vue'

interface Message {
  id: string
  session_id: string
  role: string
  content: string
  timestamp: string
}

interface Session {
  id: string
  title: string
  created_at: string
  updated_at: string
}

const sessions = ref<Session[]>([])
const currentSession = ref<Session | null>(null)
const messages = ref<Message[]>([])
const isLoading = ref(false)

onMounted(async () => {
  sessions.value = await invoke<Session[]>('list_sessions')
  if (sessions.value.length > 0) {
    currentSession.value = sessions.value[0]
    messages.value = await invoke<Message[]>('get_messages', { sessionId: currentSession.value.id })
  }
})

const createNewSession = async () => {
  const session = await invoke<Session>('create_session', { title: '新会话' })
  sessions.value.unshift(session)
  currentSession.value = session
  messages.value = []
}

const handleSend = async (content: string) => {
  if (!currentSession.value) {
    await createNewSession()
  }
  
  // 添加用户消息
  const userMsg = await invoke<Message>('add_message', {
    sessionId: currentSession.value!.id,
    role: 'user',
    content
  })
  messages.value.push(userMsg)
  
  isLoading.value = true
  
  try {
    // TODO: 调用 OpenClaw API
    const response = await invoke<string>('send_message', {
      sessionId: currentSession.value!.id,
      message: content
    })
    
    const assistantMsg = await invoke<Message>('add_message', {
      sessionId: currentSession.value!.id,
      role: 'assistant',
      content: response
    })
    messages.value.push(assistantMsg)
  } catch (e) {
    console.error('Failed to send message:', e)
  } finally {
    isLoading.value = false
  }
}
</script>
```

**Step 4: 验证编译**

Run: `npm run build`
Expected: 成功构建

**Step 5: 提交**

```bash
git add .
git commit -m "feat: add chat UI components"
```

---

### Task 3.2: OpenClaw API 集成

**Files:**
- Create: `src-tauri/src/openclaw.rs`
- Modify: `src-tauri/src/commands/chat.rs`

**Step 1: 创建 OpenClaw 客户端模块**

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<Vec<MessageItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageItem {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
}

pub struct OpenClawClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl OpenClawClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
            
        Self {
            client,
            base_url: base_url.to_string(),
            api_key: None,
        }
    }

    pub fn set_api_key(&mut self, key: String) {
        self.api_key = Some(key);
    }

    pub async fn chat(&self, message: &str) -> Result<String, String> {
        let url = format!("{}/chat", self.base_url);
        
        let mut request = self.client.post(&url).json(&ChatRequest {
            message: message.to_string(),
            history: None,
        });
        
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request.send().await
            .map_err(|e| e.to_string())?;
            
        let chat_resp: ChatResponse = response.json().await
            .map_err(|e| e.to_string())?;
            
        Ok(chat_resp.response)
    }
}
```

**Step 2: 更新 chat 命令集成 OpenClaw**

```rust
// 在 chat.rs 中添加
use crate::openclaw::OpenClawClient;

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    session_id: String,
    message: String,
) -> Result<String, CommandError> {
    // 获取消息历史
    let messages = {
        let db = state.db.lock().unwrap();
        db.get_messages(&session_id)?
    };
    
    // 构建 OpenClaw 请求
    let config = crate::config::load_config().unwrap_or_default();
    let mut client = OpenClawClient::new(&config.openclaw.url);
    
    // 调用 OpenClaw API
    let response = client.chat(&message).await
        .map_err(|e| CommandError { message: e })?;
    
    // 保存助手回复
    {
        let db = state.db.lock().unwrap();
        db.add_message(&session_id, "assistant", &response)?;
    }
    
    Ok(response)
}
```

**Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add .
git commit -m "feat: integrate OpenClaw API client"
```

---

## 阶段 4: Skills 市场功能

### Task 4.1: Skills 管理模块 (Rust)

**Files:**
- Create: `src-tauri/src/skills.rs`
- Modify: `src-tauri/src/commands/mod.rs`

**Step 1: 创建 Skills 管理模块**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub icon: Option<String>,
    pub installed: bool,
    pub enabled: bool,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub icon: Option<String>,
    pub download_url: String,
}

pub struct SkillsManager {
    skills_dir: PathBuf,
}

impl SkillsManager {
    pub fn new() -> Self {
        let skills_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shine_helper")
            .join("skills");
            
        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir).ok();
        }
        
        Self { skills_dir }
    }

    pub fn get_local_skills(&self) -> Vec<Skill> {
        let mut skills = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(meta) = fs::metadata(path.join("skill.json")) {
                        if meta.is_file() {
                            if let Ok(content) = fs::read_to_string(path.join("skill.json")) {
                                if let Ok(skill) = serde_json::from_str::<Skill>(&content) {
                                    skills.push(skill);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        skills
    }

    pub fn install_skill(&self, market_skill: &MarketSkill) -> Result<(), String> {
        let skill_dir = self.skills_dir.join(&market_skill.id);
        
        if skill_dir.exists() {
            return Err("Skill already installed".to_string());
        }
        
        fs::create_dir_all(&skill_dir).map_err(|e| e.to_string())?;
        
        // 下载 Skill 包（简化版本：直接复制）
        // 实际实现需要 HTTP 下载
        let skill = Skill {
            id: market_skill.id.clone(),
            name: market_skill.name.clone(),
            description: market_skill.description.clone(),
            version: market_skill.version.clone(),
            author: market_skill.author.clone(),
            icon: market_skill.icon.clone(),
            installed: true,
            enabled: true,
            installed_version: Some(market_skill.version.clone()),
        };
        
        let content = serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())?;
        fs::write(skill_dir.join("skill.json"), content).map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub fn uninstall_skill(&self, skill_id: &str) -> Result<(), String> {
        let skill_dir = self.skills_dir.join(skill_id);
        
        if !skill_dir.exists() {
            return Err("Skill not installed".to_string());
        }
        
        fs::remove_dir_all(&skill_dir).map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
```

**Step 2: 注册 Skills 命令**

```rust
// commands/mod.rs 添加
mod skills;
pub use skills::*;
```

**Step 3: 提交**

```bash
git add .
git commit -m "feat: add skills management module"
 Task 4.```

---

###2: Skills 市场 UI

**Files:**
- Modify: `src/views/SkillsView.vue`

**Step 1: 更新 SkillsView.vue**

```vue
<template>
  <div class="h-full flex flex-col p-6">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">Skills 市场</h1>
      <button
        @click="refreshSkills"
        class="px-4 py-2 text-primary-600 border border-primary-600 rounded hover:bg-primary-50"
      >
        🔄 刷新
      </button>
    </div>

    <!-- 搜索框 -->
    <div class="mb-6">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索 Skills..."
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
      />
    </div>

    <!-- 已安装的 Skills -->
    <div v-if="installedSkills.length > 0" class="mb-8">
      <h2 class="text-lg font-semibold mb-4">已安装</h2>
      <div class="grid grid-cols-3 gap-4">
        <div
          v-for="skill in installedSkills"
          :key="skill.id"
          class="border border-gray-200 rounded-lg p-4"
        >
           items-center gap-<div class="flex2 mb-2">
            <span class="text-2xl">{{ skill.icon || '📦' }}</span>
            <h3 class="font-semibold">{{ skill.name }}</h3>
          </div>
          <p class="text-sm text-gray-600 mb-3">{{ skill.description }}</p>
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-400">v{{ skill.version }}</span>
            <button
              @click="uninstallSkill(skill.id)"
              class="px-3 py-1 text-sm text-red-600 border border-red-600 rounded hover:bg-red-50"
            >
              卸载
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 市场 Skills -->
    <div>
      <h2 class="text-lg font-semibold mb-4">可用 Skills</h2>
      <div v-if="filteredMarketSkills.length === 0" class="text-center text-gray-400 py-10">
        暂无可用的 Skills
      </div>
      <div class="grid grid-cols-3 gap-4">
        <div
          v-for="skill in filteredMarketSkills"
          :key="skill.id"
          class="border border-gray-200 rounded-lg p-4"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="text-2xl">{{ skill.icon || '📦' }}</span>
            <h3 class="font-semibold">{{ skill.name }}</h3>
          </div>
          <p class="text-sm text-gray-600 mb-3">{{ skill.description }}</p>
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-400">{{ skill.author }}</span>
            <button
              @click="installSkill(skill)"
              class="px-3 py-1 text-sm bg-primary-500 text-white rounded hover:bg-primary-600"
            >
              安装
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Skill {
  id: string
  name: string
  description: string
  version: string
  author: string
  icon?: string
  installed: boolean
  enabled: boolean
}

const searchQuery = ref('')
const installedSkills = ref<Skill[]>([])
const marketSkills = ref<Skill[]>([])

// 模拟市场数据（实际应从 API 获取）
onMounted(async () => {
  try {
    installedSkills.value = await invoke<Skill[]>('get_local_skills')
    marketSkills.value = [
      { id: 'report', name: '报表整理', description: '帮助整理各类报表数据', version: '1.0.0', author: '官方', installed: false, enabled: false },
      { id: 'email', name: '邮件发送', description: '自动化发送邮件功能', version: '1.0.0', author: '官方', installed: false, enabled: false },
      { id: 'data-fetch', name: '数据获取', description: '从系统获取数据', version: '1.0.0', author: '官方', installed: false, enabled: false },
    ]
  } catch (e) {
    console.error('Failed to load skills:', e)
  }
})

const filteredMarketSkills = computed(() => {
  const installed = new Set(installedSkills.value.map(s => s.id))
  return marketSkills.value
    .filter(s => !installed.has(s.id))
    .filter(s => s.name.includes(searchQuery.value) || s.description.includes(searchQuery.value))
})

const refreshSkills = async () => {
  try {
    installedSkills.value = await invoke<Skill[]>('get_local_skills')
  } catch (e) {
    console.error('Failed to refresh:', e)
  }
}

const installSkill = async (skill: Skill) => {
  try {
    await invoke('install_skill', { skillId: skill.id })
    await refreshSkills()
  } catch (e) {
    console.error('Failed to install:', e)
  }
}

const uninstallSkill = async (skillId: string) => {
  try {
    await invoke('uninstall_skill', { skillId })
    await refreshSkills()
  } catch (e) {
    console.error('Failed to uninstall:', e)
  }
}
</script>
```

**Step 2: 提交**

```bash
git add .
git commit -m "feat: add skills market UI"
```

---

## 阶段 5: 配置管理功能

### Task 5.1: 配置管理 UI

**Files:**
- Modify: `src/views/ConfigView.vue`

**Step 1: 更新 ConfigView.vue**

```vue
<template>
  <div class="h-full overflow-auto p-6">
    <h1 class="text-2xl font-bold mb-6">配置</h1>

    <!-- OpenClaw 配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">OpenClaw 连接</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">服务地址</label>
          <input
            v-model="config.openclaw.url"
            type="text"
            placeholder="http://localhost:8000"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div class="flex items-center gap-2">
          <input
            v-model="config.openclaw.use_local"
            type="checkbox"
            id="use_local"
            class="w-4 h-4 text-primary-600"
          />
          <label for="use_local" class="text-sm">使用本地服务</label>
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">API Key</label>
          <input
            v-model="apiKey"
            type="password"
            placeholder="请输入 API Key"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <button
          @click="testConnection"
          class="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
        >
          测试连接
        </button>
        <span v-if="connectionStatus" :class="connectionStatus === 'success' ? 'text-green-600' : 'text-red-600'">
          {{ connectionStatus === 'success' ? '✓ 连接成功' : '✗ 连接失败' }}
        </span>
      </div>
    </div>

    <!-- Skills 市场配置 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">Skills 市场</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">市场地址</label>
          <input
            v-model="config.market.url"
            type="text"
            placeholder="http://localhost:3001"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div class="flex items-center gap-2">
          <input
            v-model="config.market.enabled"
            type="checkbox"
            id="market_enabled"
            class="w-4 h-4 text-primary-600"
          />
          <label for="market_enabled" class="text-sm">启用市场</label>
        </div>
      </div>
    </div>

    <!-- 应用偏好 -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">应用偏好</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">主题</label>
          <select
            v-model="config.preferences.theme"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
            <option value="system">跟随系统</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium mb-1">语言</label>
          <select
            v-model="config.preferences.language"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <button
      @click="saveConfig"
      class="px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
    >
      保存配置
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Config {
  openclaw: {
    url: string
    use_local: boolean
  }
  market: {
    url: string
    enabled: boolean
  }
  preferences: {
    theme: string
    language: string
  }
}

const config = ref<Config>({
  openclaw: { url: 'http://localhost:8000', use_local: true },
  market: { url: 'http://localhost:3001', enabled: true },
  preferences: { theme: 'system', language: 'zh-CN' }
})

const apiKey = ref('')
const connectionStatus = ref<'success' | 'error' | null>(null)

onMounted(async () => {
  try {
    const loaded = await invoke<Config>('get_config')
    config.value = loaded
  } catch (e) {
    console.error('Failed to load config:', e)
  }
})

const saveConfig = async () => {
  try {
    await invoke('save_config', { config: config.value })
    // 保存 API Key 到密钥库
    if (apiKey.value) {
      await invoke('save_api_key', { key: apiKey.value })
    }
    alert('配置已保存')
  } catch (e) {
    console.error('Failed to save config:', e)
    alert('保存失败')
  }
}

const testConnection = async () => {
  try {
    const result = await invoke<boolean>('test_connection')
    connectionStatus.value = result ? 'success' : 'error'
  } catch (e) {
    connectionStatus.value = 'error'
    console.error('Connection test failed:', e)
  }
}
</script>
```

**Step 2: 添加配置 Tauri Commands**

```rust
// commands/mod.rs 添加
mod config_cmd;

#[tauri::command]
pub fn get_config() -> Result<AppConfig, CommandError> {
    Ok(crate::config::load_config().unwrap_or_default())
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), CommandError> {
    crate::config::save_config(&config).map_err(|e| CommandError { message: e.to_string() })
}
```

**Step 3: 提交**

```bash
git add .
git commit -m "feat: add config management UI"
```

---

## 阶段 6: 构建与发布

### Task 6.1: 生产构建

**Step 1: 构建前端**

Run: `npm run build`
Expected: 生成 dist 目录

**Step 2: 构建 Tauri 应用**

Run: `npm run tauri build`
Expected: 生成可执行文件

**Step 3: 提交**

```bash
git add .
git commit -m "chore: build production release"
```

---

## 执行选项

**Plan complete and saved to `docs/plans/2026-03-04-shine-helper-implementation-plan.md`. Two execution options:**

**1. Subagent-Driven (this session)** - 我为每个任务分配新的子代理，任务间进行代码审查，快速迭代

**2. Parallel Session (separate)** - 在新会话中使用 executing-plans，批量执行并设置检查点

**你想选择哪种执行方式？**

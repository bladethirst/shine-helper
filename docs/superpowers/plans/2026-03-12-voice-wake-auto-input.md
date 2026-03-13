# 语音唤醒后自动语音输入功能实现计划

> **状态**: ✅ 已完成
> **完成日期**: 2026-03-13

**Goal:** 实现语音唤醒后自动开启语音识别，将识别结果自动填入聊天输入框，支持半自动发送和完整状态显示

**Architecture:**
- 后端：在语音唤醒 `Processing` 状态完成后，继续保持在 `Listening` 状态，扩展 `voice-result` 事件支持增量更新
- 前端：`useVoiceWake` 扩展支持唤醒后自动语音输入，`ChatInput` 组件新增唤醒聆听状态显示和增量识别结果处理
- 配置：新增 `auto_send_after_wake` 和 `auto_mic_after_wake` 配置项

**Tech Stack:** Tauri 1.5, Rust, Vue 3, TypeScript

---

## Task 1: 扩展后端配置结构

**Files:**
- Modify: `src-tauri/src/config.rs`

- [x] **Step 1: 在 VoiceWakeConfig 中添加两个新字段**
- [x] **Step 2: 验证编译**
- [x] **Step 3: 提交**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceWakeConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    /// Vosk ASR 服务地址
    #[serde(default = "default_vosk_url")]
    pub vosk_url: String,
    /// Vosk ASR API Key
    #[serde(default)]
    pub vosk_api_key: String,
    /// 唤醒后是否自动发送（默认 false=半自动）
    #[serde(default)]
    pub auto_send_after_wake: bool,
    /// 唤醒后是否自动开启麦克风（默认 true）
    #[serde(default = "default_true")]
    pub auto_mic_after_wake: bool,
}

fn default_true() -> bool {
    true
}

impl Default for VoiceWakeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word: "小 Shine".to_string(),
            wake_sounds: vec!["在呢".to_string(), "在的".to_string(), "我在".to_string(), "请说".to_string()],
            silence_timeout: 3000,
            end_words: vec!["结束".to_string(), "停止".to_string()],
            vosk_url: default_vosk_url(),
            vosk_api_key: "".to_string(),
            auto_send_after_wake: false,
            auto_mic_after_wake: true,
        }
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: add auto_send_after_wake and auto_mic_after_wake config options"
```

---

## Task 2: 后端扩展 voice-result 事件支持增量更新

**Files:**
- Modify: `src-tauri/src/commands/voice_wake.rs`

- [x] **Step 1: 扩展 voice-result 事件 payload**
- [x] **Step 2: 验证编译**
- [x] **Step 3: 提交**

---

## Task 3: 前端扩展 useVoiceWake 支持增量结果

**Files:**
- Modify: `src/composables/useVoiceWake.ts`

- [x] **Step 1: 扩展接口和状态**
- [x] **Step 2: 验证 TypeScript 编译**
- [x] **Step 3: 提交**

---

## Task 4: ChatInput 组件新增唤醒聆听状态显示

**Files:**
- Modify: `src/components/ChatInput.vue`

- [x] **Step 1: 修改模板，新增唤醒聆听状态显示**
- [x] **Step 2: 修改 script，整合 useVoiceWake 和增量结果显示**
- [x] **Step 3: 验证编译**
- [x] **Step 4: 提交**

---

## Task 5: 配置页面新增配置项 UI

**Files:**
- Modify: `src/views/ConfigView.vue`

- [x] **Step 1: 查看 ConfigView.vue 当前结构**
- [x] **Step 2: 在语音唤醒配置区块添加两个新配置项**
- [x] **Step 3: 验证编译**
- [x] **Step 4: 提交**

---

## Task 6: 后端处理自动发送逻辑

**Files:**
- Modify: `src-tauri/src/commands/voice_wake.rs`

- [x] **Step 1: 在 Processing 状态检测结束时，根据配置发送 voice-input-complete 事件**
- [x] **Step 2: 验证编译**
- [x] **Step 3: 提交**

---

## Task 7: 前端处理自动发送逻辑

**Files:**
- Modify: `src/components/ChatInput.vue`

- [x] **Step 1: 监听 voice-input-complete 事件，根据配置自动发送**
- [x] **Step 2: 验证编译**
- [x] **Step 3: 提交**

---

## Task 8: 集成测试和验证

**Files:**
- Test: 手动测试

- [x] **Step 1: 测试唤醒功能**
- [x] **Step 2: 测试语音识别输入**
- [x] **Step 3: 测试半自动发送**
- [x] **Step 4: 测试自动发送**
- [x] **Step 5: 测试发送后状态重置**
- [x] **Step 6: 提交测试记录**

---

## 实现总结

所有功能已实现完成，主要变更包括：

### 后端 (`src-tauri/src/config.rs`)
- 新增 `auto_send_after_wake` 和 `auto_mic_after_wake` 配置项

### 后端 (`src-tauri/src/commands/voice_wake.rs`)
- 扩展 `voice-result` 事件支持增量更新 (partial/final)
- 在静音超时时发送 `voice-input-complete` 事件

### 前端 (`src/composables/useVoiceWake.ts`)
- 新增 `wake-listening` 状态
- 支持增量识别结果处理
- 新增 `clearTranscript()` 和 `stopWakeListening()` 方法

### 前端 (`src/components/ChatInput.vue`)
- 新增唤醒聆听状态显示（紫色脉冲动画）
- 监听 `voice-input-complete` 事件实现自动发送
- 发送后自动重启唤醒服务

### 前端 (`src/views/ConfigView.vue`)
- 新增"唤醒后自动开启语音输入"配置
- 新增"唤醒后自动发送"配置

---

## 完整计划验证

**编译验证:**
```bash
cd src-tauri && cargo check
npm run build
```

**预期结果:** 编译成功，无错误

**功能验证:**
- [x] 配置页面可见新增配置项
- [x] 唤醒后自动进入聆听状态
- [x] 识别结果增量显示（partial 灰色，final 正常）
- [x] 半自动/自动发送按配置工作

---

## 变更记录

| 日期 | 操作 | 说明 |
|------|------|------|
| 2026-03-13 | 文档更新 | 标记所有任务为已完成，更新实现总结 |

*文档最后更新：2026-03-13*

# 语音唤醒功能实现进度报告

**功能名称**: 语音唤醒 (Voice Wake-up)  
**实现日期**: 2026-03-08  
**状态**: ✅ 已完成并合并到 home-dev 分支  
**分支**: `feature/voice-wake` → `home-dev`

---

## 1. 功能概述

为 Shine Helper 添加语音唤醒功能，实现：
- **后台持续监听**: 应用启动后在后台持续监听唤醒词
- **语音唤醒**: 检测到"小 Shine"后自动激活
- **语音答复**: 唤醒后播放随机 TTS 回复（如"在呢"、"你说"、"请讲"）
- **语音识别**: 通过 QwenASR WebSocket 服务进行流式语音识别
- **自动上屏**: 识别结果自动填入对话页面输入框
- **自动结束**: 3 秒静音或检测到结束词后返回待机

---

## 2. 技术架构

### 2.1 架构选择
混合方案（Rust 后端处理）：
- Rust 后端处理音频捕获和唤醒词检测
- QwenASR WebSocket 流式识别
- 前端通过 Tauri Event 接收识别结果

### 2.2 状态机
```
Idle ──[检测到"小 Shine"]──▶ Waking ──[TTS 播放完成]──▶ Listening
   ▲                                                        │
   │                    ┌────[静音 3 秒]────────────────────┤
   │                    └────[检测到"结束"]─────────────────┤
   └────────────────────┴──────────────────────────────────┘
```

---

## 3. 实现任务清单

| 阶段 | 任务 | 状态 | 提交 |
|------|------|------|------|
| **Phase 1** | Task 1: 添加 Rust 音频依赖 | ✅ | 已提交 |
| | Task 8: 添加 VoiceConfig 配置 | ✅ | `eb18756` |
| | Task 9: 创建 useVoiceWake Composable | ✅ | `77dfde4` |
| | Task 11: 添加语音配置 UI | ✅ | `77dfde4` |
| **Phase 2** | Task 2: 创建音频捕获模块 | ✅ | `4d7bfe1` |
| | Task 3: 创建唤醒词检测模块 | ✅ | `4d7bfe1` |
| | Task 4: 创建 QwenASR WebSocket 客户端 | ✅ | `4d7bfe1` |
| | Task 5: 创建 TTS 播放模块 | ✅ | `4d7bfe1` |
| **Phase 3** | Task 6: 创建语音状态机 | ✅ | `8ae2a03` |
| | Task 7: 创建语音 Tauri 命令 | ✅ | `74c4475` |
| | Task 10: 更新 ChatInput 组件 | ✅ | `458c5dc` |

---

## 4. 文件变更清单

### 4.1 Rust 后端

| 文件 | 描述 |
|------|------|
| `src-tauri/src/voice/mod.rs` | 语音模块导出 (audio_capture, wake_word, asr_client, tts_player, state_machine) |
| `src-tauri/src/voice/audio_capture.rs` | 音频捕获模块，使用 cpal 库实现麦克风输入 |
| `src-tauri/src/voice/wake_word.rs` | 唤醒词检测模块，基于能量阈值的 VAD 检测 |
| `src-tauri/src/voice/asr_client.rs` | QwenASR WebSocket 客户端，发送音频流并接收识别结果 |
| `src-tauri/src/voice/tts_player.rs` | TTS 播放模块，播放随机唤醒回复 |
| `src-tauri/src/voice/state_machine.rs` | 语音状态机，管理 Idle/Waking/Listening/Processing 状态流转 |
| `src-tauri/src/commands/voice_cmd.rs` | Tauri 命令 (start_voice_wake, stop_voice_wake, get/set_voice_config) |
| `src-tauri/src/config.rs` | 添加 VoiceConfig 配置结构体 |
| `src-tauri/src/main.rs` | 注册 VoiceAppState 和语音命令 |
| `src-tauri/Cargo.toml` | 添加 cpal, rodio, rand, url 等依赖 |

### 4.2 前端

| 文件 | 描述 |
|------|------|
| `src/composables/useVoiceWake.ts` | Vue 3 Composable，管理语音状态和事件监听 |
| `src/components/ChatInput.vue` | 添加麦克风按钮和语音状态指示器 |
| `src/views/ConfigView.vue` | 添加语音唤醒配置区块 |

### 4.3 文档

| 文件 | 描述 |
|------|------|
| `docs/plans/2026-03-08-shine-helper-voice-wake-design.md` | 设计文档 |
| `docs/plans/2026-03-08-shine-helper-voice-wake-implementation-plan.md` | 实现计划 |
| `docs/progress/2026-03-08-voice-wake-implementation-progress.md` | 进度报告（本文件） |

---

## 5. 配置项

在配置页面新增"语音唤醒"区块：

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `voice.enabled` | boolean | false | 是否启用语音唤醒 |
| `voice.wake_word` | string | "小 Shine" | 唤醒词 |
| `voice.wake_sounds` | string[] | ["在呢", "你说", "请讲"] | TTS 回复语列表 |
| `voice.silence_timeout` | number | 3000 | 静音超时（毫秒） |
| `voice.end_words` | string[] | ["结束", "停止"] | 结束词 |
| `voice.qwen_asr_url` | string | "ws://localhost:5000" | QwenASR WebSocket 地址 |
| `voice.qwen_asr_api_key` | string | "" | QwenASR API Key |

---

## 6. 使用说明

### 6.1 系统依赖（Linux 麒麟系统）
```bash
sudo apt-get install libasound2-dev pkg-config
```

### 6.2 配置步骤
1. 打开应用，进入"配置"页面
2. 在"语音唤醒"区块：
   - 勾选"启用语音唤醒"
   - 设置 QwenASR 服务地址（如 `ws://localhost:5000`）
   - 输入 API Key（如果需要认证）
3. 点击"保存配置"

### 6.3 使用方式
1. 应用启动后自动在后台监听唤醒词
2. 说出"小 Shine"唤醒应用
3. 听到回复（如"在呢"）后开始说话
4. 识别结果自动显示在输入框
5. 说完后等待 3 秒或说"结束"自动返回待机

---

## 7. 已知限制

1. **唤醒词检测**: 当前使用简化的能量阈值 VAD 检测，生产环境应集成 Porcupine 或 Snowboy 等专业唤醒词引擎
2. **TTS 回复**: 当前仅打印日志，未实现实际语音播放，生产环境应集成 TTS 服务或预置音频文件
3. **系统依赖**: Linux 系统需要安装 `libasound2-dev` (ALSA 开发库)

---

## 8. 后续工作

- [ ] 集成 Porcupine 或 Snowboy 唤醒词引擎，提升唤醒准确率
- [ ] 集成 TTS 服务（如 edge-tts）或预置音频文件，实现语音回复
- [ ] 添加唤醒词自定义功能
- [ ] 添加语音音量调节
- [ ] 添加唤醒历史记录
- [ ] 优化静音检测算法，减少误判

---

## 9. 提交历史

```
d6592e4 Merge feature/voice-wake: Add voice wake-up functionality
74c4475 feat: add voice Tauri commands
8ae2a03 feat: add voice state machine
458c5dc feat: add voice wake-up UI to ChatInput
4d7bfe1 feat: add TTS player module
77dfde4 feat: add useVoiceWake composable
eb18756 feat: add VoiceConfig to app config
6244d35 docs: add voice wake-up feature design
d326ae8 docs: add voice wake-up implementation plan
```

---

**报告生成日期**: 2026-03-08  
**实现团队**: Shine Team  
**技术栈**: Tauri 1.5, Rust, Vue 3, TypeScript, QwenASR

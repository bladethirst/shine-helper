# OpenClaw 权限问题修复说明

**问题**: Error: EACCES: permission denied, mkdir '/root/Desktop/workspace'

**日期**: 2026-03-13

---

## 一、问题原因

当使用普通用户（如 `SHDL`）运行 `start.sh` 时，出现权限错误的原因是：

1. **配置文件路径硬编码**：`openclaw.json` 中的 `workspace` 设置为 `/root/Desktop/workspace`
2. **`$HOME` 环境变量问题**：某些情况下 `$HOME` 可能指向 `/root` 而不是用户的实际家目录
3. **权限不足**：普通用户无法在 `/root` 目录下创建文件夹

---

## 二、解决方案

### 1. 修改 `openclaw.json` 配置文件

将硬编码的路径改为使用环境变量占位符：

```json
"agents": {
  "defaults": {
    "workspace": "${HOME}/Desktop/workspace"
  }
}
```

这样 OpenClaw 会自动使用当前用户的 `$HOME` 环境变量。

### 2. 修改 `start.sh` 启动脚本

确保环境变量正确设置：

```bash
# 确保使用当前用户的实际家目录
REAL_HOME="$HOME"

# OpenClaw 状态目录（使用 $HOME 环境变量）
OPENCLAW_STATE_DIR="$HOME/Desktop/workspace/.openclaw"

# OpenClaw 主目录（使用应用程序目录）
OPENCLAW_HOME="$OPENCLAW_APP_DIR"

# 创建 Desktop 目录（如果不存在）
if [ ! -d "$REAL_HOME/Desktop" ]; then
    mkdir -p "$REAL_HOME/Desktop"
fi

# 创建 OpenClaw 工作目录
if [ ! -d "$OPENCLAW_STATE_DIR" ]; then
    mkdir -p "$OPENCLAW_STATE_DIR"
fi
```

### 3. 设置环境变量

启动脚本中导出以下环境变量：

```bash
export OPENCLAW_HOME="$OPENCLAW_HOME"
export OPENCLAW_STATE_DIR="$OPENCLAW_STATE_DIR"
export OPENCLAW_CONFIG_PATH="$OPENCLAW_CONFIG_PATH"
```

---

## 三、验证步骤

1. **检查当前用户和家目录**：
   ```bash
   whoami
   echo $HOME
   ```
   应该输出：
   ```
   SHDL
   /home/SHDL
   ```

2. **检查目录权限**：
   ```bash
   ls -ld /home/SHDL/Desktop
   ls -ld /root/Desktop
   ```

3. **启动应用**：
   ```bash
   ./start.sh
   ```

4. **查看日志**：
   ```bash
   tail -20 /tmp/openclaw.log
   ```

---

## 四、预期结果

启动后应该看到：

```
[Shine Helper] 当前用户：SHDL
[Shine Helper] HOME 目录：/home/SHDL
[Shine Helper] OpenClaw 工作目录：/home/SHDL/Desktop/workspace/.openclaw
[Shine Helper] 创建 OpenClaw 工作目录：/home/SHDL/Desktop/workspace/.openclaw
```

OpenClaw 工作目录应该位于：
- `/home/SHDL/Desktop/workspace/.openclaw`

---

## 五、常见问题

### Q1: 如果 `$HOME` 仍然是 `/root` 怎么办？

使用 `su` 或 `sudo` 切换到普通用户时，使用 `-` 参数：
```bash
su - SHDL
# 或
sudo -i -u SHDL
```

### Q2: Desktop 目录在哪里？

在麒麟系统中，桌面目录通常是：
- `/home/SHDL/Desktop`

如果目录不存在，启动脚本会自动创建。

### Q3: 如果还是有权限问题怎么办？

检查工作目录的实际权限：
```bash
ls -la /home/SHDL/Desktop/workspace/
```

如果需要，修改所有者：
```bash
sudo chown -R SHDL:SHDL /home/SHDL/Desktop/workspace/
```

---

## 六、修改文件列表

| 文件 | 修改内容 |
|------|----------|
| `start.sh` | 添加用户和家目录检查，确保目录存在 |
| `resources/openclaw/data/openclaw.json` | workspace 路径改为 `${HOME}/Desktop/workspace` |

---

*文档创建：2026-03-13*

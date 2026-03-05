@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"
set "OPENCLAW_DIR=%SCRIPT_DIR%\resources\openclaw"
set "NODE_DIR=%OPENCLAW_DIR%\node"
set "OPENCLAW_APP_DIR=%OPENCLAW_DIR%\openclaw"
set "SHINE_EXE=%SCRIPT_DIR%\shine_helper.exe"

echo [Shine Helper] 检查运行环境...

REM 检查必要文件
if not exist "%NODE_DIR%\node.exe" (
    echo [错误] 未找到 Node.js 运行时
    echo 请确保 resources\openclaw\node 目录存在 Node.js
    pause
    exit /b 1
)

if not exist "%OPENCLAW_APP_DIR%\package.json" (
    echo [错误] 未找到 OpenClaw 应用
    echo 请确保 resources\openclaw\openclaw 目录存在
    pause
    exit /b 1
)

if not exist "%SHINE_EXE%" (
    echo [错误] 未找到 shine_helper.exe
    pause
    exit /b 1
)

echo [Shine Helper] 检查 OpenClaw 服务状态...

REM 检查端口 18789 是否已被占用
netstat -ano | findstr ":18789 " > nul
if %errorlevel% equ 0 (
    echo [Shine Helper] OpenClaw 服务已在运行
) else (
    echo [Shine Helper] 启动 OpenClaw 服务...
    
    if not exist "%OPENCLAW_DIR%\data" (
        mkdir "%OPENCLAW_DIR%\data"
    )
    
    cd /d "%OPENCLAW_APP_DIR%"
    start "OpenClaw" /b "%NODE_DIR%\node.exe" "%OPENCLAW_APP_DIR%\gateway.js" --port 18789 --data "%OPENCLAW_DIR%\data"
    
    echo [Shine Helper] 等待服务启动...
    timeout /t 8 /nobreak > nul
    
    REM 再次检查
    netstat -ano | findstr ":18789 " > nul
    if %errorlevel% neq 0 (
        echo [警告] OpenClaw 可能启动失败，继续尝试启动应用...
    ) else (
        echo [Shine Helper] OpenClaw 服务已就绪
    )
)

echo [Shine Helper] 启动桌面应用...
start "" "%SHINE_EXE%"

echo [Shine Helper] 启动完成
exit
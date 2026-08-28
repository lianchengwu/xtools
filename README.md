# 🪐 xtools

<p align="center">
  <img src="xtools.svg" width="96" height="96" alt="xtools logo" />
</p>

<p align="center">
  <strong>为 Linux 与 Windows 桌面打造的轻量级极简悬浮球工具箱</strong>
  <br />
  <em>纯 Rust + Slint / GTK4 构建 · 环绕轨道交互 · 多进程独立窗口 · 秒级极速响应</em>
</p>

<p align="center">
  <a href="#-核心特性"><img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-3E7BFA?logo=linux" alt="Platform Support" /></a>
  <a href="#-技术栈"><img src="https://img.shields.io/badge/Language-Rust%202024-F74C00?logo=rust" alt="Rust 2024" /></a>
  <a href="#-技术栈"><img src="https://img.shields.io/badge/GUI-Slint%20%2B%20GTK4-4B32C3" alt="GUI Stack" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT" /></a>
</p>

---

## ✨ 核心特性

- 🌐 **轨道悬浮交互**：常驻屏幕的极简主悬浮球，支持自由拖拽；点击平滑弹出 3 颗环绕功能球（时间戳 / JSON / 翻译），零干扰桌面视线。
- ⚡ **多进程独立架构**：主程序仅负责悬浮球呈现与调度；各个工具窗口均为独立编译的 Rust 进程，彼此崩溃隔离、资源解耦、随开随走。
- 🎯 **单例与极速聚焦**：各功能窗口通过平台原生 IPC（Linux Unix Domain Socket / Windows Named Pipe）实现单例机制；再次启动时毫秒级拉起并聚焦已有窗口。
- 🎨 **统一度量与主题系统**：通过 `xtools-ui` 统一 Slint 主题 token、共享组件与无边框沉浸式窗口组件；自动跟随系统浅色 / 深色模式实时切换。
- 🐧 **现代 Linux 桌面原生支持**：支持 Wayland（`gtk4-layer-shell` 协议层）与 X11 混合环境，内置 SNI 规范系统托盘（System Tray）支持。
- 🪟 **Windows 原生支持**：基于 Win32 分层窗口（`UpdateLayeredWindow` 逐像素透明）与 Named Pipe 单例 IPC，声明 Per-Monitor V2 DPI 感知，提供悬浮球与系统托盘，CI 自动产出便携 ZIP 与 Inno Setup 安装包。

---

## 🛠️ 工具集概览

| 图标 | 工具组件 | 说明 | 核心能力 |
| :---: | :--- | :--- | :--- |
| 🕒 | **`xtools-time`** | 时间戳转换 | • 实时时间毫秒级追踪<br />• Unix 秒 / 毫秒 ↔ 本地标准时间双向转换<br />• 一键快速复制结果至剪贴板 |
| 📋 | **`xtools-json`** | JSON 实用工具 | • 标准 JSON 格式化与美化排版<br />• 单行压缩（Minify）<br />• 字符串转义 / 反转义（Escape/Unescape）<br />• 精确行号与列号的错误定位提示 |
| 🌐 | **`xtools-trans`** | 划词 / 文本翻译 | • 多语言文本快速双向互译<br />• 可插拔的 `TranslateEngine` 架构设计<br />• 后台非阻塞网络请求，交互丝滑不卡顿 |

---

## 🏗️ 架构设计

```text
               +-----------------------------+
               |         xtools-host         |  <--- GTK4 / Layer-Shell（Linux）· Win32 分层窗口（Windows）悬浮主球 & 托盘
               +--------------+--------------+
                              |
                (Unix Socket / Named Pipe / IPC Spawn)
                              |
        +---------------------+---------------------+
        |                     |                     |
+-------v-------+     +-------v-------+     +-------v-------+
|  xtools-time  |     |  xtools-json  |     | xtools-trans  |  <--- 独立 Slint 窗口进程
+-------+-------+     +-------+-------+     +-------+-------+
        |                     |                     |
        +---------------------+---------------------+
                              |
                  [ 共享库: xtools-ui ]
             (主题样式 / 单例管理 / 窗口修饰)
```

- **`crates/xtools-host`**：主球与环绕动画宿主，Linux 侧 Cairo 自绘矢量图形 / Windows 侧软件光栅化，系统托盘图标管理。
- **`crates/xtools-time`**：基于 `jiff` 的高精度时间戳处理工具。
- **`crates/xtools-json`**：基于 `serde_json` 的 JSON 格式化与校验工具。
- **`crates/xtools-trans`**：基于 `ureq` 的轻量级翻译工具。
- **`crates/xtools-ui`**：共享 UI 组件体系、Slint 全局主题、X11/Wayland 窗口修饰与单例 IPC。

---

## 🚀 编译与安装

### 系统依赖准备

确保安装了 Rust 1.85+ 以及相关系统开发库：

**Ubuntu / Debian:**
```bash
sudo apt update
sudo apt install -y build-essential libgtk-4-dev libgtk4-layer-shell-dev fontconfig libfontconfig1-dev
```

**Fedora / RHEL:**
```bash
sudo dnf install -y gcc gtk4-devel gtk4-layer-shell-devel fontconfig-devel
```

**Arch Linux:**
```bash
sudo pacman -S --needed base-devel gtk4 gtk4-layer-shell fontconfig
```

### 编译构建

```bash
# 克隆仓库
git clone git@github.com:lianchengwu/xtools.git
cd xtools

# 检查代码与依赖
cargo check --workspace

# 编译所有二进制（Release 优化）
cargo build --release
```

Windows 支持构建全部组件（三个工具 + 悬浮球 host 与系统托盘），无需任何额外系统依赖：

```powershell
cargo build --workspace --release
```

Windows 侧基于 Win32 分层窗口（`UpdateLayeredWindow` 逐像素透明）与 Named Pipe 单例 IPC 实现，声明 Per-Monitor V2 DPI 感知；Linux 专属的 GTK layer-shell / X11 / KWin 代码不会参与编译。

编译产物位于 `target/release/`：
- `xtools-host`（主悬浮球与托盘）
- `xtools-time`（时间戳工具）
- `xtools-json`（JSON 工具）
- `xtools-trans`（翻译工具）

---

## 💻 运行使用

直接启动宿主主程序：

```bash
./target/release/xtools-host
```

- **移动主球**：鼠标按住主球拖动至屏幕任意位置。
- **展开/收起功能球**：单机主球，弹出 3 颗轨道功能球。
- **唤起工具**：点击对应的功能球，立即调出独立工具窗口；再次点击对应球自动聚焦窗口。
- **托盘菜单**：在系统托盘右键图标可选择“显示/隐藏”或“退出程序”。

也可以单独运行任意独立工具窗口：
```bash
./target/release/xtools-time
./target/release/xtools-json
./target/release/xtools-trans
```

---

## 📦 桌面集成与打包

仓库内包含标准的 Linux 桌面集成文件：
- `xtools.desktop`：桌面快捷方式定义
- `xtools.svg`：应用矢量图标

支持使用 `cargo-generate-rpm` 快速打包 RPM 安装包：
```bash
cargo generate-rpm -p crates/xtools-host
```

Windows 侧由 CI（`release.yml`）自动构建便携 ZIP 与 Inno Setup 安装包（安装脚本见 `scripts/windows-installer.iss`），并在每次推送时运行 Windows 单例冒烟测试（`scripts/windows-singleton-smoke.ps1`）。

---

## 📄 开源许可证

本项目采用 [MIT License](LICENSE) 许可证开源。

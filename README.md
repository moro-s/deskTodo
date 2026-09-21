# deskTodo

轻量级桌面日历待办小部件：在桌面上常驻显示当月日历，每天一个格子，点选日期即可管理当天的待办事项，支持到点提醒与系统托盘。

## 功能特性

- 📅 **月历视图**：7 列日历网格，点击日期格子切换选中日期，格子内显示待办摘要与角标
- ✅ **待办管理**：添加、勾选完成、单条删除、一键清除已完成
- ⏰ **到点提醒**：添加待办时可设定提醒时间，到点自动唤起窗口、闪烁任务栏并弹出提醒横幅（托盘隐藏状态下同样生效）
- 📌 **窗口置顶**：全局快捷键 `Ctrl+Alt+T` 随时切换，也可点击标题栏「置顶」按钮
- 🎨 **主题切换**：标题栏按钮循环切换深色 / 浅色 / 墨绿三套配色，选择自动保存
- 🖥️ **系统托盘**：关闭 / 最小化按钮和 `Alt+F4` 均隐藏到托盘；左键单击托盘图标切换显示；托盘菜单支持显示 / 隐藏、切换置顶、退出
- 💾 **本地持久化**：数据以 JSON 存储在用户数据目录，无需联网
- 🌏 **跨平台**：Windows / macOS / Linux，自动加载各平台中文字体

## 快捷键

| 快捷键 | 功能 |
| --- | --- |
| `Ctrl+Alt+T` | 切换窗口置顶 |
| `Enter` | 输入框内确认添加待办 |

## 数据存储

| 平台 | 路径 |
| --- | --- |
| Windows | `%APPDATA%\deskTodo\todos.json`（待办）、`%APPDATA%\deskTodo\config.json`（配置） |
| macOS | `~/Library/Application Support/deskTodo/` |
| Linux | `~/.local/share/deskTodo/` |

## 从源码构建

需要 Rust 1.95+（推荐通过 [rustup](https://rustup.rs/) 安装）。

```bash
git clone https://github.com/moro-s/deskTodo.git
cd deskTodo
cargo build --release
```

构建产物位于 `target/release/deskTodo`（Windows 为 `deskTodo.exe`）。

### Linux 系统依赖

托盘图标功能依赖以下系统库：

```bash
# Debian / Ubuntu
sudo apt install libgtk-3-dev libxdo-dev libappindicator3-dev

# Arch Linux
sudo pacman -S gtk3 xdotool libappindicator-gtk3
```

## 技术栈

- [eframe](https://github.com/emilk/egui/tree/main/crates/eframe) / [egui](https://github.com/emilk/egui) 0.36 —— 即时模式 GUI，采用 glow（OpenGL）渲染后端
- [tray-icon](https://github.com/tauri-apps/tray-icon) —— 跨平台系统托盘
- [global-hotkey](https://github.com/tauri-apps/global-hotkey) —— 全局快捷键
- [chrono](https://github.com/chronotope/chrono) —— 日期时间处理
- [serde](https://serde.rs) / serde_json —— 数据序列化与持久化

## License

[Apache-2.0](LICENSE)

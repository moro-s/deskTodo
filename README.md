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

需要 Rust 1.95+（推荐通过 [rustup](https://rustup.rs/) 安装），以及 [eguidev-private](https://github.com/moro-s/eguidev-private) 私有仓库的访问权限（依赖经 git 引入）。

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

## 项目结构

```
src/
├── main.rs           # 入口：窗口选项与启动
├── app.rs            # 应用状态与生命周期（eframe::App 实现）
├── core/             # 核心逻辑
│   ├── config.rs     # 配置模型
│   ├── logger.rs     # 分级日志系统
│   ├── models.rs     # 数据模型：TodoItem、TodoStore
│   ├── reminder.rs   # 提醒调度纯函数
│   └── storage.rs    # 本地 JSON 持久化
├── platform/         # 平台集成
│   ├── hotkey.rs     # 全局快捷键（Ctrl+Alt+T 置顶）
│   ├── icon.rs       # 程序化生成日历图标
│   ├── notify.rs     # 系统通知
│   ├── tray.rs       # 系统托盘与事件队列
│   └── window.rs     # 窗口常量与工具
└── ui/               # 界面渲染
    ├── calendar.rs   # 月历网格
    ├── editor.rs     # 待办编辑器
    ├── settings_page.rs # 设置页面
    ├── text.rs       # 文本测量
    ├── theme.rs      # 主题定义与全局样式
    ├── time_picker.rs # 时间选择器
    ├── titlebar.rs   # 自绘标题栏
    ├── todo_list.rs  # 待办列表
    └── widgets.rs    # 可复用自绘控件
```

## 开发与自动化测试（eguidev）

项目集成了 [eguidev](https://github.com/cortesi/eguidev) —— 运行在应用进程内的 egui 自动化测试工具链，支持通过 Luau 脚本驱动真实控件（读取状态、注入输入、截图），并可通过 MCP 连接 AI 代理。

### 构建

```bash
# 常规构建（不含开发工具）
cargo build --release

# 带开发工具构建（启用内嵌 MCP 服务与脚本执行）
cargo build --features devtools
```

`devtools` feature 只在开发时启用：`eguidev` 核心库（控件标记与追踪）始终编译且对性能零开销（无 MCP 客户端连接时自动空转），`eguidev_runtime`（脚本引擎、MCP 服务、截图）仅在该 feature 下引入，不影响发布构建。

### edev CLI

`edev` 是配套的命令行启动器，从 eguidev 私有仓库安装（保证与运行时版本一致）：

```bash
cargo install --git https://github.com/moro-s/eguidev-private.git edev
```

常用命令（通过项目根目录的 `.edev.toml` 配置启动）：

```bash
edev dump                          # 打印控件树（含所有标记控件的状态）
edev eval smoketest/20_widget_flow.luau  # 运行单个脚本
edev smoke                         # 运行完整冒烟测试套件
edev mcp                           # 作为 MCP 服务器运行（供 AI 代理连接）
```

### 冒烟测试

`smoketest/` 目录存放 Luau 冒烟脚本，`edev smoke` 一键运行：

```bash
edev smoke
```

脚本通过字符串 id（如 `todo.input`、`todo.submit`、`titlebar.settings`）引用被标记的控件。

### 连接 AI 代理（MCP）

将 `edev` 注册为 MCP 服务器后，AI 代理即可驱动应用进行自动化测试：

```json
{
  "mcpServers": {
    "eguidev": {
      "command": "edev",
      "args": ["mcp"]
    }
  }
}
```

## 技术栈

- [eframe](https://github.com/emilk/egui/tree/main/crates/eframe) / [egui](https://github.com/emilk/egui) 0.36 —— 即时模式 GUI，采用 glow（OpenGL）渲染后端
- [eguidev](https://github.com/cortesi/eguidev) —— 进程内 UI 自动化与 MCP 工具链（私有仓库 [eguidev-private](https://github.com/moro-s/eguidev-private)，经 git 依赖引入）
- [tray-icon](https://github.com/tauri-apps/tray-icon) —— 跨平台系统托盘
- [global-hotkey](https://github.com/tauri-apps/global-hotkey) —— 全局快捷键
- [chrono](https://github.com/chronotope/chrono) —— 日期时间处理
- [serde](https://serde.rs) / serde_json —— 数据序列化与持久化

## License

[Apache-2.0](LICENSE)

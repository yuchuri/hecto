# Hecto

一个用 Rust 编写的高性能现代终端文本编辑器。灵感来源于经典的 [Kilo](https://antirez.com/news/108) 编辑器及系列教程，基于 Rust 2024 Edition 构建，具备开箱即用的 Unicode / Emoji 全面支持、平滑视口滚动、增量搜索以及健壮的终端错误恢复机制。

---

## 目录

- [项目简介](#项目简介)
- [核心功能](#核心功能)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [核心模块设计](#核心模块设计)
- [Unicode 深度处理机制](#unicode-深度处理机制)
- [快捷键说明](#快捷键说明)
- [编译与运行](#编译与运行)
- [许可证](#许可证)

---

## 项目简介

**Hecto** 是一款运行于终端的微型代码与文本编辑器。它旨在探索系统级编程语言在文本渲染、终端底层控制与现代字符编码处理上的实践。

在终端环境中，中日韩双宽字符（CJK）、Emoji、组合连字符（Combining Characters）以及控制字符常常会导致光标漂移、排版错位甚至花屏崩溃。Hecto 深入解决了终端文本编辑器中复杂的 **Unicode 字素簇（Grapheme Clusters）与显示列宽对齐** 问题，并结合脏标记渲染引擎，提供了轻快而稳定的文本编辑体验。

---

## 核心功能

- **基础文件与编辑操作**：
  - 支持从命令行参数直接打开现有文件或创建新文件。
  - 支持普通模式字符插入、换行裂行、退格删除（Backspace）、向前删除（Delete）及跨行自动合并。
  - 文件保存（`Ctrl-S`）及新文件动态 "Save as" 另存为提示。
- **平滑导航与视口管理**：
  - 光标移动支持方向键、`PageUp` / `PageDown`、`Home` / `End`。
  - 自动边界检测与光标视口追踪，支持垂直与水平方向的自适应平滑滚动。
- **实时增量搜索**：
  - `Ctrl-F` 启动搜索栏，支持实时随打随搜（Incremental Search）。
  - 支持方向键向前/向后切换搜索匹配项。
  - 支持 `Enter` 定位确认与 `Esc` 取消并无损还原原光标/滚动视口位置。
- **状态指示与交互提示**：
  - 状态栏实时显示文件名、总行数、脏标记（`[modified]`）及当前光标所在行列。
  - 底部交互栏支持多模式复用（消息通知 / 命令输入 / 搜索提示）。
  - 临时消息具有自动过期机制（默认 5 秒后自动淡出清理）。
- **健壮性与安全机制**：
  - **防误退机制**：检测到未保存修改时，按下 `Ctrl-Q` 会发出警告，需连续按 3 次确认方可强制退出。
  - **Panic 恢复钩子**：注册自定义 `std::panic` hook，在发生不可预期崩溃时第一时间还原终端 Raw 模式、显示光标并退出 Alternate Screen，绝不弄乱用户的终端。

---

## 技术栈

| 模块 / 依赖 | 版本 / 说明 | 用途 |
| :--- | :--- | :--- |
| **Rust** | Edition 2024 | 核心实现语言，保障内存安全与极高运行效率 |
| **[crossterm](https://crates.io/crates/crossterm)** | `0.29.0` | 跨平台终端底层抽象（Raw 模式控制、备用屏切换、光标操控、键盘/窗口 Resize 事件监听） |
| **[unicode-segmentation](https://crates.io/crates/unicode-segmentation)** | `1.13.3` | 基于 UAX #29 标准进行 Unicode 扩展字素簇（Grapheme Clusters）分割 |
| **[unicode-width](https://crates.io/crates/unicode-width)** | `0.2.3` | 计算字符与字符串在等宽终端中的实际显示列宽（全角/半角/零宽识别） |

---

## 项目结构

```text
hecto/
├── Cargo.toml                  # 项目依赖与元数据配置
├── Cargo.lock
├── samples/                    # Unicode 与字符替换测试样例
│   ├── test-character-replacement.txt
│   ├── test-graphemes-1.txt
│   └── test-graphemes-2.txt
└── src/
    ├── main.rs                 # 程序入口，启动 Editor 事件循环
    ├── lib.rs                  # 库接口定义，导出 editor 模块
    └── editor.rs               # 主编辑器控制器 (Editor 生命周期与命令调度)
        ├── command.rs          # 终端按键事件到抽象 Command 的映射与转换
        ├── commandbar.rs       # 底部命令行输入组件（搜索输入、另存为输入）
        ├── documentstatus.rs   # 文档状态数据抽象（行数、光标、脏标记）
        ├── line.rs             # 核心文本行结构（Unicode 索引、宽度前缀和与切片视图）
        ├── messagebar.rs       # 底部临时消息通知组件（支持时间过期与按需擦除）
        ├── position.rs         # 终端物理行列坐标 (Row, Col, Position)
        ├── size.rs             # 区域尺寸模型 (Size)
        ├── statusbar.rs        # 底部状态栏组件（文档信息与右对齐光标信息）
        ├── terminal.rs         # Crossterm 终端操作封装与 Panic 保护
        ├── uicomponent.rs      # UI 组件 Trait（定义脏标记判断与绘制生命周期）
        └── view.rs             # 核心编辑视口控制器 (View)
            ├── buffer.rs       # 内存文本缓冲区 (Buffer: 行列表与持久化)
            ├── fileinfo.rs     # 文件路径元信息封装
            ├── location.rs     # 文档逻辑坐标 (Location: 行索引与字素索引)
            └── searchinfo.rs   # 搜索会话快照与撤销上下文
```

---

## 核心模块设计

### 1. 主事件驱动：`Editor`
`Editor` 是整个应用的中枢，采用经典的事件驱动循环模型：
- **屏幕重绘（`refresh_screen`）**：在等待用户输入前，按照 `MessageBar/CommandBar -> StatusBar -> View -> Caret` 顺序按需刷新。
- **事件捕获（`evaluate_event`）**：从 `crossterm::event::read()` 获取键盘按下或窗口尺寸变化事件。
- **命令模式分发**：根据当前所处交互状态（`PromptType::None`、`PromptType::Save` 或 `PromptType::Search`），将输入命令路由到对应的状态处理器。

### 2. 文本视口与缓冲区：`View` & `Buffer`
- **`Buffer`**：纯粹的数据模型层，持有 `Vec<Line>`。负责文本行的增删、分割、拼接、以及基于全文档的文本查找和读写磁盘。
- **`View`**：表现层控制器，维护视口可见范围（`size`）、滚动偏移量（`scroll_offset: Position`）以及逻辑编辑位置（`text_location: Location`）。它负责处理上下左右、翻页等移动命令，并将逻辑文本坐标投影为物理终端光标坐标。

### 3. 组件化抽象：`UIComponent`
所有的界面区域（`View`、`StatusBar`、`MessageBar`、`CommandBar`）均实现了 `UIComponent` trait：
```rust
pub trait UIComponent {
    fn set_needs_redraw(&mut self, value: bool);
    fn needs_redraw(&self) -> bool;
    fn set_size(&mut self, size: Size);
    fn draw(&mut self, origin_row: usize) -> io::Result<()>;
    fn render(&mut self, origin_row: usize) {
        if self.needs_redraw() {
            if self.draw(origin_row).is_ok() {
                self.set_needs_redraw(false);
            }
        }
    }
}
```
通过 **脏标记（Dirty Flag）模式**，仅在状态发生实质变更时才触发终端行重绘，极大减少了终端闪烁和无谓的 I/O 开销。

### 4. 终端底层安全抽象：`Terminal`
- 封装 Raw 模式、Alternate Screen 的进出，以及行清理、反色打印、光标显隐等操作。
- 在 `Editor::new()` 时通过 `std::panic::set_hook` 注入了自动恢复逻辑：即便程序意外发生 panic，终端也能平稳退出 Alternate Screen 并还原配置。

---

## Unicode 深度处理机制

在现代终端开发中，Unicode 是最具挑战性的环节。Hecto 在 `Line` 模块中实现了严谨的多层坐标映射架构：

```text
 逻辑字符流:   [ "H",   "e",   "l",   "l",   "o",   " ",   "世",   "界",   "👋" ]
 字节偏移量:    0      1      2      3      4      5      6      9      12     16 (ByteIdx)
 字素簇索引:    0      1      2      3      4      5      6      7       8        (GraphemeIdx)
 终端显示列:    0      1      2      3      4      5      6      8      10     12 (Col)
                                                            └── +2 ──┘└── +2 ──┘
```

### 1. 三层索引解耦
- **`ByteIdx`（字节索引）**：用于 Rust 原生 `String` 底层切片与内存操作。
- **`GraphemeIdx`（字素索引）**：人类感知中的“单个字符”。通过 `unicode-segmentation` 的 `grapheme_indices(true)` 拆分，光标移动与删除均以此为单位（如 Emoji、变音符号不会被撕裂半个字符）。
- **`Col`（显示列索引）**：终端实际占用的列数。通过 `unicode-width` 统计宽度前缀和（`widths: Vec<Col>`），实现 $O(1)$ 的光标屏幕横坐标定位。

### 2. 双向快速检索映射
- `Line::grapheme_idx_to_byte_idx`: 允许按字素定位到字节范围。
- `Line::byte_idx_to_grapheme_idx`: 使得底层原生字符串搜索（如 `substr.find(query)`）找到的字节位置能瞬间映射回视口光标所在的字素位置。

### 3. 控制字符与特殊符号可视化替代
针对终端易崩坏的不可见字符与控制字符，Hecto 会在渲染阶段动态替换为直观占位符，统一按 1 列宽渲染：
- 制表符 `\t` $\rightarrow$ `' '`（空格）
- 非 ASCII 空白字符 $\rightarrow$ `'␣'`
- 零宽控制字符 $\rightarrow$ `'▯'`
- 其他零宽字符 $\rightarrow$ `'·'`

### 4. 水平滚动与双宽字符截断保护（Padding）
当视口发生水平横向滚动时，若视口左边界或右边界恰好切断了一个 2 单元格宽的双宽字符（如中文字符或 Emoji）：
`Line::get_visible_graphemes` 会检测边界重叠，自动开启 `pad_left` 或 `pad_right`，使用单列省略符 `⋯` 进行优雅补齐，彻底避免了终端排版错位与乱码重叠。

---

## 快捷键说明

| 快捷键 | 功能描述 |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>Q</kbd> | 退出编辑器（若有未保存内容，需连续按下 3 次） |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | 保存文件（若是未命名文件则唤起另存为提示） |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | 进入搜索模式 |
| <kbd>Esc</kbd> | 取消当前操作（退出搜索并还原原位置、取消保存提示） |
| <kbd>Enter</kbd> | 换行（编辑时） / 确认保存（保存时） / 确认定位（搜索时） |
| <kbd>Backspace</kbd> | 删除光标左侧的一个完整字素（支持跨行合并） |
| <kbd>Delete</kbd> | 删除光标处的一个完整字素（支持跨行合并） |
| <kbd>↑</kbd> <kbd>↓</kbd> <kbd>←</kbd> <kbd>→</kbd> | 光标移动 / 搜索模式下查找上一个或下一个匹配项 |
| <kbd>PageUp</kbd> / <kbd>PageDown</kbd> | 向上 / 向下翻动一整屏 |
| <kbd>Home</kbd> / <kbd>End</kbd> | 快速跳转至当前行首 / 行尾 |

---

## 编译与运行

### 1. 前置依赖
确保本机已安装最新的 Rust 稳定版工具链（建议 Rust 1.85+，支持 Edition 2024）：
```bash
rustc --version
cargo --version
```

### 2. 编译项目
```bash
# Debug 模式编译
cargo build

# Release 模式优化编译
cargo build --release
```

### 3. 运行编辑器
```bash
# 打开空编辑器
cargo run

# 打开指定文件进行编辑
cargo run -- samples/test-graphemes-1.txt

# 或者直接运行编译出的二进制程序
./target/release/hecto path/to/your/file.txt
```

---

## 许可证

本项目遵循 MIT 协议开源。详情参见 [LICENSE](LICENSE) 文件（如有）。

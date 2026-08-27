# 经典游戏合集 (Classic Games Collection)

[![GitHub](https://img.shields.io/badge/GitHub-amonisachicken%2Fclassic--games--collection-blue)](https://github.com/amonisachicken/classic-games-collection)

一个用 Rust 编写的终端经典游戏合集 CLI 应用，包含 **贪吃蛇 / 打砖块 / 俄罗斯方块 / 飞机大战 / 五子棋** 五款游戏。

- 跨平台：支持 **Linux / macOS / Windows**（基于 [crossterm](https://github.com/crossterm-rs/crossterm)）
- 纯终端运行，兼容 **XTerm256 色**
- 主菜单选择游戏（五合一），每款游戏独立计分
- 用户名与分数记录保存在**家目录**

## 运行

```bash
cargo run --release
```

要求：Rust 1.70+（在任意终端中运行，建议终端至少 80×24）。

> 若终端尺寸过小，界面会自动裁剪居中，不会报错；建议用 100×30 以上的终端获得最佳体验。

## 操作方式

所有界面统一使用同一套按键：

| 按键 | 功能 |
| --- | --- |
| `↑ ↓ ← →` 或 `K J H L` | 移动 / 选择 |
| `空格` | 确认 / 发球 / 直落 / 落子 / 射击 |
| `回车` | 确认 / 落子 |
| `ESC` 或 `Q` | 返回 / 取消 / 暂停 |
| `C` | 暂停菜单中继续 |
| `R` | 结算界面重新开始 |
| `Ctrl+C` | 随时强制退出 |

游戏中按 `ESC` 或 `Q` 会弹出暂停菜单。

## 五款游戏

| # | 游戏 | 玩法 | 计分 |
| --- | --- | --- | --- |
| 1 | 贪吃蛇 | 方向键控制移动，吃食物变长，撞墙/撞自己结束 | 每食物 +10 |
| 2 | 打砖块 | `←→` 移动挡板，`空格` 发球，打碎全部砖块；3 条命 | 每砖 +10，通关 +100 |
| 3 | 俄罗斯方块 | `←→` 移动，`↑` 旋转，`↓` 加速，`空格` 直落 | 消行 100/300/500/800 × 等级 |
| 4 | 飞机大战 | 方向键移动，按住 `空格` 射击，3 条命 | 普通 10 / 快速 15 / 重型 30 |
| 5 | 五子棋 | 方向键移动光标，`回车/空格` 落子，人机对战 | 统计**胜 / 负次数** |

## 高分与用户

- 首次运行会提示设置用户名（默认取系统用户名，可在主菜单第 6 项修改）。
- 数据保存在 **`~/.classic-games-collection/scores.json`**（Windows 为 `%USERPROFILE%\.classic-games-collection\scores.json`）：
  - 前四款游戏：每款保存前 10 名（用户名、分数、时间）
  - 五子棋：按用户名累计胜 / 负次数
- 每局结束后显示结算界面（得分、是否刷新纪录、排行榜），可按 `R` 重玩或返回菜单。

## 项目结构

```
src/
├── main.rs         # 入口
├── app.rs          # 引擎：终端初始化、游戏主循环、暂停/结算/用户名界面
├── canvas.rs       # 帧缓冲渲染器（差量刷新，XTerm256 色，CJK 宽度处理）
├── input.rs        # 按键归一化（方向键 + HJKL / 空格 / 回车 / ESC+Q）
├── score.rs        # 分数与用户名持久化（家目录 JSON）
├── menu.rs         # 主菜单（五合一选择屏）
└── games/
    ├── mod.rs      # Game trait / GameId
    ├── snake.rs    # 贪吃蛇
    ├── breakout.rs # 打砖块
    ├── tetris.rs   # 俄罗斯方块
    ├── plane.rs    # 飞机大战
    └── gomoku.rs   # 五子棋（含启发式 AI 与胜败统计）
```

## 开发辅助脚本

- `smoke_test.py` — 伪终端自动化冒烟测试：驱动真实按键走完五款游戏的主流程。
- `screen_capture.py` — 把各游戏画面重放成文本网格，便于检查布局（`python3 screen_capture.py menu`）。

## 测试

```bash
cargo test
```

## 依赖

| crate | 用途 |
| --- | --- |
| crossterm | 跨平台终端控制（原始模式、事件、颜色） |
| serde / serde_json | 分数文件序列化 |
| rand | 随机数 |
| dirs | 定位家目录 |

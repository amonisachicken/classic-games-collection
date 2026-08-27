# 经典游戏合集 / Classic Games Collection

[![GitHub](https://img.shields.io/badge/GitHub-amonisachicken%2Fclassic--games--collection-blue)](https://github.com/amonisachicken/classic-games-collection)

**选择语言 / Choose language：**

[中文](#zh) ｜ [English](#en)

---

<a id="zh"></a>

# 中文

一个用 Rust 编写的终端经典游戏合集 CLI 应用，包含 **贪吃蛇 / 打砖块 / 俄罗斯方块 / 飞机大战 / 五子棋** 五款游戏。

- 跨平台：支持 **Linux / macOS / Windows**（基于 [crossterm](https://github.com/crossterm-rs/crossterm)）
- 纯终端运行，兼容 **XTerm256 色**；中英双语界面（自动读取 `$LANG`，含 `zh` 显示中文，否则英文）
- 主菜单选择游戏（五合一），每款游戏独立计分
- 用户名与分数记录保存在**家目录**

## 运行

```bash
cargo run --release
```

要求：Rust 1.70+（在任意终端中运行，建议终端至少 80×24）。

> 若终端尺寸过小，界面会自动裁剪居中，不会报错。

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
| 2 | 打砖块 | `←→` 移动挡板，`空格` 发球，打碎全部砖块；3 条命；击碎砖块 1/20 概率掉 `♥` 医疗包，挡板接住回血 | 每砖 +10，通关 +100 |
| 3 | 俄罗斯方块 | `←→` 移动，`↑` 旋转，`↓` 加速，`空格` 直落 | 消行 100/300/500/800 × 等级 |
| 4 | 飞机大战 | 方向键移动，按住 `空格` 射击，3 条命；吃 `♥` 医疗包回血（出现率约为敌机的 1/20） | 普通 10 / 快速 15 / 重型 30 |
| 5 | 五子棋 | 方向键移动光标，`回车/空格` 落子，人机对战 | 统计**胜 / 负次数** |

> 贪吃蛇、打砖块、俄罗斯方块的方块用 2 个半角字符表示（长宽接近正方形），旋转后形状不变形。

## 高分与用户

- 首次运行会提示设置用户名（默认取系统用户名，可在主菜单第 6 项修改）。
- 数据保存在 **`~/.classic-games-collection/scores.json`**（Windows 为 `%USERPROFILE%\.classic-games-collection\scores.json`）：
  - 前四款游戏：每款保存前 10 名（用户名、分数、时间）
  - 五子棋：按用户名累计胜 / 负次数
- 每局结束后显示结算界面（得分、是否刷新纪录、排行榜），可按 `R` 重玩或返回菜单。

## 发布与下载

打 `v*` 标签即可触发 CI 构建并发布各平台二进制到 [GitHub Releases](https://github.com/amonisachicken/classic-games-collection/releases)：

```bash
git tag v1.0.0
git push origin v1.0.0
```

- Linux (x86_64 / aarch64)、macOS (Intel / Apple Silicon)、Windows (x86_64)
- 附 SHA256SUMS.txt 校验和；macOS 首次运行需在“系统设置 → 隐私与安全性”允许（未签名应用），Windows 可能提示 SmartScreen

## 项目结构

```
src/
├── main.rs         # 入口
├── lang.rs         # 中英双语文案表（读取 $LANG）
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

- `smoke_test.py` — 伪终端自动化冒烟测试（含中英文两种模式）。
- `screen_capture.py` — 把各游戏画面重放成文本网格，便于检查布局（`CAP_LANG=en_US.UTF-8` 可截英文界面）。

## 测试

```bash
cargo test
```

## 依赖

| crate | 用途 |
| --- | --- |
| crossterm | 跨平台终端控制 |
| serde / serde_json | 分数文件序列化 |
| rand | 随机数 |
| dirs | 定位家目录 |

---

<a id="en"></a>

# English

A terminal classic-game collection CLI written in Rust, featuring **Snake / Breakout / Tetris / Plane Battle / Gomoku** — five games in one.

- Cross-platform: **Linux / macOS / Windows** (powered by [crossterm](https://github.com/crossterm-rs/crossterm))
- Pure terminal, **XTerm256-color** compatible; bilingual UI (reads `$LANG`: contains `zh` → Chinese, otherwise English)
- Pick a game from the main menu; each game keeps its own high scores
- Username & scores are stored in your **home directory**

## Running

```bash
cargo run --release
```

Requirements: Rust 1.70+ (run in a real terminal, at least 80×24 recommended).

> If the terminal is too small, the UI auto-fits and centers; it never crashes.

## Controls

All screens share the same keys:

| Key | Function |
| --- | --- |
| `↑ ↓ ← →` or `K J H L` | Move / Select |
| `Space` | Confirm / Launch / Hard drop / Place / Shoot |
| `Enter` | Confirm / Place stone |
| `ESC` or `Q` | Back / Cancel / Pause |
| `C` | Resume (in pause menu) |
| `R` | Restart (on summary screen) |
| `Ctrl+C` | Force quit anytime |

Press `ESC` or `Q` in-game to open the pause menu.

## Five Games

| # | Game | Gameplay | Scoring |
| --- | --- | --- | --- |
| 1 | Snake | Move to eat, grow; walls/self kill | +10 per food |
| 2 | Breakout | Move paddle, launch, smash bricks; 3 lives; bricks have a 1/20 chance to drop a `♥` health pack — catch it with the paddle to heal | +10/brick, +100 win |
| 3 | Tetris | Move, rotate, soft drop, hard drop | lines × level |
| 4 | Plane Battle | Move, shoot, 3 lives; grab `♥` to heal (~1/20 of enemies) | normal 10 / fast 15 / heavy 30 |
| 5 | Gomoku | Move cursor, place stones, play vs AI | tracks **wins / losses** |

> Snake, Breakout and Tetris render each cell as 2 half-width chars for a square aspect ratio, so rotated shapes keep their proportions.

## High Scores & User

- On first run you'll be asked for a username (defaults to your system username; change it via menu item 6).
- Data is stored at **`~/.classic-games-collection/scores.json`** (Windows: `%USERPROFILE%\.classic-games-collection\scores.json`):
  - The first four games keep a top-10 list (username, score, time).
  - Gomoku tallies wins/losses per username.
- After each round a summary shows score / new record / leaderboard; press `R` to replay or return to the menu.

## Releases

Tag a `v*` tag to trigger CI and publish binaries for all platforms to [GitHub Releases](https://github.com/amonisachicken/classic-games-collection/releases):

```bash
git tag v1.0.0
git push origin v1.0.0
```

- Linux (x86_64 / aarch64), macOS (Intel / Apple Silicon), Windows (x86_64)
- SHA256SUMS.txt checksums included; macOS: allow via System Settings → Privacy & Security (unsigned app); Windows: SmartScreen may prompt

## Project Structure

```
src/
├── main.rs         # Entry point
├── lang.rs         # i18n strings (reads $LANG)
├── app.rs          # Engine: init, loop, pause/summary/username
├── canvas.rs       # Frame-buffer renderer (diff-based, XTerm256, CJK width)
├── input.rs        # Key mapping
├── score.rs        # Score persistence (home-dir JSON)
├── menu.rs         # Main menu
└── games/
    ├── mod.rs      # Game trait / GameId
    ├── snake.rs    # Snake
    ├── breakout.rs # Breakout
    ├── tetris.rs   # Tetris
    ├── plane.rs    # Plane Battle
    └── gomoku.rs   # Gomoku (heuristic AI + W/L stats)
```

## Dev Scripts

- `smoke_test.py` — PTY smoke test (Chinese & English modes).
- `screen_capture.py` — Replay screens as text grids (`CAP_LANG=en_US.UTF-8` for English).

## Tests

```bash
cargo test
```

## Dependencies

| crate | Purpose |
| --- | --- |
| crossterm | Cross-platform terminal control |
| serde / serde_json | Score file serialization |
| rand | Random numbers |
| dirs | Home directory lookup |

# 经典游戏合集 / Classic Games Collection

[![GitHub](https://img.shields.io/badge/GitHub-amonisachicken%2Fclassic--games--collection-blue)](https://github.com/amonisachicken/classic-games-collection)

一个用 Rust 编写的终端经典游戏合集 CLI 应用，包含 **贪吃蛇 / 打砖块 / 俄罗斯方块 / 飞机大战 / 五子棋** 五款游戏。

A terminal classic-game collection CLI written in Rust, featuring **Snake / Breakout / Tetris / Plane Battle / Gomoku** — five games in one.

- 跨平台：支持 **Linux / macOS / Windows**（基于 [crossterm](https://github.com/crossterm-rs/crossterm)）
- Cross-platform: **Linux / macOS / Windows** (powered by [crossterm](https://github.com/crossterm-rs/crossterm))
- 纯终端运行，兼容 **XTerm256 色**；中英双语界面（自动读取 `$LANG`，含 `zh` 显示中文，否则英文）
- Pure terminal, **XTerm256-color** compatible; bilingual UI (reads `$LANG`: contains `zh` → Chinese, otherwise English)
- 主菜单选择游戏（五合一），每款游戏独立计分
- Pick a game from the main menu; each game keeps its own high scores
- 用户名与分数记录保存在**家目录**
- Username & scores are stored in your **home directory**

## 运行 / Running

```bash
cargo run --release
```

要求：Rust 1.70+（在任意终端中运行，建议终端至少 80×24）。
Requirements: Rust 1.70+ (run in a real terminal, at least 80×24 recommended).

> 若终端尺寸过小，界面会自动裁剪居中，不会报错。
> If the terminal is too small, the UI auto-fits and centers; it never crashes.

## 操作方式 / Controls

所有界面统一使用同一套按键。All screens share the same keys:

| 按键 / Key | 功能 / Function |
| --- | --- |
| `↑ ↓ ← →` 或 `K J H L` | 移动 / 选择 · Move / Select |
| `空格` / `Space` | 确认 / 发球 / 直落 / 落子 / 射击 · Confirm / Launch / Hard drop / Place / Shoot |
| `回车` / `Enter` | 确认 / 落子 · Confirm / Place stone |
| `ESC` 或 `Q` | 返回 / 取消 / 暂停 · Back / Cancel / Pause |
| `C` | 暂停菜单中继续 · Resume (in pause menu) |
| `R` | 结算界面重新开始 · Restart (on summary screen) |
| `Ctrl+C` | 随时强制退出 · Force quit anytime |

游戏中按 `ESC` 或 `Q` 会弹出暂停菜单。Press `ESC` or `Q` in-game to open the pause menu.

## 五款游戏 / Five Games

| # | 游戏 / Game | 玩法 / Gameplay | 计分 / Scoring |
| --- | --- | --- | --- |
| 1 | 贪吃蛇 / Snake | 方向键控制移动，吃食物变长，撞墙/撞自己结束 · Move to eat, grow; walls/self kill | 每食物 +10 · +10 per food |
| 2 | 打砖块 / Breakout | `←→` 移动挡板，`空格` 发球，打碎全部砖块；3 条命 · Move paddle, launch, smash bricks; 3 lives | 每砖 +10，通关 +100 · +10/brick, +100 win |
| 3 | 俄罗斯方块 / Tetris | `←→` 移动，`↑` 旋转，`↓` 加速，`空格` 直落 · Move, rotate, soft drop, hard drop | 消行 100/300/500/800 × 等级 · lines × level |
| 4 | 飞机大战 / Plane Battle | 方向键移动，按住 `空格` 射击，3 条命；吃 `♥` 医疗包回血（出现率约为敌机的 1/20）· Move, shoot, 3 lives; grab `♥` to heal (~1/20 of enemies) | 普通 10 / 快速 15 / 重型 30 · normal 10 / fast 15 / heavy 30 |
| 5 | 五子棋 / Gomoku | 方向键移动光标，`回车/空格` 落子，人机对战 · Move cursor, place stones, play vs AI | 统计**胜 / 负次数** · tracks **wins / losses** |

## 高分与用户 / High Scores & User

- 首次运行会提示设置用户名（默认取系统用户名，可在主菜单第 6 项修改）。
  On first run you'll be asked for a username (defaults to your system username; change it via menu item 6).
- 数据保存在 **`~/.classic-games-collection/scores.json`**（Windows 为 `%USERPROFILE%\.classic-games-collection\scores.json`）：
  Data is stored at **`~/.classic-games-collection/scores.json`** (Windows: `%USERPROFILE%\.classic-games-collection\scores.json`):
  - 前四款游戏：每款保存前 10 名（用户名、分数、时间）
    The first four games keep a top-10 list (username, score, time).
  - 五子棋：按用户名累计胜 / 负次数
    Gomoku tallies wins/losses per username.
- 每局结束后显示结算界面（得分、是否刷新纪录、排行榜），可按 `R` 重玩或返回菜单。
  After each round a summary shows score / new record / leaderboard; press `R` to replay or return to the menu.

## 项目结构 / Project Structure

```
src/
├── main.rs         # 入口 · Entry point
├── lang.rs         # 中英双语文案表 · i18n strings (reads $LANG)
├── app.rs          # 引擎：终端初始化、游戏主循环、暂停/结算/用户名界面 · Engine: init, loop, pause/summary/username
├── canvas.rs       # 帧缓冲渲染器（差量刷新，XTerm256 色，CJK 宽度处理）· Frame-buffer renderer (diff-based, XTerm256, CJK width)
├── input.rs        # 按键归一化（方向键 + HJKL / 空格 / 回车 / ESC+Q）· Key mapping
├── score.rs        # 分数与用户名持久化（家目录 JSON）· Score persistence (home-dir JSON)
├── menu.rs         # 主菜单（五合一选择屏）· Main menu
└── games/
    ├── mod.rs      # Game trait / GameId
    ├── snake.rs    # 贪吃蛇 · Snake
    ├── breakout.rs # 打砖块 · Breakout
    ├── tetris.rs   # 俄罗斯方块 · Tetris
    ├── plane.rs    # 飞机大战 · Plane Battle
    └── gomoku.rs   # 五子棋（含启发式 AI 与胜败统计）· Gomoku (heuristic AI + W/L stats)
```

## 开发辅助脚本 / Dev Scripts

- `smoke_test.py` — 伪终端自动化冒烟测试（含中英文两种模式）。PTY smoke test (Chinese & English modes).
- `screen_capture.py` — 把各游戏画面重放成文本网格，便于检查布局（`CAP_LANG=en_US.UTF-8` 可截英文界面）。Replay screens as text grids (`CAP_LANG=en_US.UTF-8` for English).

## 测试 / Tests

```bash
cargo test
```

## 依赖 / Dependencies

| crate | 用途 / Purpose |
| --- | --- |
| crossterm | 跨平台终端控制 · Cross-platform terminal control |
| serde / serde_json | 分数文件序列化 · Score file serialization |
| rand | 随机数 · Random numbers |
| dirs | 定位家目录 · Home directory lookup |

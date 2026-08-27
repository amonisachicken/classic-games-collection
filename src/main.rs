//! 经典游戏合集：贪吃蛇 / 打砖块 / 俄罗斯方块 / 飞机大战 / 五子棋。
//!
//! 跨 Linux / macOS / Windows 终端运行，兼容 XTerm256 色。
//! 操作：方向键 + HJKL，空格，回车，ESC/Q。
//! 用户名与分数记录保存在家目录 `~/.classic-games-collection/scores.json`。

mod app;
mod canvas;
mod games;
mod input;
mod lang;
mod menu;
mod score;

use std::io;

fn main() {
    // 读取 $LANG 确定界面语言
    lang::init();

    // 先加载分数文件再进入原始模式，避免意外崩溃丢数据
    let mut scores = score::ScoreFile::load();

    // 终端守卫：进入原始模式 + 备用屏幕；退出时自动恢复
    let guard = match app::TermGuard::new() {
        Ok(g) => g,
        Err(_) => {
            eprintln!("{}", lang::ui().err_no_term);
            std::process::exit(1);
        }
    };

    let mut eng = match app::Engine::new(&mut scores) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{}", lang::ui().err_init_term.replace("{}", &e.to_string()));
            std::process::exit(1);
        }
    };

    app::run(&mut eng);

    // 释放引擎对 scores 的借用后，显式保存收尾
    drop(eng);
    let _ = scores.save();
    drop(guard);
    let _ = io::Write::flush(&mut io::stdout());
}

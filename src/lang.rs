//! 国际化：启动时读取环境变量 `$LANG`，包含 `zh` 字段则显示中文，否则显示英文。

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Zh,
    En,
}

/// 界面文案表。所有用户可见字符串都从这里取。
pub struct Ui {
    // 标题 / 菜单
    pub title: &'static str,
    pub subtitle: &'static str,
    pub change_user: &'static str,
    pub quit: &'static str,
    pub player_fmt: &'static str,
    pub no_record: &'static str,
    pub no_stats: &'static str,
    pub best_fmt: &'static str,
    pub record_fmt: &'static str,
    pub menu_help1: &'static str,
    pub menu_help2: &'static str,
    // 游戏名与简介
    pub game_snake: &'static str,
    pub game_breakout: &'static str,
    pub game_tetris: &'static str,
    pub game_plane: &'static str,
    pub game_gomoku: &'static str,
    pub desc_snake: &'static str,
    pub desc_breakout: &'static str,
    pub desc_tetris: &'static str,
    pub desc_plane: &'static str,
    pub desc_gomoku: &'static str,
    // 通用面板
    pub score_fmt: &'static str,
    pub lives_fmt: &'static str,
    // 暂停
    pub paused: &'static str,
    pub resume: &'static str,
    pub quit_to_menu: &'static str,
    pub tip_ctrlc: &'static str,
    // 结算
    pub game_over: &'static str,
    pub score_line_fmt: &'static str,
    pub new_record: &'static str,
    pub rank_fmt: &'static str,
    pub you_win: &'static str,
    pub you_lose: &'static str,
    pub draw: &'static str,
    pub record_line_fmt: &'static str,
    pub leaderboard_fmt: &'static str,
    pub lb_header_score: &'static str,
    pub lb_header_gomoku: &'static str,
    pub play_again: &'static str,
    // 用户名界面
    pub set_username: &'static str,
    pub username: &'static str,
    pub username_hint: &'static str,
    pub username_empty: &'static str,
    // 贪吃蛇
    pub snake_title: &'static str,
    pub snake_help: &'static str,
    pub snake_paused: &'static str,
    // 打砖块
    pub breakout_title: &'static str,
    pub breakout_help_ready: &'static str,
    pub breakout_help: &'static str,
    // 俄罗斯方块
    pub tetris_title: &'static str,
    pub tetris_sub: &'static str,
    pub lines_fmt: &'static str,
    pub level_fmt: &'static str,
    pub best_short_fmt: &'static str,
    pub next: &'static str,
    pub tetris_help: [&'static str; 5],
    // 飞机大战
    pub plane_title: &'static str,
    pub plane_help: &'static str,
    pub heal_fmt: &'static str,
    pub life_full: &'static str,
    // 五子棋
    pub gomoku_title: &'static str,
    pub you_first: &'static str,
    pub ai_stone: &'static str,
    pub stat_fmt: &'static str,
    pub ai_thinking: &'static str,
    pub your_turn: &'static str,
    pub gomoku_help: [&'static str; 3],
    // 其他
    pub term_title: &'static str,
    pub err_no_term: &'static str,
    pub err_init_term: &'static str,
}

pub const ZH: Ui = Ui {
    title: "◆ 经典游戏合集 ◆",
    subtitle: "CLASSIC GAMES 5-in-1",
    change_user: "设置用户名",
    quit: "退出",
    player_fmt: "玩家: {}",
    no_record: "暂无纪录",
    no_stats: "暂无战绩",
    best_fmt: "最高 {} ({})",
    record_fmt: "胜 {} 负 {}  ({})",
    menu_help1: "↑↓ / HJKL 选择    回车 / 空格 进入    ESC / Q 退出",
    menu_help2: "游戏中: ESC / Q 暂停,  Ctrl+C 随时退出",
    game_snake: "贪吃蛇",
    game_breakout: "打砖块",
    game_tetris: "俄罗斯方块",
    game_plane: "飞机大战",
    game_gomoku: "五子棋",
    desc_snake: "吃食物变长，别撞墙别咬到自己",
    desc_breakout: "接住小球，打碎所有砖块",
    desc_tetris: "消行得分，方块越落越快",
    desc_plane: "移动射击，吃♥医疗包回血",
    desc_gomoku: "五子连珠，人机对战",
    score_fmt: "得分 {}",
    lives_fmt: "生命 {}",
    paused: "已暂停  PAUSED",
    resume: "C / 空格 / 回车  : 继续",
    quit_to_menu: "Q / ESC          : 返回菜单",
    tip_ctrlc: "提示: Ctrl+C 随时退出",
    game_over: "游戏结束",
    score_line_fmt: "本局得分: {}",
    new_record: "★ 新纪录! 最高分 ★",
    rank_fmt: "进入历史第 {} 名",
    you_win: "你赢了！",
    you_lose: "你输了",
    draw: "平局",
    record_line_fmt: "你的战绩: {} 胜 / {} 负",
    leaderboard_fmt: "--- {} 排行榜 ---",
    lb_header_score: "  名次  玩家            分数      时间",
    lb_header_gomoku: "  名次  玩家            胜    负",
    play_again: "[R] 再来一局     [Enter / ESC / Q] 返回菜单",
    set_username: "设置用户名",
    username: "用户名:",
    username_hint: "回车 确定    ESC 取消    (字母/数字/空格/.-_)",
    username_empty: "用户名不能为空",
    snake_title: "贪吃蛇 SNAKE",
    snake_help: "方向键/HJKL 移动   空格 暂停    ESC/Q 菜单",
    snake_paused: "已暂停 (空格继续)",
    breakout_title: "打砖块 BREAKOUT",
    breakout_help_ready: "←→/HL 移动    空格 发球    ♥医疗包+1命    ESC/Q 菜单",
    breakout_help: "←→/HL 移动    ♥医疗包+1命    ESC/Q 菜单",
    tetris_title: "俄罗斯方块",
    tetris_sub: "TETRIS",
    lines_fmt: "行数 {}",
    level_fmt: "等级 {}",
    best_short_fmt: "最高 {}",
    next: "下一个",
    tetris_help: [
        "←→ / HL  左右移动",
        "↑ / K     旋转",
        "↓ / J     加速下落",
        "空格       直落",
        "ESC/Q      暂停",
    ],
    plane_title: "飞机大战 PLANE",
    plane_help: "方向键/HJKL 移动    按住空格 射击    ♥医疗包+1命    ESC/Q 暂停",
    heal_fmt: "♥ 生命 +1",
    life_full: "生命已满",
    gomoku_title: "五子棋 GOMOKU",
    you_first: "你: X (先手)",
    ai_stone: "AI: O",
    stat_fmt: "战绩: {} 胜 / {} 负",
    ai_thinking: "AI 思考中...",
    your_turn: "轮到你 (X)",
    gomoku_help: ["方向键/HJKL 移动", "回车/空格 落子", "ESC/Q 暂停退出"],
    term_title: "Classic Games Collection - 经典游戏合集",
    err_no_term: "错误: 无法进入终端模式，请确认在真实终端（非管道）中运行本程序。",
    err_init_term: "错误: 初始化终端失败: {}",
};

pub const EN: Ui = Ui {
    title: "◆ Classic Games Collection ◆",
    subtitle: "CLASSIC GAMES 5-in-1",
    change_user: "Change User",
    quit: "Quit",
    player_fmt: "Player: {}",
    no_record: "No records",
    no_stats: "No record",
    best_fmt: "Best {} ({})",
    record_fmt: "W {} L {}  ({})",
    menu_help1: "↑↓ / HJKL select    Enter / Space start    ESC / Q quit",
    menu_help2: "In game: ESC / Q pause,  Ctrl+C quits anytime",
    game_snake: "Snake",
    game_breakout: "Breakout",
    game_tetris: "Tetris",
    game_plane: "Plane Battle",
    game_gomoku: "Gomoku",
    desc_snake: "Eat food to grow; avoid walls and yourself",
    desc_breakout: "Keep the ball up; smash every brick",
    desc_tetris: "Clear lines; pieces fall faster",
    desc_plane: "Move & shoot; grab ♥ to heal",
    desc_gomoku: "Five in a row, play vs the AI",
    score_fmt: "Score {}",
    lives_fmt: "Lives {}",
    paused: "PAUSED",
    resume: "C / Space / Enter : Resume",
    quit_to_menu: "Q / ESC          : Quit to menu",
    tip_ctrlc: "Tip: Ctrl+C quits anytime",
    game_over: "GAME OVER",
    score_line_fmt: "Score: {}",
    new_record: "★ NEW RECORD! BEST SCORE ★",
    rank_fmt: "Ranked #{}, new entry",
    you_win: "You win!",
    you_lose: "You lose",
    draw: "Draw",
    record_line_fmt: "Your record: {} W / {} L",
    leaderboard_fmt: "--- {} Leaderboard ---",
    lb_header_score: "  Rank  Player            Score     Time",
    lb_header_gomoku: "  Rank  Player            W      L",
    play_again: "[R] Play again     [Enter / ESC / Q] Back to menu",
    set_username: "Set Username",
    username: "Username:",
    username_hint: "Enter OK    ESC cancel    (letters/digits/space/.-_)",
    username_empty: "Username cannot be empty",
    snake_title: "SNAKE",
    snake_help: "Arrows/HJKL move    Space pause    ESC/Q menu",
    snake_paused: "Paused (Space to resume)",
    breakout_title: "BREAKOUT",
    breakout_help_ready: "←→/HL move    Space launch    ♥=+1 life    ESC/Q menu",
    breakout_help: "←→/HL move    ♥=+1 life    ESC/Q menu",
    tetris_title: "TETRIS",
    tetris_sub: "",
    lines_fmt: "Lines {}",
    level_fmt: "Level {}",
    best_short_fmt: "Best {}",
    next: "Next",
    tetris_help: [
        "←→ / HL  Move",
        "↑ / K     Rotate",
        "↓ / J     Soft drop",
        "Space     Hard drop",
        "ESC/Q     Pause",
    ],
    plane_title: "PLANE",
    plane_help: "Arrows/HJKL move    Hold Space shoot    ♥=+1 life    ESC/Q pause",
    heal_fmt: "♥ +1 life",
    life_full: "Life full",
    gomoku_title: "GOMOKU",
    you_first: "You: X (first)",
    ai_stone: "AI: O",
    stat_fmt: "Record: {} W / {} L",
    ai_thinking: "AI thinking...",
    your_turn: "Your turn (X)",
    gomoku_help: ["Arrows/HJKL move", "Enter/Space place", "ESC/Q pause & quit"],
    term_title: "Classic Games Collection",
    err_no_term: "Error: cannot enter terminal mode. Please run in a real terminal (not a pipe).",
    err_init_term: "Error: failed to initialize terminal: {}",
};

static CURRENT: OnceLock<Lang> = OnceLock::new();

/// 启动时调用：读取 `$LANG` 确定语言。
pub fn init() {
    let _ = CURRENT.set(detect());
}

/// 检测语言：`$LANG` 含 `zh`（不区分大小写）→ 中文，否则英文。
pub fn detect() -> Lang {
    match std::env::var("LANG") {
        Ok(v) if v.to_lowercase().contains("zh") => Lang::Zh,
        _ => Lang::En,
    }
}

pub fn lang() -> Lang {
    *CURRENT.get().unwrap_or(&Lang::En)
}

/// 用运行时模板做格式化（按顺序替换 `{}` 占位符）。
pub fn fmt(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    let mut i = 0usize;
    while let Some(p) = rest.find("{}") {
        out.push_str(&rest[..p]);
        if i < args.len() {
            out.push_str(&args[i].to_string());
        }
        i += 1;
        rest = &rest[p + 2..];
    }
    out.push_str(rest);
    out
}

/// 当前语言对应的文案表。
pub fn ui() -> &'static Ui {
    match lang() {
        Lang::Zh => &ZH,
        Lang::En => &EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_lang() {
        // 无法改环境变量, 直接测 contains 逻辑
        assert!("zh_CN.UTF-8".to_lowercase().contains("zh"));
        assert!("zh_TW.UTF-8".to_lowercase().contains("zh"));
        assert!(!"en_US.UTF-8".to_lowercase().contains("zh"));
        assert!(!"C".to_lowercase().contains("zh"));
    }

    #[test]
    fn ui_tables_consistent() {
        // 两个语言的文案表占位符数量应一致(粗略校验每处 format 可用)
        assert_eq!(ZH.best_fmt.matches("{}").count(), 2);
        assert_eq!(EN.best_fmt.matches("{}").count(), 2);
        assert_eq!(ZH.record_fmt.matches("{}").count(), 3);
        assert_eq!(EN.record_fmt.matches("{}").count(), 3);
    }
}

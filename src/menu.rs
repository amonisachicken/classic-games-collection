//! 主菜单：五合一游戏选择屏。

use crate::app::Engine;
use crate::canvas::{col, str_width};
use crate::games::GameId;
use crate::input::Action;
use crate::lang::{self, Lang};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuChoice {
    Play(GameId),
    ChangeUser,
    Quit,
}

/// 菜单项总数：5 款游戏 + 设置用户名 + 退出。
const ITEM_COUNT: usize = 7;

/// 主菜单循环。
pub fn show(eng: &mut Engine) -> MenuChoice {
    let mut sel = 0usize;
    loop {
        if eng.quit_requested {
            return MenuChoice::Quit;
        }
        draw(eng, sel);
        let _ = eng.canvas.flush(&mut std::io::stdout());
        match eng.poll_action(Duration::from_millis(60)) {
            Some(Action::Up) => sel = sel.saturating_sub(1),
            Some(Action::Down) => sel = (sel + 1) % ITEM_COUNT,
            Some(Action::Confirm) | Some(Action::Space) => return choose(sel),
            Some(Action::Cancel) => return MenuChoice::Quit,
            _ => {}
        }
    }
}

fn choose(sel: usize) -> MenuChoice {
    match sel {
        0..=4 => MenuChoice::Play(GameId::ALL[sel]),
        5 => MenuChoice::ChangeUser,
        _ => MenuChoice::Quit,
    }
}

fn item_line(id: GameId, scores: &crate::score::ScoreFile) -> String {
    match id {
        GameId::Gomoku => {
            let mut best: Option<(String, u32)> = None;
            let mut best_loss = 0u32;
            for (u, s) in &scores.gomoku {
                let better = match &best {
                    Some((_, w)) => s.wins > *w,
                    None => true,
                };
                if better {
                    best = Some((u.clone(), s.wins));
                    best_loss = s.losses;
                }
            }
            match best {
                Some((u, w)) => lang::fmt(lang::ui().record_fmt, &[&w, &best_loss, &u]),
                None => lang::ui().no_stats.to_string(),
            }
        }
        _ => match scores.get_vec(id.score_key()).first() {
            Some(e) => lang::fmt(lang::ui().best_fmt, &[&e.score, &e.user]),
            None => lang::ui().no_record.to_string(),
        },
    }
}

fn draw(eng: &mut Engine, sel: usize) {
    let c = &mut eng.canvas;
    c.clear();
    c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
    let cx = c.w / 2;
    let mut row = 1usize;

    // 标题
    let t1 = lang::ui().title;
    c.put_str(cx - str_width(t1) / 2, row, t1, col::YELLOW, col::BLACK);
    row += 1;
    let t2 = lang::ui().subtitle;
    c.put_str(cx - str_width(t2) / 2, row, t2, col::CYAN, col::BLACK);
    row += 2;

    // 游戏列表
    for (i, id) in GameId::ALL.iter().enumerate() {
        let selected = i == sel;
        let num = format!("{}.", i + 1);
        let name = id.display_name();
        let best = item_line(*id, eng.scores);
        // 计算行宽
        let pad = 2usize;
        let line_w = str_width(&num) + 1 + str_width(&name) + pad + str_width(&best) + 4;
        let x = cx.saturating_sub(line_w / 2);
        let bg = if selected { col::DARK_BLUE } else { col::BLACK };
        // 高亮整行
        c.fill_rect(x, row, line_w + 4, 1, ' ', col::BLACK, bg);
        if selected {
            c.put_str(x, row, "▶", col::YELLOW, bg);
        }
        let mut col_x = x + 2;
        c.put_str(col_x, row, &num, if selected { col::YELLOW } else { col::GRAY }, bg);
        col_x += str_width(&num) + 1;
        c.put_str(col_x, row, &name, if selected { col::WHITE } else { col::GREEN }, bg);
        col_x += str_width(&name) + pad;
        c.put_str(col_x, row, &best, col::GRAY, bg);
        row += 1;
        // 简介
        let desc = format!("      {}", id.desc());
        if selected {
            c.put_str(cx - str_width(&desc) / 2, row, &desc, col::GRAY, bg);
        }
        row += 1;
    }

    // 设置用户名 / 退出
    let setting_labels: [(&str, &str); 2] =
        [(lang::ui().change_user, "Change User"), (lang::ui().quit, "Quit")];
    for (i, (label, label_en)) in setting_labels.iter().enumerate() {
        let idx = 5 + i;
        let selected = idx == sel;
        let bg = if selected { col::DARK_BLUE } else { col::BLACK };
        let line = if lang::lang() == Lang::Zh {
            format!("{}. {} ({})", idx + 1, label, label_en)
        } else {
            format!("{}. {}", idx + 1, label_en)
        };
        let x = cx.saturating_sub(str_width(&line) / 2);
        c.fill_rect(x, row, str_width(&line) + 4, 1, ' ', col::BLACK, bg);
        if selected {
            c.put_str(x, row, "▶", col::YELLOW, bg);
        }
        c.put_str(
            x + 2,
            row,
            &line,
            if selected { col::WHITE } else { col::GRAY },
            bg,
        );
        row += 1;
    }

    let user = lang::fmt(lang::ui().player_fmt, &[&eng.user]);
    c.put_str(cx - str_width(&user) / 2, row, &user, col::CYAN, col::BLACK);

    // 底部操作说明
    let help1 = lang::ui().menu_help1;
    let help2 = lang::ui().menu_help2;
    let hy = c.h.saturating_sub(3);
    c.put_str(cx - str_width(help1) / 2, hy, help1, col::GRAY, col::BLACK);
    c.put_str(cx - str_width(help2) / 2, hy + 1, help2, col::GRAY, col::BLACK);
}

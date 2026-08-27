//! 应用引擎：终端初始化、通用游戏循环、暂停菜单、结算界面、用户名输入。

use crate::canvas::{col, str_width, Canvas};
use crate::games::{Game, GameId, GameOutcome, Status};
use crate::input::Action;
use crate::menu::{self, MenuChoice};
use crate::score::{self, ScoreFile};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use std::io;
use std::time::{Duration, Instant};

/// 进入/退出原始模式与备用屏幕的 RAII 守卫。
pub struct TermGuard;

impl TermGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        let _ = execute!(
            out,
            EnterAlternateScreen,
            Hide,
            SetTitle("Classic Games Collection - 经典游戏合集")
        );
        Ok(TermGuard)
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// 游戏引擎：持有画布、分数数据、当前用户。
pub struct Engine<'a> {
    pub canvas: Canvas,
    pub scores: &'a mut ScoreFile,
    /// 当前用户名（与 scores.user 同步）。
    pub user: String,
    pub term_w: usize,
    pub term_h: usize,
    /// Ctrl+C 请求退出。
    pub quit_requested: bool,
    acc: f64,
}

impl<'a> Engine<'a> {
    pub fn new(scores: &'a mut ScoreFile) -> io::Result<Self> {
        let (w, h) = size()?;
        Ok(Engine {
            canvas: Canvas::new(w as usize, h as usize),
            scores,
            user: String::new(),
            term_w: w as usize,
            term_h: h as usize,
            quit_requested: false,
            acc: 0.0,
        })
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.term_w = w as usize;
        self.term_h = h as usize;
        self.canvas.resize(w as usize, h as usize);
    }

    /// 非阻塞读取一个按键动作；处理窗口尺寸变化与 Ctrl+C。
    pub fn poll_action(&mut self, timeout: Duration) -> Option<Action> {
        match event::poll(timeout) {
            Ok(true) => {}
            _ => return None,
        }
        match event::read() {
            Ok(Event::Key(k)) => {
                if (k.code == KeyCode::Char('c') || k.code == KeyCode::Char('C'))
                    && k.modifiers.contains(KeyModifiers::CONTROL)
                {
                    self.quit_requested = true;
                    return Some(Action::Cancel);
                }
                crate::input::map_key(k)
            }
            Ok(Event::Resize(w, h)) => {
                self.resize(w, h);
                None
            }
            _ => None,
        }
    }

    /// 运行一款游戏直到结束或退出，结束时自动记分。
    pub fn play(&mut self, id: GameId, game: &mut dyn Game) -> GameOutcome {
        self.canvas.clear();
        self.canvas.force_redraw();
        let dt_fixed = 1.0 / 30.0;
        let mut last = Instant::now();
        loop {
            if self.quit_requested {
                return GameOutcome::Quit;
            }
            // 清空输入队列
            loop {
                match self.poll_action(Duration::ZERO) {
                    Some(Action::Cancel) => {
                        if self.pause_menu() == PauseChoice::Quit {
                            return GameOutcome::Quit;
                        }
                    }
                    Some(a) => game.handle(a, self),
                    None => break,
                }
                if game.status() == Status::Finished {
                    break;
                }
            }
            if game.status() == Status::Finished {
                let outcome = game.outcome();
                self.record(id, outcome);
                return outcome;
            }
            // 固定步长推进
            let now = Instant::now();
            let dt = (now - last).as_secs_f64().min(0.25);
            last = now;
            self.acc += dt;
            while self.acc >= dt_fixed {
                game.update(dt_fixed, self);
                self.acc -= dt_fixed;
                if game.status() == Status::Finished {
                    break;
                }
            }
            game.draw(&mut self.canvas, self.scores, &self.user);
            let _ = self.canvas.flush(&mut io::stdout());
            // 节流到 ~30fps
            let elapsed = last.elapsed().as_secs_f64();
            let remaining = (dt_fixed - elapsed).max(0.0);
            if remaining > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(remaining));
            }
        }
    }

    /// 结算并写入分数文件。
    fn record(&mut self, id: GameId, outcome: GameOutcome) {
        match outcome {
            GameOutcome::Score(s) => self.scores.add_score(id.score_key(), &self.user, s),
            GameOutcome::Gomoku { win } => self.scores.add_gomoku(&self.user, win),
            GameOutcome::GomokuDraw | GameOutcome::Quit => {}
        }
        let _ = self.scores.save();
    }

    /// 暂停菜单（画布上叠加显示）。
    fn pause_menu(&mut self) -> PauseChoice {
        loop {
            if self.quit_requested {
                return PauseChoice::Quit;
            }
            self.draw_pause();
            let _ = self.canvas.flush(&mut io::stdout());
            match self.poll_action(Duration::from_millis(60)) {
                Some(Action::Confirm)
                | Some(Action::Space)
                | Some(Action::Char('c'))
                | Some(Action::Char('C')) => return PauseChoice::Resume,
                Some(Action::Cancel) => return PauseChoice::Quit,
                _ => {}
            }
        }
    }

    fn draw_pause(&mut self) {
        let c = &mut self.canvas;
        let w = 44usize;
        let h = 8usize;
        let x = c.w.saturating_sub(w) / 2;
        let y = c.h.saturating_sub(h) / 2;
        c.fill_rect(x, y, w, h, ' ', col::BLACK, col::DARK_BLUE);
        c.border(x, y, w, h, col::CYAN);
        let title = "已暂停  PAUSED";
        c.put_str(
            x + (w - str_width(title)) / 2,
            y + 1,
            title,
            col::YELLOW,
            col::DARK_BLUE,
        );
        let l1 = "C / 空格 / 回车  : 继续";
        let l2 = "Q / ESC          : 返回菜单";
        let l3 = "提示: Ctrl+C 随时退出";
        c.put_str(x + 2, y + 3, l1, col::WHITE, col::DARK_BLUE);
        c.put_str(x + 2, y + 4, l2, col::WHITE, col::DARK_BLUE);
        c.put_str(
            x + (w - str_width(l3)) / 2,
            y + 6,
            l3,
            col::GRAY,
            col::DARK_BLUE,
        );
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PauseChoice {
    Resume,
    Quit,
}

/// 顶层流程：菜单 → 游戏 → 结算 → 菜单。
pub fn run(eng: &mut Engine) {
    if eng.scores.user.trim().is_empty() {
        eng.user = score::system_user();
        username_screen(eng);
    }
    eng.user = eng.scores.user.clone();
    loop {
        eng.canvas.force_redraw();
        match menu::show(eng) {
            MenuChoice::Quit => break,
            MenuChoice::ChangeUser => username_screen(eng),
            MenuChoice::Play(id) => {
                loop {
                    let mut game = id.new_game();
                    let outcome = eng.play(id, game.as_mut());
                    if matches!(outcome, GameOutcome::Quit) {
                        break;
                    }
                    match summary_screen(eng, id, outcome) {
                        SummaryChoice::Restart => continue,
                        SummaryChoice::Menu => break,
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SummaryChoice {
    Restart,
    Menu,
}

/// 结算界面：显示得分/胜负、排行榜，等待 R 重玩或返回菜单。
fn summary_screen(eng: &mut Engine, id: GameId, outcome: GameOutcome) -> SummaryChoice {
    eng.canvas.force_redraw();
    loop {
        if eng.quit_requested {
            return SummaryChoice::Menu;
        }
        draw_summary(eng, id, outcome);
        let _ = eng.canvas.flush(&mut io::stdout());
        match eng.poll_action(Duration::from_millis(60)) {
            Some(Action::Char('r')) | Some(Action::Char('R')) => return SummaryChoice::Restart,
            Some(Action::Cancel) | Some(Action::Confirm) | Some(Action::Space) => {
                return SummaryChoice::Menu
            }
            _ => {}
        }
    }
}

fn draw_summary(eng: &mut Engine, id: GameId, outcome: GameOutcome) {
    let c = &mut eng.canvas;
    c.clear();
    c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
    let cx = c.w / 2;

    let head = format!("{}  {}", id.name(), id.name_en());
    c.put_str(cx - str_width(&head) / 2, 1, &head, col::YELLOW, col::BLACK);

    let mut row = 4;
    match outcome {
        GameOutcome::Score(s) => {
            c.put_str(cx - 5, row, "游戏结束", col::WHITE, col::BLACK);
            row += 2;
            let line = format!("本局得分: {}", s);
            c.put_str(cx - str_width(&line) / 2, row, &line, col::GREEN, col::BLACK);
            row += 1;
            if eng.scores.is_new_best(id.score_key(), s) {
                let nb = "★ 新纪录! 最高分 ★";
                c.put_str(cx - str_width(nb) / 2, row, nb, col::RED, col::BLACK);
                row += 1;
            } else if let Some(rk) = eng.scores.rank_of(id.score_key(), s) {
                let rkline = format!("进入历史第 {} 名", rk);
                c.put_str(cx - str_width(&rkline) / 2, row, &rkline, col::CYAN, col::BLACK);
                row += 1;
            }
        }
        GameOutcome::Gomoku { win } => {
            let (txt, fg) = if win {
                ("你赢了！", col::GREEN)
            } else {
                ("你输了", col::RED)
            };
            c.put_str(cx - str_width(txt) / 2, row, txt, fg, col::BLACK);
            row += 2;
            let st = eng.scores.gomoku_stats(&eng.user);
            let line = format!("你的战绩: {} 胜 / {} 负", st.wins, st.losses);
            c.put_str(cx - str_width(&line) / 2, row, &line, col::WHITE, col::BLACK);
            row += 1;
        }
        GameOutcome::GomokuDraw => {
            let txt = "平局";
            c.put_str(cx - 2, row, txt, col::YELLOW, col::BLACK);
            row += 2;
            let st = eng.scores.gomoku_stats(&eng.user);
            let line = format!("你的战绩: {} 胜 / {} 负", st.wins, st.losses);
            c.put_str(cx - str_width(&line) / 2, row, &line, col::WHITE, col::BLACK);
            row += 1;
        }
        GameOutcome::Quit => return,
    }

    row += 1;
    let title = format!("--- {} 排行榜 ---", id.name());
    c.put_str(cx - str_width(&title) / 2, row, &title, col::CYAN, col::BLACK);
    row += 1;

    if id == GameId::Gomoku {
        // 五子棋榜：按胜场排
        let mut all: Vec<(String, crate::score::GomokuStats)> = eng
            .scores
            .gomoku
            .iter()
            .map(|(u, s)| (u.clone(), s.clone()))
            .collect();
        all.sort_by(|a, b| {
            b.1.wins
                .cmp(&a.1.wins)
                .then_with(|| a.1.losses.cmp(&b.1.losses))
        });
        let hdr = "  名次  玩家            胜    负";
        c.put_str(cx - str_width(hdr) / 2, row, hdr, col::GRAY, col::BLACK);
        row += 1;
        for (i, (u, s)) in all.iter().take(6).enumerate() {
            let line = format!("  {:>3}   {:<12}  {:>4}  {:>4}", i + 1, u, s.wins, s.losses);
            c.put_str(cx - str_width(&line) / 2, row, &line, col::WHITE, col::BLACK);
            row += 1;
        }
    } else {
        let hdr = "  名次  玩家            分数      时间";
        c.put_str(cx - str_width(hdr) / 2, row, hdr, col::GRAY, col::BLACK);
        row += 1;
        let entries = eng.scores.get_vec(id.score_key());
        for (i, e) in entries.iter().take(6).enumerate() {
            let line = format!(
                "  {:>3}   {:<12}  {:>7}   {}",
                i + 1, e.user, e.score, e.date
            );
            c.put_str(cx - str_width(&line) / 2, row, &line, col::WHITE, col::BLACK);
            row += 1;
        }
    }

    let foot = "[R] 再来一局     [Enter / ESC / Q] 返回菜单";
    c.put_str(
        cx - str_width(foot) / 2,
        c.h.saturating_sub(2),
        foot,
        col::GRAY,
        col::BLACK,
    );
}

// ---------------- 用户名输入 ----------------

/// 用户名输入界面。首次运行或手动修改时调用。
pub fn username_screen(eng: &mut Engine) {
    let mut buf: String = if eng.user.trim().is_empty() {
        score::system_user()
    } else {
        eng.user.clone()
    };
    eng.canvas.force_redraw();
    let mut msg: &str = "";
    loop {
        if eng.quit_requested {
            return;
        }
        draw_username(eng, &buf, msg);
        let _ = eng.canvas.flush(&mut io::stdout());
        if !event::poll(Duration::from_millis(60)).unwrap_or(false) {
            continue;
        }
        match event::read() {
            Ok(Event::Resize(w, h)) => eng.resize(w, h),
            Ok(Event::Key(k)) if k.kind != KeyEventKind::Release => {
                // Ctrl+C 随时退出
                if (k.code == KeyCode::Char('c') || k.code == KeyCode::Char('C'))
                    && k.modifiers.contains(KeyModifiers::CONTROL)
                {
                    eng.quit_requested = true;
                    return;
                }
                match k.code {
                KeyCode::Char(c) if c.is_alphanumeric() || " -_.".contains(c) => {
                    if buf.chars().count() < 16 {
                        buf.push(c);
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Enter => {
                    let name = buf.trim().to_string();
                    if name.is_empty() {
                        msg = "用户名不能为空";
                    } else {
                        eng.scores.user = name.clone();
                        eng.user = name;
                        let _ = eng.scores.save();
                        return;
                    }
                }
                KeyCode::Esc => {
                    // 取消：保留旧用户名
                    eng.user = eng.scores.user.clone();
                    return;
                }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn draw_username(eng: &mut Engine, buf: &str, msg: &str) {
    let c = &mut eng.canvas;
    c.clear();
    c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
    let w = 56usize;
    let h = 8usize;
    let x = c.w.saturating_sub(w) / 2;
    let y = c.h.saturating_sub(h) / 2;
    c.fill_rect(x, y, w, h, ' ', col::BLACK, col::DARK_BLUE);
    c.border(x, y, w, h, col::CYAN);
    let t = "设置用户名";
    c.put_str(x + (w - str_width(t)) / 2, y + 1, t, col::YELLOW, col::DARK_BLUE);
    c.put_str(x + 2, y + 3, "用户名:", col::WHITE, col::DARK_BLUE);
    c.fill_rect(x + 10, y + 3, w - 14, 1, ' ', col::BLACK, col::BLACK);
    c.put_str(x + 11, y + 3, buf, col::GREEN, col::BLACK);
    let cur_x = x + 11 + str_width(buf);
    if cur_x < c.w {
        c.put(cur_x, y + 3, ' ', col::BLACK, col::GRAY);
    }
    let h1 = "回车 确定    ESC 取消    (字母/数字/空格/.-_)";
    c.put_str(x + 2, y + 5, h1, col::GRAY, col::DARK_BLUE);
    if !msg.is_empty() {
        c.put_str(x + 2, y + 6, msg, col::RED, col::DARK_BLUE);
    }
}

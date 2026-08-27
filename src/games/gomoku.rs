//! 五子棋：人机对战（玩家先手），五子连珠获胜，统计胜败次数。

use crate::app::Engine;
use crate::canvas::{col, Canvas};
use crate::games::{Game, GameOutcome, Status};
use crate::input::Action;
use crate::lang;
use crate::score::ScoreFile;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

const N: usize = 15;
const PLAYER: u8 = 1;
const AI: u8 = 2;

pub struct Gomoku {
    board: [[Option<u8>; N]; N],
    cx: usize,
    cy: usize,
    turn: u8,
    total: usize,
    over: bool,
    win: bool,
    draw: bool,
    ai_pending: bool,
    ai_delay: f64,
    last: Option<(usize, usize)>,
}

impl Gomoku {
    pub fn new() -> Self {
        Gomoku {
            board: [[None; N]; N],
            cx: N / 2,
            cy: N / 2,
            turn: PLAYER,
            total: 0,
            over: false,
            win: false,
            draw: false,
            ai_pending: false,
            ai_delay: 0.0,
            last: None,
        }
    }

    fn place(&mut self, x: usize, y: usize, stone: u8) {
        self.board[y][x] = Some(stone);
        self.total += 1;
        self.last = Some((x, y));
        if self.has_five(x, y, stone) {
            self.over = true;
            self.win = stone == PLAYER;
            return;
        }
        if self.total >= N * N {
            self.over = true;
            self.draw = true;
        }
    }

    /// 以 (x,y) 为中心检测是否有五连。
    fn has_five(&self, x: usize, y: usize, stone: u8) -> bool {
        for (dx, dy) in [(1i32, 0i32), (0, 1), (1, 1), (1, -1)] {
            let mut cnt = 1;
            for s in [1i32, -1] {
                let (mut cx, mut cy) = (x as i32 + dx * s, y as i32 + dy * s);
                while cx >= 0
                    && cy >= 0
                    && (cx as usize) < N
                    && (cy as usize) < N
                    && self.board[cy as usize][cx as usize] == Some(stone)
                {
                    cnt += 1;
                    cx += dx * s;
                    cy += dy * s;
                }
            }
            if cnt >= 5 {
                return true;
            }
        }
        false
    }

    fn in_b(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < N && (y as usize) < N
    }

    fn line_value(cnt: i32, open: i32) -> i32 {
        if cnt >= 5 {
            return 1_000_000;
        }
        match cnt {
            4 => match open {
                2 => 100_000,
                1 => 10_000,
                _ => 1_000,
            },
            3 => match open {
                2 => 10_000,
                1 => 1_000,
                _ => 100,
            },
            2 => match open {
                2 => 1_000,
                1 => 100,
                _ => 10,
            },
            _ => match open {
                2 => 100,
                1 => 10,
                _ => 1,
            },
        }
    }

    /// 评估在 (x,y) 落子 stone 的价值。
    fn score_cell(&self, x: usize, y: usize, stone: u8) -> i32 {
        let mut total = 0;
        for (dx, dy) in [(1i32, 0i32), (0, 1), (1, 1), (1, -1)] {
            let mut cnt = 1;
            let mut open = 0;
            // 正向
            let (mut cx, mut cy) = (x as i32 + dx, y as i32 + dy);
            while Self::in_b(cx, cy) && self.board[cy as usize][cx as usize] == Some(stone) {
                cnt += 1;
                cx += dx;
                cy += dy;
            }
            if Self::in_b(cx, cy) && self.board[cy as usize][cx as usize].is_none() {
                open += 1;
            }
            // 反向
            let (mut cx, mut cy) = (x as i32 - dx, y as i32 - dy);
            while Self::in_b(cx, cy) && self.board[cy as usize][cx as usize] == Some(stone) {
                cnt += 1;
                cx -= dx;
                cy -= dy;
            }
            if Self::in_b(cx, cy) && self.board[cy as usize][cx as usize].is_none() {
                open += 1;
            }
            total += Self::line_value(cnt, open);
        }
        total
    }

    fn has_neighbor(&self, x: usize, y: usize) -> bool {
        for yy in (y as i32 - 2)..=(y as i32 + 2) {
            for xx in (x as i32 - 2)..=(x as i32 + 2) {
                if Self::in_b(xx, yy) && self.board[yy as usize][xx as usize].is_some() {
                    return true;
                }
            }
        }
        false
    }

    /// AI 选点并落子。
    fn ai_place(&mut self) {
        if self.over {
            return;
        }
        // 第一步：天元
        if self.total == 0 {
            self.place(N / 2, N / 2, AI);
            self.turn = PLAYER;
            return;
        }
        let mut candidates: Vec<(usize, usize)> = Vec::new();
        for y in 0..N {
            for x in 0..N {
                if self.board[y][x].is_none() && self.has_neighbor(x, y) {
                    candidates.push((x, y));
                }
            }
        }
        if candidates.is_empty() {
            self.turn = PLAYER;
            return;
        }
        // 1) 自己能成五 → 直接赢
        for &(x, y) in &candidates {
            if self.score_cell(x, y, AI) >= 1_000_000 {
                self.place(x, y, AI);
                self.turn = PLAYER;
                return;
            }
        }
        // 2) 对手下一步成五 → 必须堵
        for &(x, y) in &candidates {
            if self.score_cell(x, y, PLAYER) >= 1_000_000 {
                self.place(x, y, AI);
                self.turn = PLAYER;
                return;
            }
        }
        // 3) 启发式打分
        let mut rng = rand::thread_rng();
        let mut best_score = i32::MIN;
        let mut scored: Vec<(i32, usize, usize)> = Vec::new();
        for &(x, y) in &candidates {
            let off = self.score_cell(x, y, AI);
            let def = self.score_cell(x, y, PLAYER);
            let center = 6 - ((x as i32 - 7).abs() + (y as i32 - 7).abs()) / 2;
            let s = off + def + center;
            if s > best_score {
                best_score = s;
                scored.clear();
                scored.push((s, x, y));
            } else if s == best_score {
                scored.push((s, x, y));
            }
        }
        let pick = scored[rng.gen_range(0..scored.len())];
        self.place(pick.1, pick.2, AI);
        self.turn = PLAYER;
    }
}

impl Game for Gomoku {
    fn update(&mut self, dt: f64, _eng: &mut Engine) {
        if self.over {
            return;
        }
        if self.ai_pending {
            self.ai_delay -= dt;
            if self.ai_delay <= 0.0 {
                self.ai_pending = false;
                self.ai_place();
            }
        }
    }

    fn handle(&mut self, a: Action, _eng: &mut Engine) {
        if self.over || self.ai_pending {
            return;
        }
        match a {
            Action::Left => self.cx = self.cx.saturating_sub(1),
            Action::Right => self.cx = (self.cx + 1).min(N - 1),
            Action::Up => self.cy = self.cy.saturating_sub(1),
            Action::Down => self.cy = (self.cy + 1).min(N - 1),
            Action::Confirm | Action::Space => {
                if self.turn == PLAYER && self.board[self.cy][self.cx].is_none() {
                    self.place(self.cx, self.cy, PLAYER);
                    if !self.over {
                        self.turn = AI;
                        self.ai_pending = true;
                        self.ai_delay = 0.15;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&self, c: &mut Canvas, scores: &ScoreFile, user: &str) {
        c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
        let ox = c.w.saturating_sub(2 * N + 1) / 2;
        let oy = c.h.saturating_sub(N) / 2;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        // 网格与棋子
        for r in 0..N {
            for col in 0..N {
                let x = ox + col * 2;
                let y = oy + r;
                let is_cursor = col == self.cx && r == self.cy;
                let bg = if is_cursor { col::GRAY } else { col::BLACK };
                match self.board[r][col] {
                    Some(1) => {
                        c.put(x, y, 'X', col::GREEN, bg);
                    }
                    Some(2) => {
                        c.put(x, y, 'O', col::RED, bg);
                    }
                    None => {
                        if is_cursor {
                            let blink = (now / 350) % 2 == 0;
                            let bg2 = if blink { col::GRAY } else { col::DARK_BLUE };
                            c.put(x, y, '·', col::YELLOW, bg2);
                        } else {
                            c.put(x, y, '+', col::DARK_GREEN, col::BLACK);
                        }
                    }
                    _ => {}
                }
                if col < N - 1 {
                    c.put(x + 1, y, '-', col::DARK_GREEN, col::BLACK);
                }
            }
        }
        // 坐标轴
        let letters = "a b c d e f g h i j k l m n o";
        c.put_str(ox, oy + N + 1, letters, col::GRAY, col::BLACK);
        for r in 0..N {
            let n = format!("{:>2}", r + 1);
            c.put_str(ox.saturating_sub(3), oy + r, &n, col::GRAY, col::BLACK);
        }

        // 右侧面板
        let px = ox + 2 * N + 3;
        let mut py = oy;
        c.put_str(px, py, lang::ui().gomoku_title, col::YELLOW, col::BLACK);
        py += 2;
        c.put_str(px, py, lang::ui().you_first, col::GREEN, col::BLACK);
        py += 1;
        c.put_str(px, py, lang::ui().ai_stone, col::RED, col::BLACK);
        py += 2;
        let st = scores.gomoku_stats(user);
        let stat = lang::fmt(lang::ui().stat_fmt, &[&st.wins, &st.losses]);
        c.put_str(px, py, &stat, col::CYAN, col::BLACK);
        py += 2;
        let turn_txt = if self.over {
            if self.draw {
                lang::ui().draw
            } else if self.win {
                lang::ui().you_win
            } else {
                lang::ui().you_lose
            }
        } else if self.ai_pending {
            lang::ui().ai_thinking
        } else {
            lang::ui().your_turn
        };
        let tfg = if self.over && !self.draw {
            if self.win {
                col::GREEN
            } else {
                col::RED
            }
        } else {
            col::WHITE
        };
        c.put_str(px, py, turn_txt, tfg, col::BLACK);
        py += 3;
        let help = lang::ui().gomoku_help;
        for h in help {
            c.put_str(px, py, h, col::GRAY, col::BLACK);
            py += 1;
        }
    }

    fn status(&self) -> Status {
        if self.over {
            Status::Finished
        } else {
            Status::Running
        }
    }

    fn outcome(&self) -> GameOutcome {
        if self.draw {
            GameOutcome::GomokuDraw
        } else {
            GameOutcome::Gomoku { win: self.win }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_board(cells: &[(usize, usize, u8)]) -> Gomoku {
        let mut g = Gomoku::new();
        for &(x, y, s) in cells {
            g.board[y][x] = Some(s);
        }
        g
    }

    #[test]
    fn detects_five_horizontal() {
        let g = make_board(&[(0, 0, PLAYER), (1, 0, PLAYER), (2, 0, PLAYER), (3, 0, PLAYER)]);
        // 只有 4 连不算
        assert!(!g.has_five(3, 0, PLAYER));
        // 补最后一子成五
        let mut g2 = make_board(&[(0, 0, PLAYER), (1, 0, PLAYER), (2, 0, PLAYER), (3, 0, PLAYER)]);
        g2.board[0][4] = Some(PLAYER);
        assert!(g2.has_five(4, 0, PLAYER));
    }

    #[test]
    fn detects_five_diagonal() {
        let mut g = Gomoku::new();
        for i in 0..5 {
            g.board[i][i] = Some(AI);
        }
        assert!(g.has_five(4, 4, AI));
    }

    #[test]
    fn score_prefers_win() {
        let g = make_board(&[(0, 0, PLAYER), (1, 0, PLAYER), (2, 0, PLAYER), (3, 0, PLAYER)]);
        let win_cell = g.score_cell(4, 0, PLAYER);
        assert!(win_cell >= 1_000_000);
        let other = g.score_cell(5, 5, PLAYER);
        assert!(other < win_cell);
    }

    #[test]
    fn ai_blocks_win() {
        // 玩家四连，AI 应堵住第五点
        let mut g = make_board(&[(0, 7, PLAYER), (1, 7, PLAYER), (2, 7, PLAYER), (3, 7, PLAYER)]);
        g.total = 4;
        g.turn = AI;
        g.ai_place();
        // AI 必须下在 (4,7) 或 (4 的两端) 或 (-1 不可行)
        let blocked = g.board[7][4].is_some() || g.board[7][4] == Some(AI);
        assert!(blocked, "AI 应堵住四连");
    }

    #[test]
    fn ai_takes_center_first() {
        let mut g = Gomoku::new();
        g.turn = AI;
        g.ai_place();
        assert_eq!(g.board[7][7], Some(AI));
    }
}

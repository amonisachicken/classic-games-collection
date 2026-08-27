//! 俄罗斯方块：消行得分，方块越落越快。

use crate::app::Engine;
use crate::canvas::{col, Canvas};
use crate::games::{Game, GameOutcome, Status};
use crate::input::Action;
use crate::score::ScoreFile;
use rand::Rng;

const W: usize = 10;
const H: usize = 20;

/// 7 种方块，每种 4 个旋转状态，每个状态 4 个单元格偏移 (dx, dy)（相对原点）。
const SHAPES: [[[(i32, i32); 4]; 4]; 7] = [
    // I
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ],
    // O
    [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ],
    // T
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // S
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // Z
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ],
    // J
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    // L
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
];

const PIECE_COLORS: [crossterm::style::Color; 7] = [
    col::CYAN,
    col::YELLOW,
    col::MAGENTA,
    col::GREEN,
    col::RED,
    col::BLUE,
    col::ORANGE,
];

#[derive(Clone, Copy)]
struct Piece {
    kind: u8,
    rot: usize,
    x: i32,
    y: i32,
}

impl Piece {
    fn cells(&self) -> [(i32, i32); 4] {
        SHAPES[self.kind as usize][self.rot]
    }
}

pub struct Tetris {
    board: Vec<Vec<Option<u8>>>,
    cur: Piece,
    next_kind: u8,
    score: u32,
    lines: u32,
    level: u32,
    drop_acc: f64,
    /// 最后一次按 ↓ 的时间（窗口期判定软降）
    soft_since: f64,
    time: f64,
    over: bool,
}

impl Tetris {
    pub fn new() -> Self {
        let board = vec![vec![None; W]; H];
        let mut t = Tetris {
            board,
            cur: Piece { kind: 0, rot: 0, x: 0, y: 0 },
            next_kind: 0,
            score: 0,
            lines: 0,
            level: 1,
            drop_acc: 0.0,
            soft_since: -1000.0,
            time: 0.0,
            over: false,
        };
        t.next_kind = t.random_kind();
        t.spawn();
        t
    }

    fn random_kind(&self) -> u8 {
        let mut rng = rand::thread_rng();
        let mut k = rng.gen_range(0..7) as u8;
        while k == self.cur.kind && !self.over {
            k = rng.gen_range(0..7) as u8;
        }
        k
    }

    fn spawn(&mut self) {
        self.cur = Piece {
            kind: self.next_kind,
            rot: 0,
            x: 3,
            y: -1,
        };
        self.next_kind = self.random_kind();
        if !self.valid(&self.cur) {
            self.over = true;
        }
    }

    fn valid(&self, p: &Piece) -> bool {
        for (dx, dy) in p.cells() {
            let cx = p.x + dx;
            let cy = p.y + dy;
            if cx < 0 || cx >= W as i32 {
                return false;
            }
            if cy >= H as i32 {
                return false;
            }
            if cy >= 0 && self.board[cy as usize][cx as usize].is_some() {
                return false;
            }
        }
        true
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let mut p = self.cur;
        p.x += dx;
        p.y += dy;
        if self.valid(&p) {
            self.cur = p;
            true
        } else {
            false
        }
    }

    fn rotate(&mut self) {
        let mut p = self.cur;
        p.rot = (p.rot + 1) % 4;
        if self.valid(&p) {
            self.cur = p;
            return;
        }
        // 墙踢：尝试左右偏移
        for kick in [1i32, -1, 2, -2] {
            let mut k = p;
            k.x += kick;
            if self.valid(&k) {
                self.cur = k;
                return;
            }
        }
    }

    fn hard_drop(&mut self) {
        let mut dropped = 0i32;
        loop {
            let mut p = self.cur;
            p.y += 1;
            if self.valid(&p) {
                self.cur = p;
                dropped += 1;
            } else {
                break;
            }
        }
        self.score = (self.score as i64 + dropped as i64 * 2).max(0) as u32;
        self.lock();
    }

    fn lock(&mut self) {
        for (dx, dy) in self.cur.cells() {
            let cx = self.cur.x + dx;
            let cy = self.cur.y + dy;
            if cy < 0 {
                // 方块锁在顶行之上 → 游戏结束
                self.over = true;
                return;
            }
            self.board[cy as usize][cx as usize] = Some(self.cur.kind);
        }
        self.clear_lines();
        if !self.over {
            self.spawn();
        }
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut new_board = Vec::with_capacity(H);
        for row in self.board.iter() {
            if row.iter().all(|c| c.is_some()) {
                cleared += 1;
            } else {
                new_board.push(row.clone());
            }
        }
        if cleared > 0 {
            for _ in 0..cleared {
                new_board.insert(0, vec![None; W]);
            }
            self.board = new_board;
            self.lines += cleared;
            let base = [0u32, 100, 300, 500, 800][(cleared as usize).min(4)];
            self.score += base * self.level;
            self.level = self.lines / 10 + 1;
        }
    }

    fn drop_interval(&self) -> f64 {
        (0.8 * 0.8f64.powf(self.level as f64 - 1.0)).max(0.05)
    }

    /// 软降倍率：按下 ↓ 后的窗口期内加速，窗口过期自动恢复。
    fn soft_mult(&self) -> f64 {
        const SOFT_WINDOW: f64 = 0.12;
        if self.time - self.soft_since < SOFT_WINDOW {
            18.0
        } else {
            1.0
        }
    }
}

impl Game for Tetris {
    fn update(&mut self, dt: f64, _eng: &mut Engine) {
        if self.over {
            return;
        }
        self.time += dt;
        self.drop_acc += dt * self.soft_mult();
        let interval = self.drop_interval();
        while self.drop_acc >= interval {
            self.drop_acc -= interval;
            if !self.try_move(0, 1) {
                self.drop_acc = 0.0;
                self.lock();
                if self.over {
                    return;
                }
                break;
            }
        }
    }

    fn handle(&mut self, a: Action, _eng: &mut Engine) {
        if self.over {
            return;
        }
        match a {
            Action::Left => {
                self.try_move(-1, 0);
            }
            Action::Right => {
                self.try_move(1, 0);
            }
            Action::Up => self.rotate(),
            Action::Down => self.soft_since = self.time,
            Action::Space | Action::Confirm => self.hard_drop(),
            _ => {}
        }
    }

    fn draw(&self, c: &mut Canvas, scores: &ScoreFile, _user: &str) {
        c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
        // 布局：井 + 右侧面板
        let panel_w = 18usize;
        let ox = c.w.saturating_sub(W + 2 + panel_w) / 2;
        let oy = c.h.saturating_sub(H + 2) / 2;
        let bx = ox + 1;
        let by = oy + 1;

        // 井边框
        c.border(ox, oy, W + 2, H + 2, col::CYAN);
        // 已落方块
        for (r, row) in self.board.iter().enumerate() {
            for (col, cell) in row.iter().enumerate() {
                if let Some(k) = cell {
                    c.put(bx + col, by + r, '█', PIECE_COLORS[*k as usize], col::BLACK);
                }
            }
        }
        // 当前方块（含悬在顶上的部分不画）
        for (dx, dy) in self.cur.cells() {
            let cx = self.cur.x + dx;
            let cy = self.cur.y + dy;
            if cx >= 0 && cx < W as i32 && cy >= 0 && cy < H as i32 {
                c.put(
                    bx + cx as usize,
                    by + cy as usize,
                    '█',
                    PIECE_COLORS[self.cur.kind as usize],
                    col::BLACK,
                );
            }
        }

        // 面板
        let px = ox + W + 2 + 2;
        let mut py = oy;
        c.put_str(px, py, "俄罗斯方块", col::YELLOW, col::BLACK);
        py += 1;
        c.put_str(px, py, "TETRIS", col::CYAN, col::BLACK);
        py += 2;
        c.put_str(px, py, &format!("得分 {}", self.score), col::GREEN, col::BLACK);
        py += 1;
        c.put_str(px, py, &format!("行数 {}", self.lines), col::WHITE, col::BLACK);
        py += 1;
        c.put_str(px, py, &format!("等级 {}", self.level), col::MAGENTA, col::BLACK);
        py += 1;
        if let Some(e) = scores.get_vec("tetris").first() {
            c.put_str(px, py, &format!("最高 {}", e.score), col::GRAY, col::BLACK);
            py += 1;
            c.put_str(px, py, &format!("({})", e.user), col::GRAY, col::BLACK);
            py += 1;
        }
        py += 1;
        c.put_str(px, py, "下一个", col::WHITE, col::BLACK);
        py += 1;
        // 预览
        let cells = SHAPES[self.next_kind as usize][0];
        let minx = cells.iter().map(|c| c.0).min().unwrap();
        let miny = cells.iter().map(|c| c.1).min().unwrap();
        for (dx, dy) in cells {
            c.put(
                px + (dx - minx) as usize,
                py + (dy - miny) as usize,
                '█',
                PIECE_COLORS[self.next_kind as usize],
                col::BLACK,
            );
        }
        py += 4;
        let help = [
            "←→ / HL  左右移动",
            "↑ / K     旋转",
            "↓ / J     加速下落",
            "空格       直落",
            "ESC/Q      暂停",
        ];
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
        GameOutcome::Score(self.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Engine;
    use crate::input::Action;
    use crate::score::ScoreFile;

    #[test]
    fn soft_drop_window_expires() {
        let mut scores = ScoreFile::default();
        let mut eng = Engine::test_engine(&mut scores);
        let mut t = Tetris::new();
        assert_eq!(t.soft_mult(), 1.0, "初始不应软降");
        t.handle(Action::Down, &mut eng);
        assert_eq!(t.soft_mult(), 18.0, "按 ↓ 应加速");
        let y0 = t.cur.y;
        t.update(0.05, &mut eng);
        assert!(t.cur.y > y0, "软降期间方块应更快下落");
        t.update(0.15, &mut eng); // 超过 0.12s 窗口
        assert_eq!(t.soft_mult(), 1.0, "窗口过期应恢复普通速度");
    }
}

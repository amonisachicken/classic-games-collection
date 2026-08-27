//! 贪吃蛇：吃食物变长，撞墙/撞自己结束。

use crate::app::Engine;
use crate::canvas::{col, str_width, Canvas};
use crate::games::{Game, GameOutcome, Status};
use crate::input::Action;
use crate::lang;
use crate::score::ScoreFile;
use rand::Rng;
use std::collections::VecDeque;

const BW: usize = 32; // 棋盘宽（格）
const BH: usize = 20; // 棋盘高（格）

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub struct Snake {
    body: VecDeque<(usize, usize)>,
    dir: Dir,
    next_dir: Dir,
    food: (usize, usize),
    score: u32,
    move_acc: f64,
    interval: f64,
    paused: bool,
    over: bool,
}

impl Snake {
    pub fn new() -> Self {
        let mut s = Snake {
            body: VecDeque::new(),
            dir: Dir::Right,
            next_dir: Dir::Right,
            food: (0, 0),
            score: 0,
            move_acc: 0.0,
            interval: 0.14,
            paused: false,
            over: false,
        };
        // 初始蛇：中间偏左，长 4，头在最右（向右移动）
        let sy = BH / 2;
        for i in 0..4 {
            s.body.push_back((BW / 2 - 2 - i, sy));
        }
        s.spawn_food();
        s
    }

    fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let f = (rng.gen_range(0..BW), rng.gen_range(0..BH));
            if !self.body.contains(&f) {
                self.food = f;
                return;
            }
        }
    }

    /// 尝试转向（忽略 180° 反向）。
    fn turn(&mut self, d: Dir) {
        let opposite = match d {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        };
        if self.dir != opposite {
            self.next_dir = d;
        }
    }

    fn step(&mut self) {
        self.dir = self.next_dir;
        let (hx, hy) = *self.body.front().unwrap();
        let (nx, ny) = match self.dir {
            Dir::Up => (hx, hy.wrapping_sub(1)),
            Dir::Down => (hx, hy + 1),
            Dir::Left => (hx.wrapping_sub(1), hy),
            Dir::Right => (hx + 1, hy),
        };
        // 撞墙
        if nx >= BW || ny >= BH {
            self.over = true;
            return;
        }
        // 撞自己（尾巴即将移走的位置不算撞）
        let eating = (nx, ny) == self.food;
        let will_grow = eating;
        let tail = *self.body.back().unwrap();
        if self.body.contains(&(nx, ny)) && !(will_grow && (nx, ny) == tail) {
            self.over = true;
            return;
        }
        self.body.push_front((nx, ny));
        if eating {
            self.score += 10;
            self.interval = (0.14 - self.score as f64 * 0.0012).max(0.055);
            self.spawn_food();
        } else {
            self.body.pop_back();
        }
    }

    fn draw_board(&self, c: &mut Canvas, ox: usize, oy: usize) {
        // 边框
        c.border(ox, oy, BW + 2, BH + 2, col::CYAN);
        // 食物
        c.put(ox + 1 + self.food.0, oy + 1 + self.food.1, '●', col::RED, col::BLACK);
        // 蛇身
        for (i, &(x, y)) in self.body.iter().enumerate() {
            let fg = if i == 0 { col::YELLOW } else { col::GREEN };
            c.put(ox + 1 + x, oy + 1 + y, '█', fg, col::BLACK);
        }
    }
}

impl Game for Snake {
    fn update(&mut self, dt: f64, _eng: &mut Engine) {
        if self.over || self.paused {
            return;
        }
        self.move_acc += dt;
        while self.move_acc >= self.interval {
            self.move_acc -= self.interval;
            self.step();
            if self.over {
                return;
            }
        }
    }

    fn handle(&mut self, a: Action, _eng: &mut Engine) {
        if self.over {
            return;
        }
        match a {
            Action::Up => self.turn(Dir::Up),
            Action::Down => self.turn(Dir::Down),
            Action::Left => self.turn(Dir::Left),
            Action::Right => self.turn(Dir::Right),
            Action::Space => self.paused = !self.paused,
            _ => {}
        }
    }

    fn draw(&self, c: &mut Canvas, scores: &ScoreFile, _user: &str) {
        c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
        let ox = c.w.saturating_sub(BW + 2) / 2;
        let oy = c.h.saturating_sub(BH + 2) / 2;
        self.draw_board(c, ox, oy);

        // 顶部面板
        let title = lang::ui().snake_title;
        let score = lang::fmt(lang::ui().score_fmt, &[&self.score]);
        let high = scores
            .get_vec("snake")
            .first()
            .map(|e| lang::fmt(lang::ui().best_fmt, &[&e.score, &e.user]))
            .unwrap_or_else(|| lang::ui().no_record.to_string());
        let py = oy.saturating_sub(2);
        c.put_str(ox, py, title, col::YELLOW, col::BLACK);
        c.put_str(ox, py + 1, &score, col::GREEN, col::BLACK);
        let hx = ox + BW + 2 - str_width(&high);
        c.put_str(hx, py, &high, col::GRAY, col::BLACK);

        // 底部提示
        let help = lang::ui().snake_help;
        c.put_str(ox, oy + BH + 3, help, col::GRAY, col::BLACK);

        if self.paused {
            let t = lang::ui().snake_paused;
            let px = ox + (BW + 2 - str_width(t)) / 2;
            let py2 = oy + BH / 2;
            c.put_str(px, py2, t, col::YELLOW, col::BLACK);
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

    #[test]
    fn starts_alive_and_moves() {
        let mut s = Snake::new();
        assert!(!s.over);
        // 向前走几步（向右），不应立刻死亡
        for _ in 0..5 {
            s.step();
            assert!(!s.over, "蛇开局不应死亡");
        }
        // 头部应持续向右移动
        let head0 = *s.body.front().unwrap();
        assert!(head0.0 > BW / 2 - 2, "头部应向右移动");
    }

    #[test]
    fn reverse_direction_ignored() {
        let mut s = Snake::new();
        // 当前向右，按 Left 应被忽略
        s.turn(Dir::Left);
        let h = *s.body.front().unwrap();
        s.step();
        let h2 = *s.body.front().unwrap();
        assert!(h2.0 > h.0, "反向输入不应生效");
        // 按 Up 应生效
        s.turn(Dir::Up);
        s.dir = Dir::Up; // 直接设置当前方向为 Up 后再转向
        s.turn(Dir::Down);
        assert_eq!(s.next_dir, Dir::Up, "向下反向应被忽略");
    }

    #[test]
    fn eats_food_and_grows() {
        let mut s = Snake::new();
        let head = *s.body.front().unwrap();
        let len0 = s.body.len();
        // 把食物放在蛇头正前方
        s.food = (head.0 + 1, head.1);
        s.step();
        assert_eq!(s.body.len(), len0 + 1, "吃到食物应增长");
        assert_eq!(s.score, 10);
    }
}

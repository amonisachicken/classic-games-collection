//! 打砖块：接住小球，打碎所有砖块。三命，球落底失命。

use crate::app::Engine;
use crate::canvas::{col, str_width, Canvas};
use crate::games::{Game, GameOutcome, Status};
use crate::input::Action;
use crate::lang;
use crate::score::ScoreFile;
use rand::Rng;

/// 场地几何（单元 = 1 字符格）。
/// 左墙 x=0，右墙 x=41；砖块区列 1..=40，行 1..=6。
const FIELD_W: usize = 38; // 9 列砖块 × 4 格 = 36 格场地 + 左右墙
// 每个游戏格占 2 半角字符(方块像素), 场地宽 = 38×2 = 76 字符
const FIELD_H: usize = 24;
const BRICK_COLS: usize = 9;
const BRICK_ROWS: usize = 6;
const BRICK_W: f64 = 3.0;
const PADDLE_W: f64 = 7.0;
const PADDLE_Y: f64 = 21.0;
const MAX_LIVES: i32 = 3;

const BRICK_COLORS: [crossterm::style::Color; BRICK_ROWS] = [
    col::RED,
    col::ORANGE,
    col::YELLOW,
    col::GREEN,
    col::CYAN,
    col::BLUE,
];

pub struct Breakout {
    bricks: Vec<Vec<bool>>, // [row][col]
    bricks_left: usize,
    px: f64, // 挡板左端 x
    /// 最后一次收到左/右键事件的时间（用于窗口期判定按住状态）
    left_since: f64,
    right_since: f64,
    /// 游戏内部时钟
    time: f64,
    /// None = 待发球（球在挡板上）
    ball: Option<(f64, f64, f64, f64)>, // x, y, dx, dy
    speed: f64,
    lives: i32,
    score: u32,
    ready: bool,
    over: bool,
    /// 掉落的医疗包 (x, y) 单元格坐标
    packs: Vec<(f64, f64)>,
    /// 拾取提示 (剩余时间, 文本)
    hint: Option<(f64, String)>,
}

impl Breakout {
    pub fn new() -> Self {
        let bricks = vec![vec![true; BRICK_COLS]; BRICK_ROWS];
        Breakout {
            bricks_left: BRICK_COLS * BRICK_ROWS,
            bricks,
            px: (FIELD_W as f64 - PADDLE_W) / 2.0,
            left_since: -1000.0,
            right_since: -1000.0,
            time: 0.0,
            ball: None,
            speed: 7.5,
            lives: MAX_LIVES,
            score: 0,
            ready: true,
            over: false,
            packs: Vec::new(),
            hint: None,
        }
    }

    /// 医疗包掉落判定：击碎砖块时 1/20 概率。
    fn roll_pack(&self, rng: &mut impl Rng) -> bool {
        rng.gen_range(0.0..1.0) < 0.05
    }
    /// 医疗包：竖直下落，挡板接住回血，掉出底部消失。
    fn update_packs(&mut self, dt: f64) {
        const PACK_SPEED: f64 = 3.5;
        for pk in self.packs.iter_mut() {
            pk.1 += PACK_SPEED * dt;
        }
        // 挡板接住
        let mut catch: Vec<usize> = Vec::new();
        for (pi, &(x, y)) in self.packs.iter().enumerate() {
            if y >= PADDLE_Y - 0.6
                && y <= PADDLE_Y + 1.0
                && x >= self.px - 0.4
                && x <= self.px + PADDLE_W + 0.4
            {
                catch.push(pi);
            }
        }
        for &pi in catch.iter().rev() {
            if pi < self.packs.len() {
                self.packs.remove(pi);
                if self.lives < MAX_LIVES {
                    self.lives += 1;
                    self.hint = Some((0.9, lang::ui().heal_fmt.to_string()));
                } else {
                    self.hint = Some((0.9, lang::ui().life_full.to_string()));
                }
            }
        }
        // 掉出底部
        self.packs.retain(|pk| pk.1 <= FIELD_H as f64 + 1.0);
        // 提示倒计时
        if let Some((ttl, _)) = &mut self.hint {
            *ttl -= dt;
            if *ttl <= 0.0 {
                self.hint = None;
            }
        }
    }



    fn brick_cell(&self, row: usize, col: usize) -> (f64, f64, f64, f64) {
        // 返回 (x0, y0, x1, y1) 闭区间
        let x0 = 1.0 + col as f64 * (BRICK_W + 1.0);
        let y0 = 1.0 + row as f64;
        (x0, y0, x0 + BRICK_W, y0 + 1.0)
    }

    fn brick_at(&self, bx: f64, by: f64) -> Option<(usize, usize)> {
        // 砖块列: x0 = 1 + c*4 → c = (x-1)/4 向下取整；行: r = (y-1) 向下取整。
        // 注意最底行砖块位于 y∈[6,7)，这里不做 y 上限预判，交给下方边界检查。
        let c = ((bx - 1.0) / 4.0).floor() as i64;
        let r = (by - 1.0).floor() as i64;
        if r >= 0 && r < BRICK_ROWS as i64 && c >= 0 && c < BRICK_COLS as i64 {
            let (r, c) = (r as usize, c as usize);
            if self.bricks[r][c] {
                return Some((r, c));
            }
        }
        None
    }

    fn launch(&mut self) {
        if !self.ready || self.ball.is_some() {
            return;
        }
        let bx = self.px + PADDLE_W / 2.0;
        let by = PADDLE_Y - 0.8;
        self.ball = Some((bx, by, 0.0, -1.0));
        self.ready = false;
    }

    fn reset_ball(&mut self) {
        self.ball = None;
        self.ready = true;
    }

    fn move_paddle(&mut self, dt: f64) {
        // 按键窗口期（秒）：按住时终端的按键重复事件会持续刷新窗口；
        // 松开后（部分终端不发送释放事件）窗口过期自动停止。
        const HOLD_WINDOW: f64 = 0.15;
        let left = self.time - self.left_since < HOLD_WINDOW;
        let right = self.time - self.right_since < HOLD_WINDOW;
        let dir = (right as i32 - left as i32) as f64;
        if dir == 0.0 {
            return;
        }
        let mut nx = self.px + dir * 26.0 * dt;
        nx = nx.clamp(1.0, FIELD_W as f64 - 1.0 - PADDLE_W);
        self.px = nx;
        // 待发球时球跟随挡板（在 step_ball 里处理）
    }

    fn step_ball(&mut self, dt: f64) {
        let mut b = match self.ball {
            Some(b) => b,
            None => return,
        };
        if self.ready {
            // 球在挡板上跟随
            b.0 = self.px + PADDLE_W / 2.0;
            b.1 = PADDLE_Y - 0.8;
            self.ball = Some(b);
            return;
        }
        let (mut bx, mut by, mut dx, mut dy) = b;
        // 速度归一
        let sp = self.speed;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        dx = dx / len * sp;
        dy = dy / len * sp;

        // 拆分步进，防止高速穿透（每步不超过 0.35 格）
        let steps = ((sp * dt) / 0.35).ceil().max(1.0) as u32;
        let sdt = dt / steps as f64;
        for _ in 0..steps {
            bx += dx * sdt;
            by += dy * sdt;
            // 左右墙
            if bx <= 1.0 {
                bx = 1.0;
                dx = dx.abs();
            } else if bx >= 36.0 {
                bx = 36.0;
                dx = -dx.abs();
            }
            // 顶墙
            if by <= 1.0 {
                by = 1.0;
                dy = dy.abs();
            }
            // 挡板
            if dy > 0.0 && by >= PADDLE_Y - 0.6 && by <= PADDLE_Y + 0.6
                && bx >= self.px - 0.4 && bx <= self.px + PADDLE_W + 0.4
            {
                let rel = ((bx - (self.px + PADDLE_W / 2.0)) / (PADDLE_W / 2.0)).clamp(-1.0, 1.0);
                dx = rel * 2.6;
                dy = -1.0;
                by = PADDLE_Y - 0.8;
                let len = (dx * dx + dy * dy).sqrt();
                dx = dx / len * sp;
                dy = dy / len * sp;
            }
            // 砖块
            if let Some((r, c)) = self.brick_at(bx, by) {
                let (x0, y0, x1, y1) = self.brick_cell(r, c);
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                self.bricks[r][c] = false;
                self.bricks_left -= 1;
                self.score += 10;
                self.speed = (7.5 + self.score as f64 * 0.012).min(12.0);
                // 医疗包掉落：1/20 概率，从砖块中心落下
                let mut rng = rand::thread_rng();
                if self.roll_pack(&mut rng) {
                    self.packs.push((cx, y0));
                }
                if (bx - cx).abs() > (by - cy).abs() {
                    dx = if bx < cx { -dx.abs() } else { dx.abs() };
                    // 推离砖块
                    bx = if dx < 0.0 { x0 - 0.1 } else { x1 + 0.1 };
                } else {
                    dy = if by < cy { -dy.abs() } else { dy.abs() };
                    by = if dy < 0.0 { y0 - 0.1 } else { y1 + 0.1 };
                }
                if self.bricks_left == 0 {
                    self.score += 100;
                    self.over = true;
                    self.ball = None;
                    return;
                }
            }
            // 落底
            if by > FIELD_H as f64 + 1.0 {
                self.lives -= 1;
                if self.lives <= 0 {
                    self.over = true;
                    self.ball = None;
                    return;
                }
                self.reset_ball();
                return;
            }
        }
        self.ball = Some((bx, by, dx, dy));
    }
}

impl Game for Breakout {
    fn update(&mut self, dt: f64, _eng: &mut Engine) {
        if self.over {
            return;
        }
        self.time += dt;
        self.move_paddle(dt);
        self.step_ball(dt);
        self.update_packs(dt);
    }

    fn handle(&mut self, a: Action, _eng: &mut Engine) {
        if self.over {
            return;
        }
        match a {
            Action::Left => self.left_since = self.time,
            Action::Right => self.right_since = self.time,
            Action::Space | Action::Confirm => {
                if self.ready {
                    self.launch();
                }
            }
            _ => {}
        }
    }

    fn draw(&self, c: &mut Canvas, scores: &ScoreFile, _user: &str) {
        c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
        let ox = c.w.saturating_sub(FIELD_W * 2) / 2;
        let oy = c.h.saturating_sub(FIELD_H) / 2;
        // 边框
        c.border(ox, oy, FIELD_W * 2, FIELD_H, col::CYAN);
        // 砖块
        for r in 0..BRICK_ROWS {
            for col in 0..BRICK_COLS {
                if !self.bricks[r][col] {
                    continue;
                }
                let (x0, y0, x1, _y1) = self.brick_cell(r, col);
                for x in (x0 as usize)..=(x1 as usize) {
                    c.put_block(ox + x * 2, oy + y0 as usize, BRICK_COLORS[r], col::BLACK);
                }
            }
        }
        // 挡板
        for i in 0..PADDLE_W as usize {
            c.put_block(ox + (self.px as usize + i) * 2, oy + PADDLE_Y as usize, col::YELLOW, col::BLACK);
        }
        // 球
        if let Some((bx, by, _, _)) = self.ball {
            c.put(ox + (bx as usize) * 2, oy + by as usize, '●', col::WHITE, col::BLACK);
        }
        // 医疗包
        for &(x, y) in &self.packs {
            c.put(ox + (x as usize) * 2, oy + y as usize, '♥', col::PINK, col::BLACK);
        }
        // 拾取提示（显示在挡板上方）
        if let Some((ttl, text)) = &self.hint {
            if *ttl > 0.0 {
                let sx = ox + (self.px as usize) * 2;
                let sy = (oy + PADDLE_Y as usize).saturating_sub(1);
                let tx = sx.saturating_sub(str_width(text) / 2);
                c.put_str(tx, sy, text, col::GREEN, col::BLACK);
            }
        }

        // 面板
        let title = lang::ui().breakout_title;
        let score = lang::fmt(lang::ui().score_fmt, &[&self.score]);
        let lives = lang::fmt(lang::ui().lives_fmt, &[&"♥".repeat(self.lives.max(0) as usize)]);
        let high = scores
            .get_vec("breakout")
            .first()
            .map(|e| lang::fmt(lang::ui().best_fmt, &[&e.score, &e.user]))
            .unwrap_or_else(|| lang::ui().no_record.to_string());
        let py = oy.saturating_sub(2);
        c.put_str(ox, py, title, col::YELLOW, col::BLACK);
        c.put_str(ox, py + 1, &score, col::GREEN, col::BLACK);
        c.put_str(ox + str_width(&score) + 2, py + 1, &lives, col::RED, col::BLACK);
        let hx = ox + FIELD_W * 2 - str_width(&high);
        c.put_str(hx, py, &high, col::GRAY, col::BLACK);

        let help = if self.ready && self.ball.is_none() {
            lang::ui().breakout_help_ready
        } else {
            lang::ui().breakout_help
        };
        c.put_str(ox, oy + FIELD_H + 1, help, col::GRAY, col::BLACK);
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
    use crate::score::ScoreFile;

    fn new_engine(scores: &mut ScoreFile) -> Engine<'_> {
        Engine::test_engine(scores)
    }

    #[test]
    fn paddle_stops_after_tap() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Breakout::new();
        let x0 = g.px;
        // 点按一次左键：挡板短暂移动
        g.handle(Action::Left, &mut eng);
        g.update(0.05, &mut eng);
        assert!(g.px < x0, "点按左键应使挡板左移");
        // 窗口过期后（无释放事件）挡板必须停下
        let x1 = g.px;
        g.update(0.3, &mut eng);
        assert_eq!(g.px, x1, "窗口过期后挡板不应继续漂移");
    }

    #[test]
    fn bottom_row_bricks_are_destroyable() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Breakout::new();
        // 清空所有砖块，只留最底行中间一块
        for r in 0..BRICK_ROWS {
            for c in 0..BRICK_COLS {
                g.bricks[r][c] = false;
            }
        }
        g.bricks[BRICK_ROWS - 1][4] = true;
        g.bricks_left = 1;
        // 球位于底行砖块内部（y=6.5，处于 6..7 区间）
        g.ball = Some((20.0, 6.5, 0.0, 1.0));
        g.ready = false;
        g.update(0.05, &mut eng);
        assert_eq!(g.bricks_left, 0, "底行砖块应能被击碎");
        assert!(g.over, "击碎最后一块砖后游戏应结束");
    }

    #[test]
    fn paddle_moves_while_held() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Breakout::new();
        let x0 = g.px;
        // 模拟按住：持续收到按键事件刷新窗口
        for _ in 0..10 {
            g.handle(Action::Right, &mut eng);
            g.update(0.05, &mut eng);
        }
        assert!(g.px > x0 + 3.0, "按住右键应持续右移");
    }

    #[test]
    fn pack_drop_rate_is_1_20() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(11);
        let g = Breakout::new();
        let n = 10000;
        let mut packs = 0;
        for _ in 0..n {
            if g.roll_pack(&mut rng) {
                packs += 1;
            }
        }
        // 期望 500 次(1/20), 允许 ±150
        assert!(
            (350..=650).contains(&packs),
            "医疗包掉率应约 1/20, 实际 {packs}/{n}"
        );
    }

    #[test]
    fn paddle_catches_pack_and_heals() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Breakout::new();
        g.lives = 1;
        g.packs.push((g.px + PADDLE_W / 2.0, PADDLE_Y));
        g.update(0.1, &mut eng);
        assert_eq!(g.lives, 2, "挡板接住医疗包应回一条命");
        assert!(g.packs.is_empty(), "医疗包应被消耗");
        assert!(g.hint.is_some(), "应有拾取提示");
        // 上限 3 颗心
        g.lives = 3;
        g.packs.push((g.px + PADDLE_W / 2.0, PADDLE_Y));
        g.update(0.1, &mut eng);
        assert_eq!(g.lives, 3, "生命不能超过 3 颗心");
        assert!(g.packs.is_empty(), "满血时医疗包也应被接住");
    }

    #[test]
    fn pack_falls_past_bottom() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Breakout::new();
        g.lives = 2;
        g.packs.push((10.0, FIELD_H as f64 + 0.5));
        g.update(0.5, &mut eng);
        assert!(g.packs.is_empty(), "医疗包应掉出屏幕");
        assert_eq!(g.lives, 2, "没接住不影响生命");
    }
}

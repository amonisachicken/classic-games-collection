//! 飞机大战：移动射击，躲避敌机与敌弹，三命。

use crate::app::Engine;
use crate::canvas::{col, str_width, Canvas};
use crate::games::{Game, GameOutcome, Status};
use crate::input::Action;
use crate::score::ScoreFile;
use rand::Rng;

const FW: f64 = 48.0; // 战场宽
const FH: f64 = 20.0; // 战场高
const MAX_LIVES: i32 = 3;

#[derive(Clone, Copy)]
struct Enemy {
    x: f64,
    y: f64,
    hp: i32,
    kind: u8, // 0 普通 1 快速 2 重型
    speed: f64,
    shoot_acc: f64,
}

pub struct Plane {
    px: f64,
    py: f64,
    /// 各方向最后一次按键事件的时间（窗口期判定按住）
    left_since: f64,
    right_since: f64,
    up_since: f64,
    down_since: f64,
    space_since: f64,
    bullets: Vec<(f64, f64)>,
    ebullets: Vec<(f64, f64)>,
    enemies: Vec<Enemy>,
    explosions: Vec<(f64, f64, f64)>,
    shoot_acc: f64,
    spawn_acc: f64,
    lives: i32,
    score: u32,
    invuln: f64,
    time: f64,
    over: bool,
}

impl Plane {
    pub fn new() -> Self {
        Plane {
            px: FW / 2.0,
            py: FH - 2.0,
            left_since: -1000.0,
            right_since: -1000.0,
            up_since: -1000.0,
            down_since: -1000.0,
            space_since: -1000.0,
            bullets: Vec::new(),
            ebullets: Vec::new(),
            enemies: Vec::new(),
            explosions: Vec::new(),
            shoot_acc: 0.0,
            spawn_acc: 1.0,
            lives: MAX_LIVES,
            score: 0,
            invuln: 0.0,
            time: 0.0,
            over: false,
        }
    }

    fn move_player(&mut self, dt: f64) {
        // 按键窗口期：按住时终端重复事件持续刷新；松开后窗口过期自动停止。
        const HOLD_WINDOW: f64 = 0.15;
        let mut dx = 0.0f64;
        let mut dy = 0.0f64;
        if self.time - self.right_since < HOLD_WINDOW {
            dx += 1.0;
        }
        if self.time - self.left_since < HOLD_WINDOW {
            dx -= 1.0;
        }
        if self.time - self.down_since < HOLD_WINDOW {
            dy += 1.0;
        }
        if self.time - self.up_since < HOLD_WINDOW {
            dy -= 1.0;
        }
        if dx != 0.0 && dy != 0.0 {
            let inv = std::f64::consts::FRAC_1_SQRT_2;
            dx *= inv;
            dy *= inv;
        }
        let sp = 22.0;
        self.px = (self.px + dx * sp * dt).clamp(1.0, FW - 2.0);
        self.py = (self.py + dy * sp * dt).clamp(1.0, FH - 2.0);
    }

    fn fire(&mut self) {
        if self.bullets.len() < 12 {
            self.bullets.push((self.px, self.py - 1.0));
        }
    }

    fn spawn_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(1.0..FW - 1.0);
        let r: f64 = rng.gen_range(0.0..1.0);
        let (kind, hp, speed) = if r < 0.7 {
            (0u8, 1, (5.0 + self.score as f64 * 0.002).min(9.0))
        } else if r < 0.85 {
            (1u8, 1, 13.0)
        } else {
            (2u8, 3, 3.5)
        };
        self.enemies.push(Enemy {
            x,
            y: 1.0,
            hp,
            kind,
            speed,
            shoot_acc: rng.gen_range(1.0..3.0),
        });
    }

    fn explode(&mut self, x: f64, y: f64) {
        self.explosions.push((x, y, 0.35));
    }

    fn hit_player(&mut self) -> bool {
        if self.invuln > 0.0 {
            return false;
        }
        self.lives -= 1;
        self.invuln = 1.3;
        if self.lives <= 0 {
            self.over = true;
        }
        true
    }
}

impl Game for Plane {
    fn update(&mut self, dt: f64, _eng: &mut Engine) {
        if self.over {
            return;
        }
        self.time += dt;
        if self.invuln > 0.0 {
            self.invuln -= dt;
        }
        self.move_player(dt);

        // 玩家射击：空格按下后窗口期内持续开火，松开（无释放事件）后自动停止
        const SHOOT_WINDOW: f64 = 0.18;
        if self.time - self.space_since < SHOOT_WINDOW {
            self.shoot_acc += dt;
            if self.shoot_acc >= 0.16 {
                self.shoot_acc = 0.0;
                self.fire();
            }
        }
        // 子弹
        for b in self.bullets.iter_mut() {
            b.1 -= 26.0 * dt;
        }
        self.bullets.retain(|b| b.1 >= 0.0);

        // 敌机生成
        self.spawn_acc += dt;
        let interval = (1.7 - self.score as f64 * 0.0008).max(0.45);
        if self.spawn_acc >= interval {
            self.spawn_acc = 0.0;
            self.spawn_enemy();
        }
        // 敌机移动/射击
        for e in self.enemies.iter_mut() {
            e.y += e.speed * dt;
            if e.kind != 2 {
                e.shoot_acc += dt;
                if e.shoot_acc >= 2.6 && e.y > 3.0 {
                    e.shoot_acc = 0.0;
                    self.ebullets.push((e.x, e.y + 1.0));
                }
            }
        }
        // 敌弹
        for b in self.ebullets.iter_mut() {
            b.1 += 14.0 * dt;
        }
        self.ebullets.retain(|b| b.1 <= FH + 1.0);

        // 子弹 vs 敌机
        let mut hit: Vec<usize> = Vec::new();
        for (bi, &(bx, by)) in self.bullets.iter().enumerate() {
            for e in self.enemies.iter() {
                if (bx - e.x).abs() < 1.0 && (by - e.y).abs() < 1.0 {
                    hit.push(bi);
                    break;
                }
            }
        }
        // 从后往前移除命中的子弹，并结算敌机
        let mut removed_bullets: Vec<usize> = hit;
        removed_bullets.sort_unstable();
        removed_bullets.dedup();
        for &bi in removed_bullets.iter().rev() {
            if bi < self.bullets.len() {
                self.bullets.remove(bi);
            }
        }
        let mut i = 0;
        while i < self.enemies.len() {
            let e = self.enemies[i];
            let mut was_hit = false;
            for &(bx, by) in &self.bullets {
                if (bx - e.x).abs() < 1.0 && (by - e.y).abs() < 1.0 {
                    was_hit = true;
                    break;
                }
            }
            if was_hit {
                self.enemies[i].hp -= 1;
                if self.enemies[i].hp <= 0 {
                    let pts = match e.kind {
                        0 => 10,
                        1 => 15,
                        _ => 30,
                    };
                    self.score += pts;
                    self.explode(e.x, e.y);
                    self.enemies.remove(i);
                    continue;
                }
            }
            i += 1;
        }

        // 敌机 vs 玩家
        let mut player_hit = false;
        let mut remove: Vec<usize> = Vec::new();
        for (ei, e) in self.enemies.iter().enumerate() {
            if (e.x - self.px).abs() < 0.9 && (e.y - self.py).abs() < 0.9 {
                player_hit = true;
                remove.push(ei);
            }
        }
        // 敌弹 vs 玩家
        let mut ebr: Vec<usize> = Vec::new();
        for (bi, &(bx, by)) in self.ebullets.iter().enumerate() {
            if (bx - self.px).abs() < 0.6 && (by - self.py).abs() < 0.8 {
                player_hit = true;
                ebr.push(bi);
            }
        }
        if player_hit && self.hit_player() {
            for &ei in remove.iter().rev() {
                if ei < self.enemies.len() {
                    let e = self.enemies[ei];
                    self.explode(e.x, e.y);
                    self.enemies.remove(ei);
                }
            }
            for &bi in ebr.iter().rev() {
                if bi < self.ebullets.len() {
                    self.ebullets.remove(bi);
                }
            }
            if self.over {
                return;
            }
        }

        // 敌机飞出底部（炸弹命中基地）
        let mut bottom: Vec<usize> = Vec::new();
        for (ei, e) in self.enemies.iter().enumerate() {
            if e.y > FH - 1.0 {
                bottom.push(ei);
            }
        }
        for &ei in bottom.iter().rev() {
            if ei < self.enemies.len() {
                let e = self.enemies[ei];
                self.explode(e.x, FH - 1.0);
                self.enemies.remove(ei);
                if !self.over {
                    self.lives -= 1;
                    self.invuln = 1.3;
                    if self.lives <= 0 {
                        self.over = true;
                        return;
                    }
                }
            }
        }

        // 爆炸动画
        for e in self.explosions.iter_mut() {
            e.2 -= dt;
        }
        self.explosions.retain(|e| e.2 > 0.0);
    }

    fn handle(&mut self, a: Action, _eng: &mut Engine) {
        if self.over {
            return;
        }
        match a {
            Action::Left => self.left_since = self.time,
            Action::Right => self.right_since = self.time,
            Action::Up => self.up_since = self.time,
            Action::Down => self.down_since = self.time,
            Action::Space => {
                // 新一次按压（距上次按压足够久）预充能，保证立即开火
                if self.time - self.space_since > 0.3 {
                    self.shoot_acc = 0.16;
                }
                self.space_since = self.time;
            }
            _ => {}
        }
    }

    fn draw(&self, c: &mut Canvas, scores: &ScoreFile, _user: &str) {
        c.fill_rect(0, 0, c.w, c.h, ' ', col::BLACK, col::BLACK);
        let ox = c.w.saturating_sub(FW as usize) / 2;
        let oy = c.h.saturating_sub(FH as usize) / 2;
        // 边框
        c.border(ox, oy, FW as usize, FH as usize, col::CYAN);
        // 玩家（无敌时闪烁）
        let blink = self.invuln > 0.0 && ((self.time * 8.0) as u32) % 2 == 0;
        if !blink {
            c.put(ox + self.px as usize, oy + self.py as usize, '^', col::GREEN, col::BLACK);
        }
        // 子弹
        for &(x, y) in &self.bullets {
            c.put(ox + x as usize, oy + y as usize, '|', col::YELLOW, col::BLACK);
        }
        // 敌弹
        for &(x, y) in &self.ebullets {
            c.put(ox + x as usize, oy + y as usize, '!', col::RED, col::BLACK);
        }
        // 敌机
        for e in &self.enemies {
            let (ch, fg) = match e.kind {
                0 => ('v', col::RED),
                1 => ('V', col::MAGENTA),
                _ => ('M', col::PURPLE),
            };
            c.put(ox + e.x as usize, oy + e.y as usize, ch, fg, col::BLACK);
        }
        // 爆炸
        for &(x, y, ttl) in &self.explosions {
            let ch = if ttl > 0.2 { '*' } else { '+' };
            c.put(ox + x as usize, oy + y as usize, ch, col::ORANGE, col::BLACK);
        }

        // 面板
        let title = "飞机大战 PLANE";
        let score = format!("得分 {}", self.score);
        let lives = format!("生命 {}", "♥".repeat(self.lives.max(0) as usize));
        let high = scores
            .get_vec("plane")
            .first()
            .map(|e| format!("最高 {} ({})", e.score, e.user))
            .unwrap_or_else(|| "暂无纪录".to_string());
        let py = oy.saturating_sub(2);
        c.put_str(ox, py, title, col::YELLOW, col::BLACK);
        c.put_str(ox, py + 1, &score, col::GREEN, col::BLACK);
        c.put_str(ox + str_width(&score) + 2, py + 1, &lives, col::RED, col::BLACK);
        let hx = ox + FW as usize - str_width(&high);
        c.put_str(hx, py, &high, col::GRAY, col::BLACK);

        let help = "方向键/HJKL 移动    按住空格 射击    ESC/Q 暂停";
        c.put_str(ox, oy + FH as usize + 1, help, col::GRAY, col::BLACK);
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
    fn plane_stops_after_tap() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        let y0 = g.py;
        g.handle(Action::Up, &mut eng);
        g.update(0.05, &mut eng);
        assert!(g.py < y0, "点按上键应使飞机上移");
        let y1 = g.py;
        g.update(0.3, &mut eng);
        assert_eq!(g.py, y1, "窗口过期后飞机不应继续漂移");
    }

    #[test]
    fn shooting_works_per_tap() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        let n0 = g.bullets.len();
        // 点按空格：预充能，立即开火一发
        g.handle(Action::Space, &mut eng);
        g.update(0.05, &mut eng);
        assert_eq!(g.bullets.len(), n0 + 1, "点按空格应立即开火一发");
        // 松开（无释放事件）：窗口过期后不再追加子弹
        g.update(0.5, &mut eng);
        let n1 = g.bullets.len();
        assert!(n1 <= n0 + 1, "松手后不应继续开火");
        // 再次点按又能开火
        g.handle(Action::Space, &mut eng);
        g.update(0.05, &mut eng);
        assert!(g.bullets.len() > n1, "再次点按应能开火");
    }
}

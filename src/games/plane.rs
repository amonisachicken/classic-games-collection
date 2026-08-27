//! 飞机大战：移动射击，躲避敌机与敌弹，三命。

use crate::app::Engine;
use crate::canvas::{col, str_width, Canvas};
use crate::games::{Game, GameOutcome, Status};
use crate::input::Action;
use crate::score::ScoreFile;
use crossterm::style::Color;
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
    /// 医疗包 (x, y)
    packs: Vec<(f64, f64)>,
    /// 爆炸特效 (x, y, 剩余时间, 颜色)
    explosions: Vec<(f64, f64, f64, Color)>,
    /// 屏幕提示 (剩余时间, 文本)
    hint: Option<(f64, String)>,
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
            packs: Vec::new(),
            explosions: Vec::new(),
            hint: None,
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
        self.explosions.push((x, y, 0.35, col::ORANGE));
    }

    /// 医疗包掉落判定：概率为敌机的 1/20。
    fn roll_pack(&self, rng: &mut impl Rng) -> bool {
        rng.gen_range(0.0..1.0) < 0.05
    }

    fn spawn_pack(&mut self, rng: &mut impl Rng) {
        let x = rng.gen_range(1.0..FW - 1.0);
        self.packs.push((x, 1.0));
    }

    /// 拾取医疗包：恢复一条生命（上限 MAX_LIVES）。
    fn collect_pack(&mut self, i: usize) {
        let (x, y) = self.packs.remove(i);
        if self.lives < MAX_LIVES {
            self.lives += 1;
            self.hint = Some((0.9, "♥ 生命 +1".to_string()));
            self.explosions.push((x, y, 0.35, col::GREEN));
        } else {
            self.hint = Some((0.9, "生命已满".to_string()));
        }
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
            // 医疗包伴随出现：概率为敌机的 1/20
            let mut rng = rand::thread_rng();
            if self.roll_pack(&mut rng) {
                self.spawn_pack(&mut rng);
            }
        }
        // 医疗包向下飞行（迎面而来）
        const PACK_SPEED: f64 = 4.0;
        for pk in self.packs.iter_mut() {
            pk.1 += PACK_SPEED * dt;
        }
        self.packs.retain(|pk| pk.1 <= FH + 1.0);
        // 拾取医疗包
        let mut collect: Vec<usize> = Vec::new();
        for (pi, &(x, y)) in self.packs.iter().enumerate() {
            if (x - self.px).abs() < 0.9 && (y - self.py).abs() < 0.9 {
                collect.push(pi);
            }
        }
        for &pi in collect.iter().rev() {
            if pi < self.packs.len() {
                self.collect_pack(pi);
            }
        }
        // 提示文字倒计时
        if let Some((ttl, _)) = &mut self.hint {
            *ttl -= dt;
            if *ttl <= 0.0 {
                self.hint = None;
            }
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

        // 子弹 vs 敌机：一发子弹命中一架敌机，立即结算伤害
        let mut hit_bullets: Vec<usize> = Vec::new();
        for (bi, &(bx, by)) in self.bullets.iter().enumerate() {
            let mut ei = 0;
            while ei < self.enemies.len() {
                let e = self.enemies[ei];
                if (bx - e.x).abs() < 1.0 && (by - e.y).abs() < 1.0 {
                    hit_bullets.push(bi);
                    self.enemies[ei].hp -= 1;
                    if self.enemies[ei].hp <= 0 {
                        let pts = match e.kind {
                            0 => 10,
                            1 => 15,
                            _ => 30,
                        };
                        self.score += pts;
                        self.explosions.push((e.x, e.y, 0.35, col::ORANGE));
                        self.enemies.remove(ei);
                    }
                    break;
                }
                ei += 1;
            }
        }
        // 移除已命中的子弹
        if !hit_bullets.is_empty() {
            hit_bullets.sort_unstable();
            hit_bullets.dedup();
            for &bi in hit_bullets.iter().rev() {
                if bi < self.bullets.len() {
                    self.bullets.remove(bi);
                }
            }
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

        // 敌机飞出底部：直接离场，不扣生命（只有被敌弹击中或与敌机相撞才受伤）
        self.enemies.retain(|e| e.y <= FH - 1.0);

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
        for &(x, y, ttl, color) in &self.explosions {
            let ch = if ttl > 0.2 { '*' } else { '+' };
            c.put(ox + x as usize, oy + y as usize, ch, color, col::BLACK);
        }
        // 医疗包
        for &(x, y) in &self.packs {
            c.put(ox + x as usize, oy + y as usize, '♥', col::PINK, col::BLACK);
        }
        // 拾取提示（显示在玩家上方）
        if let Some((ttl, text)) = &self.hint {
            if *ttl > 0.0 {
                let sx = ox + self.px as usize;
                let sy = (oy + self.py as usize).saturating_sub(1);
                let tx = sx.saturating_sub(str_width(text) / 2);
                c.put_str(tx, sy, text, col::GREEN, col::BLACK);
            }
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

        let help = "方向键/HJKL 移动    按住空格 射击    ♥医疗包+1命    ESC/Q 暂停";
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

    #[test]
    fn enemy_exiting_bottom_does_not_damage() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        g.lives = 3;
        g.spawn_acc = 0.0; // 测试期间不生成新敌机
        // 放一架即将飞出底部的敌机（没碰到玩家、没开火）
        g.enemies.push(Enemy {
            x: 10.0,
            y: FH - 0.5,
            hp: 1,
            kind: 0,
            speed: 2.0,
            shoot_acc: 100.0, // 不射击
        });
        g.update(1.0, &mut eng); // 敌机越过底部
        assert_eq!(g.lives, 3, "敌机飞出底部不应扣生命");
        assert!(g.enemies.is_empty(), "飞出底部的敌机应离场");
        assert!(!g.over, "不应因此游戏结束");
    }

    #[test]
    fn only_collision_or_bullet_damages() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        g.lives = 3;
        g.spawn_acc = 0.0;
        // 敌弹击中玩家 → 扣命
        g.ebullets.push((g.px, g.py - 1.0));
        g.update(0.1, &mut eng);
        assert_eq!(g.lives, 2, "被敌弹击中应扣命");
        // 敌机相撞 → 扣命
        g.enemies.push(Enemy {
            x: g.px,
            y: g.py,
            hp: 1,
            kind: 0,
            speed: 0.0,
            shoot_acc: 100.0,
        });
        g.invuln = 0.0;
        g.update(0.05, &mut eng);
        assert_eq!(g.lives, 1, "与敌机相撞应扣命");
    }

    #[test]
    fn bullets_damage_and_kill_enemies() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        g.spawn_acc = 0.0;
        // 普通敌机 hp=1：一颗子弹击落
        g.enemies.push(Enemy {
            x: 20.0,
            y: 3.0,
            hp: 1,
            kind: 0,
            speed: 0.0,
            shoot_acc: 0.0,
        });
        g.bullets.push((20.0, 4.0)); // 子弹从敌机下方飞入
        g.update(0.01, &mut eng);
        assert!(g.enemies.is_empty(), "普通敌机应被一颗子弹击落");
        assert_eq!(g.score, 10, "击落普通敌机得分 10");
        assert!(g.bullets.is_empty(), "命中的子弹应被消耗");

        // 重型敌机 hp=3：需要三颗子弹
        g.enemies.push(Enemy {
            x: 30.0,
            y: 3.0,
            hp: 3,
            kind: 2,
            speed: 0.0,
            shoot_acc: 0.0,
        });
        g.bullets.push((30.0, 4.0));
        g.update(0.01, &mut eng);
        assert_eq!(g.enemies.len(), 1, "第一发后重型敌机仍在");
        assert_eq!(g.enemies[0].hp, 2, "重型敌机中一弹后剩 2 血");
        g.bullets.push((30.0, 4.0));
        g.update(0.01, &mut eng);
        g.bullets.push((30.0, 4.0));
        g.update(0.01, &mut eng);
        assert!(g.enemies.is_empty(), "三颗子弹应击落重型敌机");
        assert_eq!(g.score, 40, "累计得分 10+30");
    }

    #[test]
    fn pack_spawn_rate_is_1_20() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(7);
        let g = Plane::new();
        let n = 10000;
        let mut packs = 0;
        for _ in 0..n {
            if g.roll_pack(&mut rng) {
                packs += 1;
            }
        }
        // 期望 500 次（1/20），允许 ±150
        assert!(
            (350..=650).contains(&packs),
            "医疗包概率应约 1/20, 实际 {packs}/{n}"
        );
    }

    #[test]
    fn pack_collect_heals() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        g.lives = 1;
        g.spawn_acc = 0.0;
        // 医疗包与玩家重合
        g.packs.push((g.px, g.py));
        g.update(0.05, &mut eng);
        assert_eq!(g.lives, 2, "吃掉医疗包应恢复一条生命");
        assert!(g.packs.is_empty(), "医疗包应被消耗");
        assert!(g.hint.is_some(), "应有拾取提示");
        // 上限 3 条
        g.lives = 3;
        g.packs.push((g.px, g.py));
        g.update(0.05, &mut eng);
        assert_eq!(g.lives, 3, "生命已满不应超过上限");
        assert!(g.packs.is_empty(), "满血时医疗包也应被吃掉");
    }

    #[test]
    fn pack_flies_off_bottom() {
        let mut scores = ScoreFile::default();
        let mut eng = new_engine(&mut scores);
        let mut g = Plane::new();
        g.lives = 2;
        g.spawn_acc = 0.0;
        g.packs.push((5.0, FH - 0.5));
        g.update(1.0, &mut eng); // 医疗包飞出底部
        assert!(g.packs.is_empty(), "医疗包应飞出屏幕");
        assert_eq!(g.lives, 2, "医疗包离场不影响生命");
    }
}

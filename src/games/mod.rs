//! 五款游戏：贪吃蛇、打砖块、俄罗斯方块、飞机大战、五子棋。

pub mod breakout;
pub mod gomoku;
pub mod plane;
pub mod snake;
pub mod tetris;

use crate::app::Engine;
use crate::canvas::Canvas;
use crate::input::Action;
use crate::lang::{self, Lang};
use crate::score::ScoreFile;

/// 游戏进行状态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Running,
    Finished,
}

/// 游戏结束后的结果（用于记分与结算界面）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameOutcome {
    /// 普通分数游戏，结束分数。
    Score(u32),
    /// 五子棋：胜/负。
    Gomoku { win: bool },
    /// 五子棋：平局（不记胜负）。
    GomokuDraw,
    /// 中途退出（不记分）。
    Quit,
}

/// 游戏 ID，也是主菜单的可选项。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameId {
    Snake,
    Breakout,
    Tetris,
    Plane,
    Gomoku,
}

impl GameId {
    pub const ALL: [GameId; 5] = [
        GameId::Snake,
        GameId::Breakout,
        GameId::Tetris,
        GameId::Plane,
        GameId::Gomoku,
    ];

    pub fn name(&self) -> &'static str {
        let u = lang::ui();
        match self {
            GameId::Snake => u.game_snake,
            GameId::Breakout => u.game_breakout,
            GameId::Tetris => u.game_tetris,
            GameId::Plane => u.game_plane,
            GameId::Gomoku => u.game_gomoku,
        }
    }

    /// 菜单/标题用的显示名：中文界面显示"贪吃蛇 (Snake)"，英文界面显示"Snake"。
    pub fn display_name(&self) -> String {
        if lang::lang() == Lang::Zh {
            format!("{} ({})", self.name(), self.name_en())
        } else {
            self.name_en().to_string()
        }
    }

    pub fn name_en(&self) -> &'static str {
        match self {
            GameId::Snake => "Snake",
            GameId::Breakout => "Breakout",
            GameId::Tetris => "Tetris",
            GameId::Plane => "Plane Battle",
            GameId::Gomoku => "Gomoku",
        }
    }

    /// scores.json 中的键名。
    pub fn score_key(&self) -> &'static str {
        match self {
            GameId::Snake => "snake",
            GameId::Breakout => "breakout",
            GameId::Tetris => "tetris",
            GameId::Plane => "plane",
            GameId::Gomoku => "gomoku",
        }
    }

    /// 一句游戏简介。
    pub fn desc(&self) -> &'static str {
        let u = lang::ui();
        match self {
            GameId::Snake => u.desc_snake,
            GameId::Breakout => u.desc_breakout,
            GameId::Tetris => u.desc_tetris,
            GameId::Plane => u.desc_plane,
            GameId::Gomoku => u.desc_gomoku,
        }
    }

    pub fn new_game(&self) -> Box<dyn Game> {
        match self {
            GameId::Snake => Box::new(snake::Snake::new()),
            GameId::Breakout => Box::new(breakout::Breakout::new()),
            GameId::Tetris => Box::new(tetris::Tetris::new()),
            GameId::Plane => Box::new(plane::Plane::new()),
            GameId::Gomoku => Box::new(gomoku::Gomoku::new()),
        }
    }
}

/// 每款游戏实现的接口。
pub trait Game {
    /// 逻辑更新（固定步长调用；回合制游戏可为空操作）。
    fn update(&mut self, dt: f64, eng: &mut Engine);
    /// 处理输入动作。
    fn handle(&mut self, a: Action, eng: &mut Engine);
    /// 绘制到画布（可读取排行榜数据与当前用户名用于显示最高分）。
    fn draw(&self, c: &mut Canvas, scores: &ScoreFile, user: &str);
    /// 当前状态。
    fn status(&self) -> Status;
    /// 结束时的结果（仅在 Finished 时有意义）。
    fn outcome(&self) -> GameOutcome {
        GameOutcome::Quit
    }
}

//! 分数 / 用户名持久化：存放在家目录 `~/.classic-games-collection/scores.json`。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

/// 游戏标识（用于 scores.json 的键名与显示）。
pub const GAME_KEYS: [&str; 5] = ["snake", "breakout", "tetris", "plane", "gomoku"];


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entry {
    pub user: String,
    pub score: u32,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct GomokuStats {
    pub wins: u32,
    pub losses: u32,
    pub last: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScoreFile {
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub snake: Vec<Entry>,
    #[serde(default)]
    pub breakout: Vec<Entry>,
    #[serde(default)]
    pub tetris: Vec<Entry>,
    #[serde(default)]
    pub plane: Vec<Entry>,
    #[serde(default)]
    pub gomoku: BTreeMap<String, GomokuStats>,
}

impl Default for ScoreFile {
    fn default() -> Self {
        ScoreFile {
            user: String::new(),
            snake: Vec::new(),
            breakout: Vec::new(),
            tetris: Vec::new(),
            plane: Vec::new(),
            gomoku: BTreeMap::new(),
        }
    }
}

/// 最高分榜单保留条数。
pub const TOP_N: usize = 10;

impl ScoreFile {
    /// 数据文件路径：~/.classic-games-collection/scores.json
    pub fn path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".classic-games-collection").join("scores.json"))
    }

    pub fn load() -> Self {
        match Self::path() {
            Some(p) => match fs::read_to_string(&p) {
                Ok(s) => match serde_json::from_str::<ScoreFile>(&s) {
                    Ok(mut f) => {
                        // 容错：旧文件缺字段时补默认值
                        f.normalize();
                        f
                    }
                    Err(_) => ScoreFile::default(),
                },
                Err(_) => ScoreFile::default(),
            },
            None => ScoreFile::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = match Self::path() {
            Some(p) => p,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "no home dir")),
        };
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        // 先写临时文件再改名，避免写坏中断。
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// 补全缺失字段（向前兼容）。
    fn normalize(&mut self) {
        for k in GAME_KEYS {
            if k != "gomoku" {
                let v = self.get_vec_mut(k);
                v.retain(|e| !e.user.trim().is_empty());
                v.sort_by(|a, b| b.score.cmp(&a.score));
                v.truncate(TOP_N);
            }
        }
    }

    fn get_vec_mut(&mut self, key: &str) -> &mut Vec<Entry> {
        match key {
            "snake" => &mut self.snake,
            "breakout" => &mut self.breakout,
            "tetris" => &mut self.tetris,
            "plane" => &mut self.plane,
            _ => &mut self.snake,
        }
    }

    pub fn get_vec(&self, key: &str) -> &[Entry] {
        match key {
            "snake" => &self.snake,
            "breakout" => &self.breakout,
            "tetris" => &self.tetris,
            "plane" => &self.plane,
            _ => &self.snake,
        }
    }

    /// 记录一局分数：插入榜单、排序、截断。
    pub fn add_score(&mut self, key: &str, user: &str, score: u32) {
        if key == "gomoku" || score == 0 {
            return;
        }
        let v = self.get_vec_mut(key);
        v.push(Entry {
            user: user.to_string(),
            score,
            date: now_string(),
        });
        v.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.date.cmp(&b.date)));
        v.truncate(TOP_N);
    }

    /// 当前分数在榜单中的名次（1 开始；未上榜返回 None）。
    pub fn rank_of(&self, key: &str, score: u32) -> Option<usize> {
        self.get_vec(key)
            .iter()
            .position(|e| e.score == score)
            .map(|i| i + 1)
    }

    /// 该游戏当前用户是否刷新了第一名。
    pub fn is_new_best(&self, key: &str, score: u32) -> bool {
        score > 0 && self.get_vec(key).first().map(|e| e.score == score).unwrap_or(false)
    }

    /// 五子棋胜负统计。
    pub fn gomoku_stats(&self, user: &str) -> GomokuStats {
        self.gomoku.get(user).cloned().unwrap_or_default()
    }

    /// 记录一局五子棋胜负。
    pub fn add_gomoku(&mut self, user: &str, win: bool) {
        let s = self.gomoku.entry(user.to_string()).or_default();
        if win {
            s.wins += 1;
        } else {
            s.losses += 1;
        }
        s.last = now_string();
    }
}

/// 生成 `YYYY-MM-DD HH:MM` 格式的当前时间（不依赖 chrono）。
pub fn now_string() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86400) as i64;
    let sod = secs % 86400;
    let (y, m, d) = civil_from_days(days);
    let (hh, mm) = (sod / 3600, (sod % 3600) / 60);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, d, hh, mm)
}

/// Howard Hinnant 的 civil_from_days 算法：天数 → (年,月,日)。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// 系统用户名（用于默认用户名）。
pub fn system_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "player".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_format() {
        let s = now_string();
        assert_eq!(s.len(), 16);
        assert_eq!(&s[4..5], "-");
    }

    #[test]
    fn civil_days_known() {
        // 1970-01-01
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01
        assert_eq!(civil_from_days(11017), (2000, 3, 1));
        // 2024-02-29 (闰年)
        assert_eq!(civil_from_days(19782), (2024, 2, 29));
        assert_eq!(civil_from_days(19783), (2024, 3, 1));
    }

    #[test]
    fn add_and_sort() {
        let mut f = ScoreFile::default();
        f.user = "alice".into();
        f.add_score("snake", "alice", 30);
        f.add_score("snake", "bob", 100);
        f.add_score("snake", "alice", 50);
        let v = f.get_vec("snake");
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].score, 100);
        assert_eq!(v[1].score, 50);
        assert_eq!(v[2].score, 30);
        assert_eq!(f.rank_of("snake", 50), Some(2));
    }

    #[test]
    fn gomoku_stats_work() {
        let mut f = ScoreFile::default();
        f.add_gomoku("alice", true);
        f.add_gomoku("alice", true);
        f.add_gomoku("alice", false);
        let s = f.gomoku_stats("alice");
        assert_eq!((s.wins, s.losses), (2, 1));
    }

    #[test]
    fn save_load_roundtrip() {
        let mut f = ScoreFile::default();
        f.user = "tester".into();
        f.add_score("tetris", "tester", 42);
        f.add_gomoku("tester", true);
        // 用临时路径测试序列化本身
        let json = serde_json::to_string(&f).unwrap();
        let back: ScoreFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.user, "tester");
        assert_eq!(back.tetris.len(), 1);
        assert_eq!(back.gomoku_stats("tester").wins, 1);
    }
}

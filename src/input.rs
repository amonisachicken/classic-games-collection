//! 按键归一化：方向键 + HJKL、空格、回车、ESC/Q 统一映射为动作。

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// 统一操作动作。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    /// 空格
    Space,
    /// 回车
    Confirm,
    /// ESC 或 Q
    Cancel,
    /// 其他字符键
    Char(char),
    Backspace,
    // ---- 释放事件（用于按住连发控制） ----
    ReleaseUp,
    ReleaseDown,
    ReleaseLeft,
    ReleaseRight,
    ReleaseSpace,
}

/// 把 crossterm 按键事件映射为统一动作。
/// 按下与重复映射为普通动作，释放映射为 Release* 动作。
pub fn map_key(k: KeyEvent) -> Option<Action> {
    if k.kind == KeyEventKind::Release {
        return match k.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Some(Action::ReleaseUp),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Some(Action::ReleaseDown),
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => Some(Action::ReleaseLeft),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => Some(Action::ReleaseRight),
            KeyCode::Char(' ') => Some(Action::ReleaseSpace),
            _ => None,
        };
    }
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        // Ctrl+C 让用户能强制退出
        if k.code == KeyCode::Char('c') || k.code == KeyCode::Char('C') {
            return None; // 交给调用方处理（结束程序）
        }
    }
    match k.code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Some(Action::Down),
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => Some(Action::Left),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => Some(Action::Right),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Char(' ') => Some(Action::Space),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Cancel),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Char(c) => Some(Action::Char(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn arrows_and_hjkl() {
        assert_eq!(map_key(key(KeyCode::Up)), Some(Action::Up));
        assert_eq!(map_key(key(KeyCode::Char('k'))), Some(Action::Up));
        assert_eq!(map_key(key(KeyCode::Char('K'))), Some(Action::Up));
        assert_eq!(map_key(key(KeyCode::Down)), Some(Action::Down));
        assert_eq!(map_key(key(KeyCode::Char('j'))), Some(Action::Down));
        assert_eq!(map_key(key(KeyCode::Left)), Some(Action::Left));
        assert_eq!(map_key(key(KeyCode::Char('h'))), Some(Action::Left));
        assert_eq!(map_key(key(KeyCode::Right)), Some(Action::Right));
        assert_eq!(map_key(key(KeyCode::Char('l'))), Some(Action::Right));
    }

    #[test]
    fn confirm_cancel_space() {
        assert_eq!(map_key(key(KeyCode::Enter)), Some(Action::Confirm));
        assert_eq!(map_key(key(KeyCode::Esc)), Some(Action::Cancel));
        assert_eq!(map_key(key(KeyCode::Char('q'))), Some(Action::Cancel));
        assert_eq!(map_key(key(KeyCode::Char('Q'))), Some(Action::Cancel));
        assert_eq!(map_key(key(KeyCode::Char(' '))), Some(Action::Space));
    }

    #[test]
    fn release_keys() {
        let mut k = key(KeyCode::Char(' '));
        k.kind = KeyEventKind::Release;
        assert_eq!(map_key(k), Some(Action::ReleaseSpace));
        let mut k2 = key(KeyCode::Left);
        k2.kind = KeyEventKind::Release;
        assert_eq!(map_key(k2), Some(Action::ReleaseLeft));
    }
}

//! 终端帧缓冲渲染器。
//!
//! 用一个字符网格作为画布，游戏只负责往画布上画内容，
//! `flush` 时与上一帧做差量比较，只输出变化的区域（行内 RLE 合并），
//! 兼容 XTerm256 色（`Color::AnsiValue`）。

use crossterm::style::Color;
use std::io::{self, Write};

/// 常用 XTerm256 颜色常量。
pub mod col {
    use super::Color;
    pub const BLACK: Color = Color::AnsiValue(0);
    pub const GRAY: Color = Color::AnsiValue(244);
    pub const WHITE: Color = Color::AnsiValue(15);
    pub const RED: Color = Color::AnsiValue(196);
    pub const GREEN: Color = Color::AnsiValue(46);
    pub const DARK_GREEN: Color = Color::AnsiValue(28);
    pub const YELLOW: Color = Color::AnsiValue(226);
    pub const ORANGE: Color = Color::AnsiValue(208);
    pub const BLUE: Color = Color::AnsiValue(27);
    pub const DARK_BLUE: Color = Color::AnsiValue(24);
    pub const MAGENTA: Color = Color::AnsiValue(129);
    pub const PURPLE: Color = Color::AnsiValue(135);
    pub const PINK: Color = Color::AnsiValue(213);
    pub const CYAN: Color = Color::AnsiValue(51);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        fg: Color::Reset,
        bg: Color::Reset,
    };
}

/// 字符画布。`cur` 是当前帧，`prev` 是上一帧（用于差量刷新）。
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    cur: Vec<Vec<Cell>>,
    prev: Vec<Vec<Cell>>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        let w = w.max(1);
        let h = h.max(1);
        let cur = vec![vec![Cell::EMPTY; w]; h];
        let prev = vec![vec![Cell::EMPTY; w]; h];
        Canvas { w, h, cur, prev }
    }

    /// 终端尺寸变化时重建画布（强制整帧重绘）。
    pub fn resize(&mut self, w: usize, h: usize) {
        let w = w.max(1);
        let h = h.max(1);
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        self.cur = vec![vec![Cell::EMPTY; w]; h];
        self.prev = vec![vec![Cell::EMPTY; w]; h];
    }

    /// 整帧清空。
    pub fn clear(&mut self) {
        for row in self.cur.iter_mut() {
            for c in row.iter_mut() {
                *c = Cell::EMPTY;
            }
        }
    }

    /// 强制下一帧整帧重绘（例如游戏切换）。
    pub fn force_redraw(&mut self) {
        self.prev = vec![vec![Cell::EMPTY; self.w]; self.h];
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        if x >= self.w || y >= self.h {
            return;
        }
        self.cur[y][x] = Cell { ch, fg, bg };
    }

    /// 在 (x,y) 处画一个字符，带前景色（背景透明）。
    /// 画字符串，自动处理 CJK 全角字符占两列的问题。
    pub fn put_str(&mut self, x: usize, y: usize, s: &str, fg: Color, bg: Color) {
        let mut cx = x as i64;
        for ch in s.chars() {
            if cx < 0 {
                cx += char_width(ch) as i64;
                continue;
            }
            let cxu = cx as usize;
            if cxu >= self.w {
                break;
            }
            self.put(cxu, y, ch, fg, bg);
            let w = char_width(ch);
            if w == 2 && cxu + 1 < self.w {
                // 全角字符占两个格子，第二个格子用背景色填充，避免残留。
                self.put(cxu + 1, y, ' ', fg, bg);
            }
            cx += w as i64;
        }
    }

    /// 画一个由字符组成的实心矩形。
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, ch: char, fg: Color, bg: Color) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.put(xx, yy, ch, fg, bg);
            }
        }
    }

    /// 画 ASCII 边框（+ - |）。
    pub fn border(&mut self, x: usize, y: usize, w: usize, h: usize, fg: Color) {
        if w == 0 || h == 0 {
            return;
        }
        let xr = x.saturating_add(w - 1).min(self.w - 1);
        let yb = y.saturating_add(h - 1).min(self.h - 1);
        for i in x..=xr {
            self.put(i, y, '-', fg, Color::Reset);
            self.put(i, yb, '-', fg, Color::Reset);
        }
        for j in y..=yb {
            self.put(x, j, '|', fg, Color::Reset);
            self.put(xr, j, '|', fg, Color::Reset);
        }
        self.put(x, y, '+', fg, Color::Reset);
        self.put(xr, y, '+', fg, Color::Reset);
        self.put(x, yb, '+', fg, Color::Reset);
        self.put(xr, yb, '+', fg, Color::Reset);
    }

    /// 差量刷新：只输出与上一帧不同的区域，行内按相同颜色合并为 run。
    pub fn flush(&mut self, out: &mut impl Write) -> io::Result<()> {
        let mut buf = String::with_capacity(self.w * self.h / 3 + 64);
        let mut last_fg: Option<u8> = None;
        let mut last_bg: Option<u8> = None;
        for y in 0..self.h {
            let mut x = 0usize;
            while x < self.w {
                if self.cur[y][x] == self.prev[y][x] {
                    x += 1;
                    continue;
                }
                let cell = self.cur[y][x];
                // 找到相同前景/背景的连续段（无论字符是否变化，重打一遍无害）。
                let mut x2 = x + 1;
                while x2 < self.w
                    && self.cur[y][x2].fg == cell.fg
                    && self.cur[y][x2].bg == cell.bg
                {
                    x2 += 1;
                }
                let (fg8, bg8) = (color_to_ansi(cell.fg), color_to_ansi(cell.bg));
                if last_fg != Some(fg8) {
                    buf.push_str("\x1b[38;5;");
                    buf.push_str(&fg8.to_string());
                    buf.push('m');
                    last_fg = Some(fg8);
                }
                if last_bg != Some(bg8) {
                    buf.push_str("\x1b[48;5;");
                    buf.push_str(&bg8.to_string());
                    buf.push('m');
                    last_bg = Some(bg8);
                }
                buf.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                for c in &self.cur[y][x..x2] {
                    buf.push(c.ch);
                }
                x = x2;
            }
        }
        // 恢复默认颜色，避免残留。
        if last_fg.is_some() || last_bg.is_some() {
            buf.push_str("\x1b[39m\x1b[49m");
        }
        self.prev = self.cur.clone();
        out.write_all(buf.as_bytes())?;
        // 终端下 stdout 为行缓冲，而逐帧输出不含换行符，
        // 必须显式 flush，否则小差异会积压在缓冲区里不显示。
        out.flush()
    }
}

/// 将任意 crossterm Color 归一化为 XTerm256 的 8bit 索引。
pub fn color_to_ansi(c: Color) -> u8 {
    match c {
        Color::Reset => 0, // 用黑色兜底；调用方一般传 Reset 时不需要颜色
        Color::Black => 0,
        Color::DarkGrey => 8,
        Color::Red => 1,
        Color::DarkRed => 52,
        Color::Green => 2,
        Color::DarkGreen => 22,
        Color::Yellow => 3,
        Color::DarkYellow => 94,
        Color::Blue => 4,
        Color::DarkBlue => 18,
        Color::Magenta => 5,
        Color::DarkMagenta => 53,
        Color::Cyan => 6,
        Color::DarkCyan => 23,
        Color::White => 7,
        Color::Grey => 244,
        Color::AnsiValue(n) => n,
        Color::Rgb { r, g, b } => rgb_to_ansi(r, g, b),
    }
}

/// 标准 RGB → XTerm256 立方体近似。
pub fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    let (r, g, b) = (r as u16, g as u16, b as u16);
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + ((r - 8) / 10) as u8;
    }
    let ri = r * 5 / 255;
    let gi = g * 5 / 255;
    let bi = b * 5 / 255;
    (16 + 36 * ri + 6 * gi + bi) as u8
}

/// 估算字符在终端中占用的列宽：CJK 全角按 2，其余按 1。
pub fn char_width(c: char) -> usize {
    if c.is_ascii() {
        return 1;
    }
    let cp = c as u32;
    if matches!(cp,
        0x1100..=0x115F | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF | 0xA000..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6 | 0x20000..=0x2FFFD
        | 0x30000..=0x3FFFD)
    {
        2
    } else {
        1
    }
}

/// 字符串显示宽度（CJK 按 2 计）。
pub fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_cjk() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('中'), 2);
        assert_eq!(str_width("贪吃蛇"), 6);
        assert_eq!(str_width("Snake"), 5);
    }

    #[test]
    fn flush_delta() {
        let mut c = Canvas::new(10, 3);
        let mut out = Vec::new();
        c.put(0, 0, 'A', Color::AnsiValue(1), Color::Reset);
        c.flush(&mut out).unwrap();
        assert!(out.len() > 0);
        // 第二次无变化 → 空输出
        out.clear();
        c.flush(&mut out).unwrap();
        assert_eq!(out.len(), 0);
        // 只改一个格 → 输出包含该坐标
        c.put(5, 2, 'B', Color::AnsiValue(2), Color::Reset);
        out.clear();
        c.flush(&mut out).unwrap();
        assert!(String::from_utf8_lossy(&out).contains("3;6H"));
    }

    #[test]
    fn rgb_nearest() {
        let n = rgb_to_ansi(255, 0, 0);
        assert!(n == 9 || n == 196); // 红
    }
}

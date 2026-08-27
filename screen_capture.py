#!/usr/bin/env python3
"""屏幕捕获与 ANSI 重放：驱动游戏进入指定画面，把输出流重放成文本网格。

用法: python3 screen_capture.py [menu|snake|breakout|tetris|plane|gomoku|pause|summary]
输出: 渲染后的屏幕网格（文本），写入终端。
"""
import os
import pty
import select
import subprocess
import sys
import time
import fcntl
import termios
import struct
import re

BIN = os.path.join(os.path.dirname(os.path.abspath(__file__)), "target", "release", "classic-games-collection")

W, H = 110, 32


class Term:
    """极简终端模拟器：支持我们程序用到的转义序列。"""

    def __init__(self, w, h):
        self.w, self.h = w, h
        self.grid = [[(" ", None, None) for _ in range(w)] for _ in range(h)]
        self.cx, self.cy = 0, 0
        self.fg = None
        self.bg = None

    def feed(self, data):
        text = data.decode("utf-8", "replace")
        i = 0
        n = len(text)
        while i < n:
            ch = text[i]
            if ch == "\x1b":  # ESC
                if i + 1 < n and text[i + 1] == "[":
                    j = i + 2
                    while j < n and not ("@" <= text[j] <= "~"):
                        j += 1
                    if j < n:
                        self.csi(text[i + 2 : j], text[j])
                        i = j + 1
                        continue
                elif i + 1 < n and text[i + 1] == "]":
                    j = i + 2
                    while j < n and text[j] != "\x07":
                        j += 1
                    i = j + 1
                    continue
                i += 1
                continue
            elif ch == "\n" or ch == "\r":
                i += 1
                continue
            else:
                self.put(ch)
                i += 1

    def put(self, ch):
        if 0 <= self.cy < self.h and 0 <= self.cx < self.w:
            self.grid[self.cy][self.cx] = (ch, self.fg, self.bg)
            # 全角字符在终端占 2 列：右半格被覆盖（写入哨兵）
            if is_wide(ch) and self.cx + 1 < self.w:
                self.grid[self.cy][self.cx + 1] = ("\x00", self.fg, self.bg)
        # 全角字符在终端占 2 列
        self.cx += 2 if is_wide(ch) else 1

    def csi(self, params, final):
        if params.startswith("?"):
            return  # 私有模式（如 ?1049h）忽略
        ps = [int(x) if x else 0 for x in params.split(";")] if params else []
        if final == "H" or final == "f":
            r = ps[0] if len(ps) > 0 and ps[0] else 1
            c = ps[1] if len(ps) > 1 and ps[1] else 1
            self.cy, self.cx = r - 1, c - 1
        elif final == "m":
            if not ps:
                ps = [0]
            i = 0
            while i < len(ps):
                v = ps[i]
                if v == 0:
                    self.fg = self.bg = None
                elif v == 38 and i + 2 < len(ps) and ps[i + 1] == 5:
                    self.fg = ps[i + 2]
                    i += 2
                elif v == 48 and i + 2 < len(ps) and ps[i + 1] == 5:
                    self.bg = ps[i + 2]
                    i += 2
                elif v == 39:
                    self.fg = None
                elif v == 49:
                    self.bg = None
                i += 1
        elif final == "J":
            self.grid = [[(" ", None, None) for _ in range(self.w)] for _ in range(self.h)]
        # 其他序列忽略

    def render_text(self):
        # 全角字符占 2 列：打印后跳过其右半格（终端里被字符覆盖）
        lines = []
        for row in self.grid:
            line = ""
            skip = False
            for (ch, fg, bg) in row:
                if ch == "\x00" or skip:
                    skip = False
                    continue
                line += ch
                if is_wide(ch):
                    skip = True
            lines.append(line.rstrip())
        return "\n".join(lines)


def is_wide(ch):
    cp = ord(ch)
    return (
        0x1100 <= cp <= 0x115F
        or 0x2E80 <= cp <= 0x303E
        or 0x3041 <= cp <= 0x33FF
        or 0x3400 <= cp <= 0x4DBF
        or 0x4E00 <= cp <= 0x9FFF
        or 0xA000 <= cp <= 0xA4CF
        or 0xAC00 <= cp <= 0xD7A3
        or 0xF900 <= cp <= 0xFAFF
        or 0xFE30 <= cp <= 0xFE4F
        or 0xFF00 <= cp <= 0xFF60
        or 0xFFE0 <= cp <= 0xFFE6
        or 0x20000 <= cp <= 0x2FFFD
        or 0x30000 <= cp <= 0x3FFFD
    )


def drive(keys_script):
    """按脚本发送按键，返回最终屏幕网格。"""
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", H, W, 0, 0))
    env = dict(os.environ)
    env["TERM"] = "xterm-256color"
    env["LANG"] = os.environ.get("CAP_LANG", "zh_CN.UTF-8")
    env["HOME"] = "/tmp/shot_home"
    os.makedirs(env["HOME"], exist_ok=True)
    p = subprocess.Popen(
        [BIN], stdin=slave, stdout=slave, stderr=slave, env=env, close_fds=True
    )
    os.close(slave)
    raw = b""
    last_send = time.time()
    for keys in keys_script:
        if keys:
            os.write(master, keys.encode())
        # 等待输出稳定
        end = time.time() + 1.0
        while time.time() < end:
            r, _, _ = select.select([master], [], [], 0.15)
            if r:
                try:
                    d = os.read(master, 65536)
                except OSError:
                    break
                if not d:
                    break
                raw += d
            elif time.time() - last_send > 0.4:
                break
            last_send = time.time()
    time.sleep(0.3)
    while True:
        r, _, _ = select.select([master], [], [], 0.15)
        if not r:
            break
        try:
            d = os.read(master, 65536)
        except OSError:
            break
        if not d:
            break
        raw += d
    p.kill()
    os.close(master)
    t = Term(W, H)
    t.feed(raw)
    return t


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "menu"
    home = "/tmp/shot_home"
    if os.path.exists(home):
        import shutil

        shutil.rmtree(home)
    scripts = {
        "menu": ["", "\r"],  # 用户名界面回车 → 菜单
        "snake": ["", "\r", "\r"],
        "breakout": ["", "\r", "j", "\r", " "],
        "tetris": ["", "\r", "j", "j", " "],
        "plane": ["", "\r", "j", "j", "j", "\r"],
        "gomoku": ["", "\r", "j", "j", "j", "j", "\r"],
        "pause": ["", "\r", "\r", "\x1b"],  # 贪吃蛇里按 ESC
        "summary": ["", "\r", "j", "\r", " ", " "],  # 打砖块里发球等一会
    }
    if which not in scripts:
        print("unknown screen:", which)
        sys.exit(1)
    t = drive(scripts[which])
    print(t.render_text())


if __name__ == "__main__":
    main()

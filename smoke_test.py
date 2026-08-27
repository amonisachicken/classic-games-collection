#!/usr/bin/env python3
"""PTY 冒烟测试：驱动 classic-games-collection 在伪终端里运行，模拟按键验证主要流程。"""
import os
import pty
import select
import sys
import time
import fcntl
import termios
import struct

BIN = os.path.join(os.path.dirname(os.path.abspath(__file__)), "target", "release", "classic-games-collection")
TERM = os.environ.get("TERM", "xterm-256color")


class Driver:
    def __init__(self, lang="zh_CN.UTF-8"):
        self.master, slave = pty.openpty()
        # 设置伪终端窗口尺寸，确保 terminal::size() 正常返回
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 110, 0, 0))
        self.proc = None
        import subprocess
        env = dict(os.environ)
        env["TERM"] = TERM
        env["LANG"] = lang
        env["HOME"] = "/tmp/smoke_home"
        os.makedirs(env["HOME"], exist_ok=True)
        self.proc = subprocess.Popen(
            [BIN], stdin=slave, stdout=slave, stderr=slave, env=env, close_fds=True
        )
        os.close(slave)
        self.buf = b""

    def read(self, timeout=1.0):
        end = time.time() + timeout
        out = b""
        while time.time() < end:
            r, _, _ = select.select([self.master], [], [], 0.1)
            if r:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                out += chunk
            else:
                if out:
                    break
        self.buf += out
        return out

    def send(self, s):
        os.write(self.master, s.encode())
        time.sleep(0.15)

    def drain(self, t=0.4):
        return self.read(t)

    def close(self):
        try:
            self.proc.kill()
        except Exception:
            pass
        try:
            os.close(self.master)
        except Exception:
            pass


def visible_text(buf):
    """去除 ANSI 转义码后得到可见文本。"""
    import re
    t = buf.decode("utf-8", "replace")
    t = re.sub(r"\x1b\[[0-9;?]*[a-zA-Z]", "", t)
    t = re.sub(r"\x1b\][^\x07]*\x07", "", t)
    t = re.sub(r"\x1b\][^\x1b]*\x1b\\", "", t)
    t = t.replace("\x1b", "")
    return t


def has_cjk(v, s):
    """CJK 全角字符在画布中每字后带占位空格，比较时去掉所有空格。"""
    return s.replace(" ", "") in v.replace(" ", "")


def step_text(d, keys="", wait=0.6):
    """发送按键并返回这段时间内新输出的可见文本（不含历史帧）。"""
    if keys:
        d.send(keys)
    return visible_text(d.drain(wait))


def main():
    ok = True
    d = Driver()
    try:
        # 1. 首次运行 → 用户名输入界面
        v = step_text(d, "", 1.2)
        assert has_cjk(v, "设置用户名"), f"未出现用户名界面: {v[:200]!r}"
        print("[ok] 首次运行显示用户名输入界面")

        # 2. 回车确认默认用户名 → 主菜单
        v = step_text(d, "\r", 0.8)
        assert has_cjk(v, "经典游戏合集"), f"未出现主菜单: {v[:200]!r}"
        for g in ["贪吃蛇", "打砖块", "俄罗斯方块", "飞机大战", "五子棋", "退出"]:
            assert has_cjk(v, g), f"菜单缺少 {g}"
        print("[ok] 主菜单显示五款游戏与退出")

        # 3. HJKL 菜单导航：j 下移一项 → 回车进入打砖块
        v = step_text(d, "j", 0.3)
        v = step_text(d, "\r", 0.8)
        assert has_cjk(v, "打砖块") and "BREAKOUT" in v, f"未进入打砖块: {v[-200:]!r}"
        print("[ok] HJKL(j) 菜单导航进入打砖块")

        # 4. 打砖块：空格发球 → L/R 移动 → ESC 暂停 → q 返回
        v = step_text(d, " ", 0.4)
        v = step_text(d, "l", 0.3)
        v = step_text(d, "h", 0.3)
        assert len(v) > 0, "打砖块无响应"
        v = step_text(d, "\x1b", 0.5)
        assert has_cjk(v, "已暂停"), "打砖块暂停菜单未出现"
        v = step_text(d, "q", 0.7)
        assert has_cjk(v, "经典游戏合集"), "打砖块未返回菜单"
        print("[ok] 打砖块 发球/移动/暂停/返回")

        # 5. 贪吃蛇：默认选中项回车进入；j/h 移动；ESC 暂停 c 继续；ESC q 返回
        v = step_text(d, "\r", 0.8)
        assert has_cjk(v, "贪吃蛇") and has_cjk(v, "得分"), f"未进入贪吃蛇: {v[-200:]!r}"
        v = step_text(d, "j", 0.35)
        v = step_text(d, "h", 0.35)
        assert len(v) > 0, "贪吃蛇无响应"
        v = step_text(d, "\x1b", 0.5)
        assert has_cjk(v, "已暂停"), "贪吃蛇暂停菜单未出现"
        v = step_text(d, "c", 0.5)
        assert not has_cjk(v, "已暂停"), "继续后仍显示暂停"
        v = step_text(d, "\x1b", 0.5)
        v = step_text(d, "q", 0.7)
        assert has_cjk(v, "经典游戏合集"), "贪吃蛇未返回菜单"
        print("[ok] 贪吃蛇 移动/暂停/继续/返回")

        # 6. 俄罗斯方块：jj → 空格进入；h/k/空格 操作；ESC q 返回
        v = step_text(d, "j", 0.25)
        v = step_text(d, "j", 0.25)
        v = step_text(d, " ", 0.8)
        assert has_cjk(v, "俄罗斯方块") and "TETRIS" in v, f"未进入俄罗斯方块: {v[-200:]!r}"
        v = step_text(d, "h", 0.3)
        assert len(v) > 0, "俄罗斯方块左移无响应"
        v = step_text(d, "k", 0.3)
        assert len(v) > 0, "俄罗斯方块旋转无响应"
        v = step_text(d, " ", 0.3)
        assert len(v) > 0, "俄罗斯方块直落无响应"
        v = step_text(d, "\x1b", 0.5)
        v = step_text(d, "q", 0.7)
        assert has_cjk(v, "经典游戏合集"), "俄罗斯方块未返回菜单"
        print("[ok] 俄罗斯方块 移动/旋转/直落/返回")

        # 7. 飞机大战：jjj → 回车进入；方向键移动；按住空格射击；ESC q 返回
        for _ in range(3):
            v = step_text(d, "\x1b[B", 0.2)
        v = step_text(d, "\r", 0.8)
        assert has_cjk(v, "飞机大战") and "PLANE" in v, f"未进入飞机大战: {v[-200:]!r}"
        v = step_text(d, "\x1b[A", 0.3)  # Up
        v = step_text(d, "\x1b[D", 0.3)  # Left
        v = step_text(d, " ", 0.5)        # 按住空格(按下)
        assert len(v) > 0, "飞机大战无响应"
        v = step_text(d, " ", 0.2)        # 释放空格
        v = step_text(d, "\x1b", 0.5)
        v = step_text(d, "q", 0.7)
        assert has_cjk(v, "经典游戏合集"), "飞机大战未返回菜单"
        print("[ok] 飞机大战 移动/射击/返回")

        # 8. 五子棋：jjjj → 回车进入；回车落子 → AI 回应；ESC q 返回
        for _ in range(4):
            v = step_text(d, "j", 0.2)
        v = step_text(d, "\r", 0.8)
        assert has_cjk(v, "五子棋") and "GOMOKU" in v, f"未进入五子棋: {v[-200:]!r}"
        v = step_text(d, "\r", 1.2)
        assert has_cjk(v, "轮到你"), f"五子棋 AI 未回应: {v[-200:]!r}"
        v = step_text(d, "\x1b", 0.5)
        v = step_text(d, "q", 0.7)
        assert has_cjk(v, "经典游戏合集"), "五子棋未返回菜单"
        print("[ok] 五子棋 落子/AI回应/返回")

        # 9. 设置用户名：jjjjj → 回车 → 输入新名 alice → 回车确认
        for _ in range(5):
            v = step_text(d, "j", 0.2)
        v = step_text(d, "\r", 0.6)
        assert has_cjk(v, "设置用户名"), "未进入设置用户名界面"
        v = step_text(d, "\x7f" * 6, 0.4)  # 清空默认用户名 player
        v = step_text(d, "alice", 0.5)
        v = step_text(d, "\r", 0.8)
        assert has_cjk(v, "玩家:alice"), "用户名未更新"
        print("[ok] 设置用户名 alice")

        # 10. 返回菜单后直接 q 退出程序
        v = step_text(d, "q", 0.7)
        assert d.proc.poll() is not None, "程序未退出"
        print("[ok] 程序正常退出")

        # 11. 检查分数文件已写入家目录
        import json
        p = "/tmp/smoke_home/.classic-games-collection/scores.json"
        assert os.path.exists(p), "scores.json 未创建"
        data = json.load(open(p))
        assert data.get("user"), "user 未保存"
        print(f"[ok] 分数文件已写入: {p}")

        # 12. 英文模式(LANG=en_US): 已有用户, 直接进入菜单, 界面应显示英文
        d.close()
        d = Driver(lang="en_US.UTF-8")
        v = step_text(d, "", 1.2)
        assert "Classic Games Collection" in v, f"英文标题: {v[:150]!r}"
        for g in ["Snake", "Breakout", "Tetris", "Plane", "Gomoku", "Quit"]:
            assert g in v, f"英文菜单缺少 {g}"
        assert not has_cjk(v, "贪吃蛇"), "英文界面不应出现中文"
        print("[ok] 英文模式(LANG=en_US)界面正常")
        v = step_text(d, "q", 0.7)
        assert d.proc.poll() is not None, "英文模式退出失败"
        print("[ok] 英文模式正常退出")

    except AssertionError as e:
        ok = False
        print(f"[FAIL] {e}")
    finally:
        d.close()

    sys.exit(0 if ok else 1)

if __name__ == "__main__":
    main()

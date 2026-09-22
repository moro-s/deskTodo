"""从 SVG path 采样生成 Rust 图钉图标轮廓数据。

用法：python scripts/gen_pin_icon.py <输入.svg> <输出.rs>
采样后的多边形交给运行时的 earcut 三角化（见 src/ui/pin_icon.rs），
因此这里只需输出足够密集的轮廓点，不处理填充规则。
"""

import math
import re
import sys

BEZIER_SAMPLES = 12
ARC_SAMPLES = 16

TOKEN_RE = re.compile(
    r"[MmLlHhVvCcSsQqTtAaZz]|[-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?"
)


def tokenize(d):
    return TOKEN_RE.findall(d)


class PathSampler:
    def __init__(self, d):
        self.tokens = tokenize(d)
        self.pos = 0
        self.x = 0.0
        self.y = 0.0
        self.start = (0.0, 0.0)
        self.subpaths = []

    def number(self):
        value = float(self.tokens[self.pos])
        self.pos += 1
        return value

    def pair(self, relative=False):
        dx, dy = self.number(), self.number()
        if relative:
            return self.x + dx, self.y + dy
        return dx, dy

    def begin_subpath(self, x, y):
        self.x, self.y = x, y
        self.start = (x, y)
        self.subpaths.append([(x, y)])

    def line_to(self, x, y):
        self.current().append((x, y))
        self.x, self.y = x, y

    def current(self):
        return self.subpaths[-1]

    def cubic(self, c1, c2, end):
        x0, y0 = self.x, self.y
        x1, y1 = c1
        x2, y2 = c2
        x3, y3 = end
        for step in range(1, BEZIER_SAMPLES + 1):
            t = step / BEZIER_SAMPLES
            u = 1.0 - t
            x = u**3 * x0 + 3 * u**2 * t * x1 + 3 * u * t**2 * x2 + t**3 * x3
            y = u**3 * y0 + 3 * u**2 * t * y1 + 3 * u * t**2 * y2 + t**3 * y3
            self.current().append((x, y))
        self.x, self.y = x3, y3

    def arc(self, rx, ry, rotation_deg, large, sweep, end):
        x1, y1 = self.x, self.y
        x2, y2 = end
        phi = math.radians(rotation_deg)
        dx, dy = (x1 - x2) / 2.0, (y1 - y2) / 2.0
        x1p = math.cos(phi) * dx + math.sin(phi) * dy
        y1p = -math.sin(phi) * dx + math.cos(phi) * dy
        rx, ry = abs(rx), abs(ry)
        lam = x1p**2 / rx**2 + y1p**2 / ry**2
        if lam > 1.0:
            scale = math.sqrt(lam)
            rx, ry = rx * scale, ry * scale
        sign = -1.0 if large == sweep else 1.0
        num = max(rx**2 * ry**2 - rx**2 * y1p**2 - ry**2 * x1p**2, 0.0)
        den = rx**2 * y1p**2 + ry**2 * x1p**2
        co = sign * math.sqrt(num / den) if den else 0.0
        cxp = co * rx * y1p / ry
        cyp = -co * ry * x1p / rx
        cx = math.cos(phi) * cxp - math.sin(phi) * cyp + (x1 + x2) / 2.0
        cy = math.sin(phi) * cxp + math.cos(phi) * cyp + (y1 + y2) / 2.0

        def angle(ux, uy, vx, vy):
            dot = ux * vx + uy * vy
            cross = ux * vy - uy * vx
            return math.atan2(cross, dot)

        theta1 = angle(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry)
        dtheta = angle(
            (x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry
        )
        if not sweep and dtheta > 0.0:
            dtheta -= 2.0 * math.pi
        elif sweep and dtheta < 0.0:
            dtheta += 2.0 * math.pi
        for step in range(1, ARC_SAMPLES + 1):
            t = step / ARC_SAMPLES
            a = theta1 + dtheta * t
            cos_a, sin_a = math.cos(a), math.sin(a)
            x = cx + rx * cos_a * math.cos(phi) - ry * sin_a * math.sin(phi)
            y = cy + rx * cos_a * math.sin(phi) + ry * sin_a * math.cos(phi)
            self.current().append((x, y))
        self.x, self.y = x2, y2

    def parse(self):
        command = None
        while self.pos < len(self.tokens):
            token = self.tokens[self.pos]
            if token.isalpha():
                command = token
                self.pos += 1
                implicit_start = True
            else:
                if command in ("M", "m"):
                    command = "L" if command == "M" else "l"
                elif command in ("Z", "z"):
                    raise ValueError("path 数据在 Z 后仍有数字")
                implicit_start = False
            cmd = command
            if cmd in ("M", "m"):
                x, y = self.pair(relative=cmd == "m")
                if implicit_start:
                    self.begin_subpath(x, y)
                else:
                    self.line_to(x, y)
            elif cmd in ("L", "l"):
                x, y = self.pair(relative=cmd == "l")
                self.line_to(x, y)
            elif cmd in ("H", "h"):
                value = self.number()
                x = self.x + value if cmd == "h" else value
                self.line_to(x, self.y)
            elif cmd in ("V", "v"):
                value = self.number()
                y = self.y + value if cmd == "v" else value
                self.line_to(self.x, y)
            elif cmd in ("C", "c"):
                rel = cmd == "c"
                c1 = self.pair(relative=rel)
                c2 = self.pair(relative=rel)
                end = self.pair(relative=rel)
                self.cubic(c1, c2, end)
            elif cmd in ("A", "a"):
                rx, ry = self.number(), self.number()
                rotation = self.number()
                large = self.number()
                sweep = self.number()
                end = self.pair(relative=cmd == "a")
                self.arc(rx, ry, rotation, large, sweep, end)
            elif cmd in ("Z", "z"):
                self.current().append(self.start)
                self.x, self.y = self.start
                command = None
            else:
                raise ValueError(f"不支持的 SVG 命令：{cmd}")
        return self.subpaths


def format_points(points):
    values = []
    for x, y in points:
        values.append(f"{x:.5f}")
        values.append(f"{y:.5f}")
    lines = []
    for i in range(0, len(values), 8):
        lines.append("    " + ", ".join(values[i : i + 8]) + ",")
    return "\n".join(lines)


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 1
    with open(sys.argv[1], encoding="utf-8") as file:
        svg = file.read()
    match = re.search(r'\bd="([^"]+)"', svg)
    if not match:
        raise ValueError("SVG 中未找到 path d 属性")
    subpaths = PathSampler(match.group(1)).parse()
    if len(subpaths) != 2:
        raise ValueError(f"预期 2 个子路径（外轮廓 + 内洞），实际 {len(subpaths)}")
    outline, hole = subpaths[0], subpaths[1]
    xs = [p[0] for p in outline]
    ys = [p[1] for p in outline]
    min_x, max_x, min_y, max_y = min(xs), max(xs), min(ys), max(ys)
    center_x = (min_x + max_x) / 2.0
    center_y = (min_y + max_y) / 2.0
    max_dim = max(max_x - min_x, max_y - min_y)

    def normalize(points):
        return [((x - center_x) / max_dim, (y - center_y) / max_dim) for x, y in points]

    outline_n, hole_n = normalize(outline), normalize(hole)
    content = f"""//! 图钉图标轮廓数据。
//!
//! 由 scripts/gen_pin_icon.py 从用户提供的 SVG 采样生成，请勿手改。
//! 坐标以图标中心为原点，最大边归一化为 1（y 轴与 egui 一致向下）。

/// 外轮廓顶点（x, y 平铺），共 {len(outline_n)} 个点。
pub(crate) const PIN_OUTLINE: &[f32] = &[
{format_points(outline_n)}
];

/// 内部镂空（针槽）顶点（x, y 平铺），共 {len(hole_n)} 个点。
pub(crate) const PIN_HOLE: &[f32] = &[
{format_points(hole_n)}
];
"""
    with open(sys.argv[2], "w", encoding="utf-8", newline="\n") as file:
        file.write(content)
    print(
        f"outline={len(outline_n)} 点, hole={len(hole_n)} 点, "
        f"bbox={max_x - min_x:.1f}x{max_y - min_y:.1f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

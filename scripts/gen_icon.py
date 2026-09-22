"""从 SVG path 采样生成 Rust 图标轮廓数据（通用版）。

用法：python scripts/gen_icon.py <输入.svg> <输出.rs> <常量前缀> [图标名称]

支持任意数量的子路径，按 nonzero 填充语义自动分类：
互不包含的环是独立形状的外环；被外环包含且绕向相反的环是它的洞。
采样后的多边形交给运行时的 earcut 三角化（见 src/ui/icon_mesh.rs），
因此这里只需输出足够密集的轮廓点。
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


def dedupe_ring(points):
    """去掉首尾重复与相邻重复点。"""
    result = []
    for pt in points:
        if not result or pt != result[-1]:
            result.append(pt)
    if len(result) > 1 and result[0] == result[-1]:
        result.pop()
    return result


def signed_area(points):
    total = 0.0
    for (x1, y1), (x2, y2) in zip(points, points[1:] + points[:1]):
        total += x1 * y2 - x2 * y1
    return total / 2.0


def bbox(points):
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    return min(xs), min(ys), max(xs), max(ys)


def interior_point(points):
    """取环最左顶点左侧的微小偏移点，保证落在环内部。"""
    min_x = min(p[0] for p in points)
    min_y = min(p[1] for p in points if p[0] == min_x)
    width = max(p[0] for p in points) - min_x
    eps = max(width, 1.0) * 1e-7
    return (min_x - eps, min_y)


def point_in_polygon(pt, poly):
    x, y = pt
    inside = False
    for (x1, y1), (x2, y2) in zip(poly, poly[1:] + poly[:1]):
        if (y1 > y) != (y2 > y):
            x_cross = x1 + (y - y1) * (x2 - x1) / (y2 - y1)
            if x < x_cross:
                inside = not inside
    return inside


def classify_rings(rings):
    """按 nonzero 语义把环分类为形状（外环+洞列表）。

    返回 [(outer_ring, [hole, ...]), ...]，形状按外环出现顺序排列。
    """
    areas = [signed_area(ring) for ring in rings]
    probes = [interior_point(ring) for ring in rings]
    for index, area in enumerate(areas):
        if abs(area) < 1e-9:
            raise ValueError(f"环 {index} 面积为 0（退化路径）")

    def contains(outer_index, inner_index):
        return point_in_polygon(probes[inner_index], rings[outer_index])

    # 直接父环 = 包含该环的最小环（不被更内层环隔开）
    parents = []
    for i in range(len(rings)):
        candidates = [j for j in range(len(rings)) if j != i and contains(j, i)]
        if not candidates:
            parents.append(None)
            continue
        # 最小包含环：其面积最小（嵌套最深的直接父环）
        best = min(candidates, key=lambda j: abs(areas[j]))
        # 排除"经由孙环"的假父环：若候选 j 本身被另一个候选 k 包含，则 j 不是直接父环
        direct = [
            j
            for j in candidates
            if not any(k != j and contains(k, j) for k in candidates)
        ]
        best = min(direct, key=lambda j: abs(areas[j]))
        parents.append(best)

    shapes = []
    used = set()
    for i, ring in enumerate(rings):
        if parents[i] is not None:
            continue  # 洞或嵌套形状，由其父环处理
        shape = [ring]
        used.add(i)
        holes = [
            j
            for j in range(len(rings))
            if parents[j] == i and areas[j] * areas[i] < 0
        ]
        nested = [
            j
            for j in range(len(rings))
            if parents[j] == i and areas[j] * areas[i] > 0
        ]
        if nested:
            raise ValueError(
                f"环 {i} 内出现同向嵌套环 {nested}，nonzero 下应视为岛，暂不支持"
            )
        for j in sorted(holes):
            shape.append(rings[j])
            used.add(j)
        shapes.append((i, shape))

    if len(used) != len(rings):
        leftover = [i for i in range(len(rings)) if i not in used]
        raise ValueError(f"存在未分类的环：{leftover}")
    return [shape for _, shape in shapes]


def format_values(values, per_line=8):
    lines = []
    for i in range(0, len(values), per_line):
        lines.append("    " + ", ".join(values[i : i + per_line]) + ",")
    return "\n".join(lines)


def main():
    if len(sys.argv) not in (4, 5):
        print(__doc__)
        return 1
    svg_path, out_path, prefix = sys.argv[1], sys.argv[2], sys.argv[3]
    icon_name = sys.argv[4] if len(sys.argv) == 5 else prefix

    with open(svg_path, encoding="utf-8") as file:
        svg = file.read()
    match = re.search(r'\bd="([^"]+)"', svg)
    if not match:
        raise ValueError("SVG 中未找到 path d 属性")

    subpaths = PathSampler(match.group(1)).parse()
    rings = []
    for index, subpath in enumerate(subpaths):
        ring = dedupe_ring(subpath)
        if len(ring) < 3:
            raise ValueError(f"子路径 {index} 顶点过少：{len(ring)}")
        rings.append(ring)

    shapes = classify_rings(rings)

    all_points = [pt for shape in shapes for ring in shape for pt in ring]
    xs = [p[0] for p in all_points]
    ys = [p[1] for p in all_points]
    min_x, max_x, min_y, max_y = min(xs), max(xs), min(ys), max(ys)
    center_x = (min_x + max_x) / 2.0
    center_y = (min_y + max_y) / 2.0
    max_dim = max(max_x - min_x, max_y - min_y)

    def normalize(point):
        x, y = point
        return ((x - center_x) / max_dim, (y - center_y) / max_dim)

    vertices = []
    ring_sizes = []
    shape_ring_counts = []
    for shape in shapes:
        shape_ring_count = 0
        for ring in shape:
            for point in ring:
                nx, ny = normalize(point)
                vertices.append(f"{nx:.5f}")
                vertices.append(f"{ny:.5f}")
            ring_sizes.append(str(len(ring)))
            shape_ring_count += 1
        shape_ring_counts.append(str(shape_ring_count))

    content = f"""//! {icon_name} 图标轮廓数据。
//!
//! 由 scripts/gen_icon.py 从用户提供的 SVG 采样生成，请勿手改。
//! 坐标以图标中心为原点，最大边归一化为 1（y 轴与 egui 一致向下）。

/// 所有环的顶点（x, y 平铺），按形状与环的顺序排列。
pub(crate) const {prefix}_VERTICES: &[f32] = &[
{format_values(vertices)}
];

/// 每个环的顶点数。
pub(crate) const {prefix}_RING_SIZES: &[u16] = &[
    {", ".join(ring_sizes)},
];

/// 每个形状包含的环数（首环为外环，其余为洞）。
pub(crate) const {prefix}_SHAPE_RING_COUNTS: &[u16] = &[
    {", ".join(shape_ring_counts)},
];
"""
    with open(out_path, "w", encoding="utf-8", newline="\n") as file:
        file.write(content)

    ring_desc = ", ".join(
        f"环{i}({len(ring)}点{'+' if signed_area(ring) > 0 else '-'})"
        for i, ring in enumerate(rings)
    )
    print(
        f"rings={len(rings)} [{ring_desc}], shapes={len(shapes)}, "
        f"bbox={max_x - min_x:.1f}x{max_y - min_y:.1f}, "
        f"vertices={len(vertices) // 2}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

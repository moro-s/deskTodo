//! 通用 SVG 图标 mesh 绘制。
//!
//! 图标数据由 scripts/gen_icon.py ���样生成：所有环的顶点池、
//! 每个环的顶点数、每个形状包含的环数（首环为外环，其余为洞）。
//! 运行时对每个形状独立 earcut 三角化，索引偏移回全局顶点池后合并绘制。

use earcut::Earcut;
use eframe::egui::{self, Color32, Mesh, Shape, Vec2};
use eframe::egui::epaint::Vertex;
use std::sync::OnceLock;

/// 一个图标的静态几何描述。
pub(crate) struct IconGeometry {
    vertices: &'static [f32],
    ring_sizes: &'static [u16],
    shape_ring_counts: &'static [u16],
    triangles: OnceLock<Vec<usize>>,
}

impl IconGeometry {
    pub(crate) const fn new(
        vertices: &'static [f32],
        ring_sizes: &'static [u16],
        shape_ring_counts: &'static [u16],
    ) -> Self {
        Self {
            vertices,
            ring_sizes,
            shape_ring_counts,
            triangles: OnceLock::new(),
        }
    }

    /// 环顶点数的前缀和：`ring_bounds[i]` 为环 i 结束后的顶点总数。
    fn ring_bounds(&self) -> Vec<usize> {
        let mut bounds = Vec::with_capacity(self.ring_sizes.len());
        let mut total = 0;
        for &size in self.ring_sizes {
            total += size as usize;
            bounds.push(total);
        }
        bounds
    }

    /// 三角化所有形状，输出全局顶点池索引（进程内计算一次）。
    fn triangulated(&self) -> &[usize] {
        self.triangles.get_or_init(|| {
            let ring_bounds = self.ring_bounds();
            let mut all_indices = Vec::new();
            let mut ring_start = 0;
            let mut vertex_start = 0;

            for &ring_count in self.shape_ring_counts {
                let ring_end = ring_start + ring_count as usize;
                let vertex_end = ring_bounds[ring_end - 1];

                let mut local: Vec<[f64; 2]> = (vertex_start..vertex_end)
                    .map(|index| {
                        [
                            f64::from(self.vertices[index * 2]),
                            f64::from(self.vertices[index * 2 + 1]),
                        ]
                    })
                    .collect();
                let holes: Vec<usize> = (ring_start + 1..ring_end)
                    .map(|ring| ring_bounds[ring - 1] - vertex_start)
                    .collect();

                let mut shape_indices = Vec::new();
                Earcut::<f64>::new().earcut(local.drain(..), &holes, &mut shape_indices);
                all_indices.extend(
                    shape_indices
                        .into_iter()
                        .map(|index| index + vertex_start),
                );

                ring_start = ring_end;
                vertex_start = vertex_end;
            }
            all_indices
        })
    }

    /// 在 center 处绘制图标，size 为图标最大边的长度（逻辑像素）。
    pub(crate) fn paint(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        size: f32,
        color: Color32,
    ) {
        let indices = self.triangulated();
        if indices.is_empty() {
            return;
        }
        let mut mesh = Mesh::default();
        for pair in self.vertices.as_chunks::<2>().0 {
            mesh.vertices.push(Vertex::untextured(
                center + Vec2::new(pair[0] * size, pair[1] * size),
                color,
            ));
        }
        mesh.indices = indices.iter().map(|&index| index as u32).collect();
        painter.add(Shape::mesh(mesh));
    }
}

#[cfg(test)]
pub(crate) fn assert_icon_area(geometry: &IconGeometry) {
    let to_points = |ring: &[f32]| -> Vec<[f64; 2]> {
        ring.as_chunks::<2>()
            .0
            .iter()
            .map(|pair| [f64::from(pair[0]), f64::from(pair[1])])
            .collect()
    };
    let ring_area = |points: &[[f64; 2]]| -> f64 {
        let mut area = 0.0;
        for index in 0..points.len() {
            let next = (index + 1) % points.len();
            area += points[index][0] * points[next][1] - points[next][0] * points[index][1];
        }
        (area / 2.0).abs()
    };

    let rings: Vec<Vec<[f64; 2]>> = {
        let mut rings = Vec::new();
        let mut offset = 0;
        for &size in geometry.ring_sizes {
            rings.push(to_points(&geometry.vertices[offset..offset + size as usize * 2]));
            offset += size as usize * 2;
        }
        rings
    };
    let mut expected = 0.0;
    let mut ring_start = 0;
    for &ring_count in geometry.shape_ring_counts {
        let ring_end = ring_start + ring_count as usize;
        for (position, ring) in rings[ring_start..ring_end].iter().enumerate() {
            if position == 0 {
                expected += ring_area(ring);
            } else {
                expected -= ring_area(ring);
            }
        }
        ring_start = ring_end;
    }

    let triangle_area = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| -> f64 {
        ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
    };
    let indices = geometry.triangulated();
    let mut actual = 0.0;
    for triangle in indices.as_chunks::<3>().0 {
        let vertex = |index: usize| {
            [
                f64::from(geometry.vertices[index * 2]),
                f64::from(geometry.vertices[index * 2 + 1]),
            ]
        };
        actual += triangle_area(vertex(triangle[0]), vertex(triangle[1]), vertex(triangle[2]));
    }

    assert!(
        (actual - expected).abs() < 1e-3,
        "triangles={} actual={actual} expected={expected}",
        indices.len() / 3
    );
}

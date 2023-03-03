use macroquad::prelude::*;
use macroquad::models::Vertex;
use std::f32::consts::PI;

pub fn draw_ring_mesh(x: f32, y: f32, inner: f32, outer: f32, color: Color) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let n = 24;
    let delta = 2. * PI / n as f32;
    let mut alpha: f32 = 0.;

    vertices.push(Vertex {
        position: Vec3::new(x + inner * alpha.cos(), y + inner * alpha.sin(), 0.),
        uv: Vec2::new(0., 0.),
        color,
    });

    vertices.push(Vertex {
        position: Vec3::new(x + outer * alpha.cos(), y + outer * alpha.sin(), 0.),
        uv: Vec2::new(0., 0.),
        color,
    });

    for i in 0..n {
        alpha += delta;
        vertices.push(Vertex {
            position: Vec3::new(x + inner * alpha.cos(), y + inner * alpha.sin(), 0.),
            uv: Vec2::new(0., 0.),
            color,
        });
        vertices.push(Vertex {
            position: Vec3::new(x + outer * alpha.cos(), y + outer * alpha.sin(), 0.),
            uv: Vec2::new(0., 0.),
            color,
        });

        indices.push(0 + 2 * i);
        indices.push(1 + 2 * i);
        indices.push(3 + 2 * i);

        indices.push(0 + 2 * i);
        indices.push(3 + 2 * i);
        indices.push(2 + 2 * i);
    }

    let mesh = Mesh {
        vertices,
        indices,
        texture: None,
    };

    draw_mesh(&mesh);
}

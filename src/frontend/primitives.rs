use macroquad::models::Vertex;
use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::{common::coord::{Point, HexCoordF, HexCoord}, core::entities::Player};

use super::mouse::MouseEvent;

#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    MouseEntered,
    MouseLeft,
    ElementMoved(Point),
    ElementShow,
    ElementHide,
    MouseClicked(HexCoord),
    Tick,
    FlipMarker(HexCoord),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    FlipMarker(HexCoord),
    RemoveMarker(HexCoord),
    RemoveRing(HexCoord),
    MoveRing(HexCoord, HexCoord),
    PlaceRing(Player, HexCoord),
}

pub fn build_grid_lines(radius: f32) -> Vec<[HexCoordF; 2]> {
    let dx: f32 = 0.5 * (3. as f32).sqrt();
    let mut res = Vec::new();

    // diagonals
    for dy in [-0.5 as f32, 0.5 as f32] {
        let lambda: f32 = radius / (1. - dy.powi(2)).sqrt();
        let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

        for l in l0..=l1 {
            let l = l as f32;
            let det = (l.powi(2) * (dy.powi(2) - 1.) + radius.powi(2)).sqrt();
            if det <= 0. {
                continue;
            }
            let mut mu1 = -l * dy - det;
            let mut mu2 = -l * dy + det;
            if l.abs() > radius {
                mu1 = mu1.ceil();
                mu2 = mu2.floor();
            } else {
                mu1 = mu1.trunc();
                mu2 = mu2.floor();
            }

            let vec = [
                HexCoordF(mu1 * dx, l + mu1 * dy),
                HexCoordF(mu2 * dx, l + mu2 * dy),
            ];
            res.push(vec);
        }
    }

    // verticals
    let lambda: f32 = radius * 2. / 3. * (3. as f32).sqrt();
    let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

    for l in l0..=l1 {
        let l = l as f32;
        let det = (4. * radius.powi(2) - 3. * l.powi(2)).sqrt();
        if det <= 0. {
            continue;
        }
        let mut mu1 = 0.5 * (l - det);
        let mut mu2 = 0.5 * (l + det);
        if l.abs() > radius {
            mu1 = mu1.ceil();
            mu2 = mu2.floor();
        } else {
            mu1 = mu1.trunc();
            mu2 = mu2.floor();
        }

        let vec = [
            HexCoordF(l * dx, -0.5 * l + mu1),
            HexCoordF(l * dx, -0.5 * l + mu2),
        ];
        res.push(vec);
    }
    res
}

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

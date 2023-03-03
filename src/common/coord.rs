use itertools::PeekingNext;
use num::{self, integer::Roots};
use std::{
    cmp,
    ops::{Add, Sub},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    N,
    NE,
    SE,
    S,
    SW,
    NW,
}

impl Direction {
    pub fn dir_vec(&self) -> HexCoord {
        match *self {
            Direction::N => HexCoord(0, 1),
            Direction::NE => HexCoord(1, 1),
            Direction::SE => HexCoord(1, 0),
            Direction::S => HexCoord(0, -1),
            Direction::SW => HexCoord(-1, -1),
            Direction::NW => HexCoord(-1, 0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::N => Direction::S,
            Direction::NE => Direction::SW,
            Direction::SE => Direction::NW,
            Direction::S => Direction::N,
            Direction::SW => Direction::NE,
            Direction::NW => Direction::SE,
        }
    }

    pub fn all() -> Vec<Direction> {
        vec![
            Direction::N,
            Direction::NE,
            Direction::SE,
            Direction::S,
            Direction::SW,
            Direction::NW,
        ]
    }
}

pub fn distance_squared(p0: &Point, p1: &Point) -> f32 {
    norm_squared(&Point(p0.0 - p1.0, p0.1 - p1.1))
}

pub fn norm_squared(p: &Point) -> f32 {
    p.0.powi(2) + p.1.powi(2)
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Point(pub f32, pub f32);

impl From<HexCoord> for Point {
    fn from(value: HexCoord) -> Self {
        Point(
            value.0 as f32 * 0.5 * num::Float::sqrt(3.),
            value.1 as f32 - 0.5 * value.0 as f32,
        )
    }
}

impl From<HexCoordF> for Point {
    fn from(value: HexCoordF) -> Self {
        Point(
            value.0 * 0.5 * num::Float::sqrt(3.),
            value.1 - 0.5 * value.0,
        )
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct HexCoordF(pub f32, pub f32);

impl From<Point> for HexCoordF {
    fn from(value: Point) -> Self {
        let s3: f32 = (3. as f32).sqrt();
        HexCoordF(2. / 3. * s3 * value.0, value.1 + s3 / 3. * value.0)
    }
}

impl From<HexCoord> for HexCoordF {
    fn from(value: HexCoord) -> Self {
        HexCoordF(value.0 as f32, value.1 as f32)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct HexCoord(pub i8, pub i8);

impl HexCoord {
    pub fn new(x: i8, y: i8) -> Self {
        HexCoord(x, y)
    }

    pub fn cartesian_sq_norm(&self) -> f32 {
        norm_squared(&self.clone().into())
    }

    pub fn cartesian_sq_dist(&self, other: &HexCoord) -> f32 {
        (*self - *other).cartesian_sq_norm()
    }

    pub fn closest_coord_to_point(pt: &Point) -> (Self, f32) {
        let hex_approx = HexCoordF::from(*pt);
        let origin = HexCoord::new(hex_approx.0.floor() as i8, hex_approx.1.floor() as i8);

        let mut c = origin;
        let mut d = distance_squared(&c.into(), pt);

        for dir in [Direction::N, Direction::NE, Direction::SE] {
            let n = origin.neighbour(&dir);
            let newd = distance_squared(&n.into(), pt);
            if newd < d {
                c = n;
                d = newd;
            }
        }
        (c, d)
    }

    pub fn sq_norm(&self) -> i8 {
        ((self.0 as f32).powi(2) + (self.1 as f32).powi(2)) as i8
    }

    pub fn neighbour(&self, direction: &Direction) -> HexCoord {
        *self + direction.dir_vec()
    }

    pub fn is_neighbour(&self, coord: &HexCoord) -> bool {
        let dir = self.dir_vec_to(coord);
        dir.map_or(false, |d| *self + d == *coord)
    }

    pub fn abs(&self) -> HexCoord {
        HexCoord(num::abs(self.0), num::abs(self.1))
    }

    pub fn is_dir_vec(&self) -> bool {
        let c = self.abs();
        c.0 <= 1 && c.1 <= 1
    }

    pub fn is_norm_dir_vec(&self) -> bool {
        self.is_dir_vec() && self.abs() == *self
    }

    pub fn dir_vec_to(&self, other: &HexCoord) -> Option<HexCoord> {
        if !self.connected_to(other) || self == other {
            return None;
        }
        let delta = *other - *self; //.abs()??;
        let div = cmp::max(num::abs(delta.0), num::abs(delta.1));

        Some(HexCoord(
            num::Integer::div_floor(&delta.0, &div),
            num::Integer::div_floor(&delta.1, &div),
        ))
    }

    pub fn connected_to(&self, other: &HexCoord) -> bool {
        let HexCoord(x0, y0) = *self;
        let HexCoord(x1, y1) = *other;
        x0 == x1 || y0 == y1 || x0 - y0 == x1 - y1
    }

    pub fn line_iter(&self, dir: &Direction) -> LineIter {
        LineIter {
            current: self.clone(),
            dir: dir.dir_vec(),
        }
    }

    pub fn range_iter(&self, to: &HexCoord) -> Option<RangeIter> {
        self.dir_vec_to(to).and_then(|dir| {
            Some(RangeIter {
                current: self.clone(),
                to: to.clone(),
                dir,
            })
        })
    }

    pub fn between_iter(&self, to: &HexCoord) -> Option<impl Iterator<Item = HexCoord>> {
        let to = to.clone();
        self.range_iter(&to)
            .and_then(move |iter| Some(iter.skip(1).take_while(move |x| *x != to)))
    }
}

impl From<(i8, i8)> for HexCoord {
    fn from(t: (i8, i8)) -> Self {
        HexCoord::new(t.0, t.1)
    }
}

impl Add for HexCoord {
    type Output = HexCoord;

    fn add(self, other: HexCoord) -> Self {
        HexCoord(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for HexCoord {
    type Output = HexCoord;

    fn sub(self, other: HexCoord) -> Self {
        HexCoord(self.0 - other.0, self.1 - other.1)
    }
}

pub struct LineIter {
    current: HexCoord,
    dir: HexCoord,
}

impl Iterator for LineIter {
    type Item = HexCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(self.current);
        self.current = self.current + self.dir;
        ret
    }
}

// dir = current.dir_to(to) must hold!
pub struct RangeIter {
    current: HexCoord,
    to: HexCoord,
    dir: HexCoord,
}

impl Iterator for RangeIter {
    type Item = HexCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.to + self.dir {
            return None;
        }
        let res = self.current;
        self.current = self.current + self.dir;
        Some(res)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn create_vec<T>(pos: T, dir: T, n: usize) -> Vec<HexCoord>
    where
        T: Into<HexCoord> + Copy,
    {
        let mut res = vec![];
        let mut curr = pos.into();
        for _ in 0..n {
            res.push(curr);
            curr = curr + dir.into();
        }
        res
    }

    #[test]
    fn test_line_iter() {
        let cases = vec![
            (HexCoord::new(0, 0), Direction::N, 5),
            (HexCoord::new(2, 2), Direction::NE, 7),
            (HexCoord::new(-1, -3), Direction::SE, 3),
            (HexCoord::new(1, -3), Direction::S, 1),
            (HexCoord::new(-1, 3), Direction::SW, 6),
            (HexCoord::new(1, 3), Direction::NW, 2),
        ];
        for (c, d, n) in cases {
            assert_eq!(
                c.line_iter(&d).take(n).collect::<Vec<_>>(),
                create_vec(c, d.dir_vec(), n)
            );
        }
    }

    #[test]
    fn test_between_iter() {
        assert!(HexCoord::new(1, 1)
            .between_iter(&HexCoord::new(1, 1))
            .is_none());
        assert!(HexCoord::new(1, 1)
            .between_iter(&HexCoord::new(5, 4))
            .is_none());

        let cases = vec![
            (
                HexCoord::new(0, 0),
                HexCoord::new(4, 4),
                create_vec((1, 1), (1, 1), 3),
            ),
            (
                HexCoord::new(4, 4),
                HexCoord::new(0, 0),
                create_vec((3, 3), (-1, -1), 3),
            ),
            (
                HexCoord::new(0, 2),
                HexCoord::new(-3, -1),
                create_vec((-1, 1), (-1, -1), 2),
            ),
            (
                HexCoord::new(-10, -3),
                HexCoord::new(12, -3),
                create_vec((-9, -3), (1, 0), 21),
            ),
            (HexCoord::new(1, 1), HexCoord::new(2, 2), vec![]),
            (HexCoord::new(1, -1), HexCoord::new(2, -1), vec![]),
        ];
        for (from, to, res) in cases {
            assert_eq!(from.between_iter(&to).unwrap().collect::<Vec<_>>(), res);
        }
    }

    #[test]
    fn test_range_iter() {
        assert!(HexCoord::new(1, 1)
            .range_iter(&HexCoord::new(1, 1))
            .is_none());
        assert!(HexCoord::new(1, 1)
            .range_iter(&HexCoord::new(5, 4))
            .is_none());

        let cases = vec![
            (
                HexCoord::new(0, 0),
                HexCoord::new(4, 4),
                create_vec((0, 0), (1, 1), 5),
            ),
            (
                HexCoord::new(4, 4),
                HexCoord::new(0, 0),
                create_vec((4, 4), (-1, -1), 5),
            ),
            (
                HexCoord::new(0, 2),
                HexCoord::new(-3, -1),
                create_vec((0, 2), (-1, -1), 4),
            ),
            (
                HexCoord::new(-10, -3),
                HexCoord::new(12, -3),
                create_vec((-10, -3), (1, 0), 23),
            ),
            (
                HexCoord::new(1, 1),
                HexCoord::new(2, 2),
                vec![HexCoord::new(1, 1), HexCoord::new(2, 2)],
            ),
            (
                HexCoord::new(1, -1),
                HexCoord::new(2, -1),
                vec![HexCoord::new(1, -1), HexCoord::new(2, -1)],
            ),
        ];
        for (from, to, res) in cases {
            assert_eq!(from.range_iter(&to).unwrap().collect::<Vec<_>>(), res);
        }
    }
}

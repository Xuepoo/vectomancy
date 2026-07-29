#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Basic 2D point representation in double precision.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn distance(&self, other: &Point2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Axis-aligned 2D Bounding Box.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_points(points: &[Point2D]) -> Self {
        if points.is_empty() {
            return Self::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for pt in points {
            if pt.x < min_x {
                min_x = pt.x;
            }
            if pt.y < min_y {
                min_y = pt.y;
            }
            if pt.x > max_x {
                max_x = pt.x;
            }
            if pt.y > max_y {
                max_y = pt.y;
            }
        }

        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn to_array_f32(&self) -> [f32; 4] {
        [
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        ]
    }
}

/// Polyline consisting of points and open/closed state.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polyline {
    pub points: Vec<Point2D>,
    pub closed: bool,
}

impl Polyline {
    pub fn new(points: Vec<Point2D>, closed: bool) -> Self {
        Self { points, closed }
    }
}

/// Generic path styled with metadata (e.g. color).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StyledPath<G> {
    pub geometry: G,
    pub color_style: Option<String>,
}

impl<G> StyledPath<G> {
    pub fn new(geometry: G, color_style: Option<String>) -> Self {
        Self {
            geometry,
            color_style,
        }
    }
}

/// Top-level Scene containing paths, viewport dimensions, and bounds.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scene<G> {
    pub paths: Vec<StyledPath<G>>,
    pub dimensions: (u32, u32),
    pub bounds: BoundingBox,
}

pub type PolylineScene = Scene<Polyline>;

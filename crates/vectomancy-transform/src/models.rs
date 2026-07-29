use vectomancy_geometry::Point2D;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BezierSegment {
    MoveTo(Point2D),
    LineTo(Point2D),
    QuadraticTo(Point2D, Point2D),
    CubicTo(Point2D, Point2D, Point2D),
    Close,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SplineEquation {
    pub start_t: f64,
    pub end_t: f64,
    pub x_poly: Vec<f64>,
    pub y_poly: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FourierTerm {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
}

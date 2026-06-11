use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BezierSegment {
    MoveTo(Point2D),
    LineTo(Point2D),
    QuadraticTo(Point2D, Point2D),
    CubicTo(Point2D, Point2D, Point2D),
    Close,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct FourierTerm {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SplineEquation {
    pub start_t: f64,
    pub end_t: f64,
    // Coefficients for powers of t: a + bt + ct^2 + dt^3
    pub x_poly: Vec<f64>,
    pub y_poly: Vec<f64>,
}

use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)] // For backwards compatibility with old `color_rgb: [f32; 3]` JSON ASTs
pub enum ColorStyle {
    Solid([f32; 3]),
    LinearGradient {
        start: [f32; 3],
        end: [f32; 3],
        angle: f32, // Degrees
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ColoredPath<T> {
    #[serde(rename = "color_rgb")]
    pub color_style: Option<ColorStyle>,
    pub data: T,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MathExpressionAST {
    Fourier {
        strokes: Vec<ColoredPath<Vec<FourierTerm>>>,
        bounding_box: [f32; 4],
    },
    Spline {
        equations: Vec<ColoredPath<Vec<SplineEquation>>>,
        bounding_box: [f32; 4],
    },
    Polyline {
        paths: Vec<ColoredPath<Vec<Point2D>>>,
        bounding_box: [f32; 4],
    },
}

#[derive(Debug, Clone)]
pub enum ParserOutput {
    Paths {
        paths: Vec<ColoredPath<Vec<Point2D>>>,
        original_dimensions: (u32, u32),
    },
    Segments {
        segments: Vec<ColoredPath<Vec<BezierSegment>>>,
        original_dimensions: (u32, u32),
    },
}

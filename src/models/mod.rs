use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MathExpressionAST {
    Fourier { strokes: Vec<Vec<FourierTerm>> },
    Spline { equations: Vec<SplineEquation> },
}

#[derive(Debug, Clone)]
pub enum ParserOutput {
    Paths(Vec<Vec<Point2D>>),
    Segments(Vec<BezierSegment>),
}

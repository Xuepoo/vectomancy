use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct SafeF32(pub f32);

impl<'de> Deserialize<'de> for SafeF32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SafeF32Visitor;

        impl<'de> serde::de::Visitor<'de> for SafeF32Visitor {
            type Value = SafeF32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a float, a string representing NaN/Infinity, or null")
            }

            fn visit_bool<E>(self, _v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(0.0))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(v as f32))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(v as f32))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(v as f32))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "NaN" | "nan" | "NAN" => Ok(SafeF32(f32::NAN)),
                    "Infinity" | "inf" | "INF" => Ok(SafeF32(f32::INFINITY)),
                    "-Infinity" | "-inf" | "-INF" => Ok(SafeF32(f32::NEG_INFINITY)),
                    _ => match v.parse::<f32>() {
                        Ok(val) => Ok(SafeF32(val)),
                        Err(_) => Ok(SafeF32(f32::NAN)),
                    },
                }
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(f32::NAN))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SafeF32(f32::NAN))
            }
        }

        deserializer.deserialize_any(SafeF32Visitor)
    }
}

fn to_f32_clean(v: SafeF32) -> f32 {
    let val = v.0;
    if val.is_nan() {
        0.0
    } else if val.is_infinite() {
        if val.is_sign_positive() {
            f32::MAX
        } else {
            f32::MIN
        }
    } else {
        val
    }
}

fn clean_coord(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn process_gradient_fields(
    raw_stops: Vec<(SafeF32, [SafeF32; 3])>,
    angle: Option<SafeF32>,
    raw_start_pos: Option<[SafeF32; 2]>,
    raw_end_pos: Option<[SafeF32; 2]>,
) -> Result<GradientData, &'static str> {
    if raw_stops.is_empty() {
        return Err("LinearGradient stops cannot be empty");
    }

    let mut stops: Vec<(f32, [f32; 3])> = raw_stops
        .into_iter()
        .map(|(pos, color)| {
            let s_pos = to_f32_clean(pos).clamp(0.0, 1.0);
            let r = to_f32_clean(color[0]).clamp(0.0, 1.0);
            let g = to_f32_clean(color[1]).clamp(0.0, 1.0);
            let b = to_f32_clean(color[2]).clamp(0.0, 1.0);
            (s_pos, [r, g, b])
        })
        .collect();

    stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let start_pos = if let Some(sp) = raw_start_pos {
        [
            clean_coord(to_f32_clean(sp[0])),
            clean_coord(to_f32_clean(sp[1])),
        ]
    } else {
        [0.0, 0.5]
    };

    let end_pos = if let Some(ep) = raw_end_pos {
        [
            clean_coord(to_f32_clean(ep[0])),
            clean_coord(to_f32_clean(ep[1])),
        ]
    } else {
        let a = angle.map(to_f32_clean).unwrap_or(0.0);
        let rad = a.to_radians();
        let ep = if rad.is_finite() {
            [0.5 + rad.cos() * 0.5, 0.5 + rad.sin() * 0.5]
        } else {
            [1.0, 0.5]
        };
        [clean_coord(ep[0]), clean_coord(ep[1])]
    };

    Ok(GradientData {
        stops,
        start_pos,
        end_pos,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ColorStyle {
    Solid([f32; 3]),
    LinearGradient(Arc<GradientData>),
}

impl ColorStyle {
    pub fn to_solid_rgba(&self) -> [f32; 4] {
        match self {
            ColorStyle::Solid(rgb) => [rgb[0], rgb[1], rgb[2], 1.0],
            ColorStyle::LinearGradient(grad) => {
                let start = grad.stops.first().map(|s| s.1).unwrap_or([0.0, 0.0, 0.0]);
                [start[0], start[1], start[2], 1.0]
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GradientData {
    pub stops: Vec<(f32, [f32; 3])>,
    pub start_pos: [f32; 2],
    pub end_pos: [f32; 2],
}

#[derive(Deserialize)]
struct GradientDataRaw {
    stops: Vec<(SafeF32, [SafeF32; 3])>,
    #[serde(default)]
    angle: Option<SafeF32>,
    #[serde(default)]
    start_pos: Option<[SafeF32; 2]>,
    #[serde(default)]
    end_pos: Option<[SafeF32; 2]>,
}

impl<'de> Deserialize<'de> for GradientData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = GradientDataRaw::deserialize(deserializer)?;
        process_gradient_fields(raw.stops, raw.angle, raw.start_pos, raw.end_pos)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ColorStyleRaw {
    Solid([SafeF32; 3]),
    LinearGradient {
        stops: Vec<(SafeF32, [SafeF32; 3])>,
        angle: Option<SafeF32>,
        start_pos: Option<[SafeF32; 2]>,
        end_pos: Option<[SafeF32; 2]>,
    },
    LegacyLinearGradient {
        start: [SafeF32; 3],
        end: [SafeF32; 3],
        angle: Option<SafeF32>,
    },
}

impl<'de> Deserialize<'de> for ColorStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ColorStyleRaw::deserialize(deserializer)?;
        match raw {
            ColorStyleRaw::Solid(rgb) => {
                let r = to_f32_clean(rgb[0]).clamp(0.0, 1.0);
                let g = to_f32_clean(rgb[1]).clamp(0.0, 1.0);
                let b = to_f32_clean(rgb[2]).clamp(0.0, 1.0);
                Ok(ColorStyle::Solid([r, g, b]))
            }
            ColorStyleRaw::LinearGradient {
                stops,
                angle,
                start_pos,
                end_pos,
            } => {
                let grad = process_gradient_fields(stops, angle, start_pos, end_pos)
                    .map_err(serde::de::Error::custom)?;
                Ok(ColorStyle::LinearGradient(Arc::new(grad)))
            }
            ColorStyleRaw::LegacyLinearGradient { start, end, angle } => {
                let r1 = to_f32_clean(start[0]).clamp(0.0, 1.0);
                let g1 = to_f32_clean(start[1]).clamp(0.0, 1.0);
                let b1 = to_f32_clean(start[2]).clamp(0.0, 1.0);

                let r2 = to_f32_clean(end[0]).clamp(0.0, 1.0);
                let g2 = to_f32_clean(end[1]).clamp(0.0, 1.0);
                let b2 = to_f32_clean(end[2]).clamp(0.0, 1.0);

                let a = angle.map(to_f32_clean).unwrap_or(0.0);
                let rad = a.to_radians();

                let start_pos = [0.0, 0.5];
                let end_pos = if rad.is_finite() {
                    [
                        clean_coord(0.5 + rad.cos() * 0.5),
                        clean_coord(0.5 + rad.sin() * 0.5),
                    ]
                } else {
                    [1.0, 0.5]
                };

                let stops = vec![(0.0, [r1, g1, b1]), (1.0, [r2, g2, b2])];

                Ok(ColorStyle::LinearGradient(Arc::new(GradientData {
                    stops,
                    start_pos,
                    end_pos,
                })))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_deserialization() {
        let json = r#"[1.0, 0.5, 0.0]"#;
        let style: ColorStyle = serde_json::from_str(json).unwrap();
        match style {
            ColorStyle::Solid(rgb) => {
                assert_eq!(rgb, [1.0, 0.5, 0.0]);
            }
            _ => panic!("Expected ColorStyle::Solid"),
        }
    }

    #[test]
    fn test_legacy_linear_gradient_deserialization() {
        let json = r#"{"start": [1.0, 0.0, 0.0], "end": [0.0, 0.0, 1.0], "angle": 90.0}"#;
        let style: ColorStyle = serde_json::from_str(json).unwrap();
        match style {
            ColorStyle::LinearGradient(grad) => {
                assert_eq!(grad.stops.len(), 2);
                assert_eq!(grad.stops[0].0, 0.0);
                assert_eq!(grad.stops[0].1, [1.0, 0.0, 0.0]);
                assert_eq!(grad.stops[1].0, 1.0);
                assert_eq!(grad.stops[1].1, [0.0, 0.0, 1.0]);
                assert!((grad.start_pos[0] - 0.0).abs() < 1e-5);
                assert!((grad.start_pos[1] - 0.5).abs() < 1e-5);
                assert!((grad.end_pos[0] - 0.5).abs() < 1e-5);
                assert!((grad.end_pos[1] - 1.0).abs() < 1e-5);
            }
            _ => panic!("Expected ColorStyle::LinearGradient"),
        }
    }

    #[test]
    fn test_linear_gradient_deserialization() {
        let json = r#"{"stops": [[0.2, [1.0, 1.0, 1.0]], [0.0, [0.0, 0.0, 0.0]]], "start_pos": [0.1, 0.2], "end_pos": [0.9, 0.8]}"#;
        let style: ColorStyle = serde_json::from_str(json).unwrap();
        match style {
            ColorStyle::LinearGradient(grad) => {
                assert_eq!(grad.stops.len(), 2);
                assert_eq!(grad.stops[0].0, 0.0);
                assert_eq!(grad.stops[0].1, [0.0, 0.0, 0.0]);
                assert_eq!(grad.stops[1].0, 0.2);
                assert_eq!(grad.stops[1].1, [1.0, 1.0, 1.0]);
                assert_eq!(grad.start_pos, [0.1, 0.2]);
                assert_eq!(grad.end_pos, [0.9, 0.8]);
            }
            _ => panic!("Expected ColorStyle::LinearGradient"),
        }
    }

    #[test]
    fn test_nan_deserialization_protection() {
        use serde_json::json;

        // Test with string representations of NaN and Infinity
        let v_solid = json!(["NaN", 0.5, "Infinity"]);
        let style: ColorStyle = serde_json::from_value(v_solid).unwrap();
        match style {
            ColorStyle::Solid(rgb) => {
                assert_eq!(rgb[0], 0.0);
                assert_eq!(rgb[1], 0.5);
                assert_eq!(rgb[2], 1.0);
            }
            _ => panic!("Expected ColorStyle::Solid"),
        }

        // Test with raw nulls (which is what serde_json maps f32::NAN/f32::INFINITY to when serialized without quotes)
        let v_solid_null = json!([null, 0.5, null]);
        let style_null: ColorStyle = serde_json::from_value(v_solid_null).unwrap();
        match style_null {
            ColorStyle::Solid(rgb) => {
                assert_eq!(rgb[0], 0.0);
                assert_eq!(rgb[1], 0.5);
                assert_eq!(rgb[2], 0.0);
            }
            _ => panic!("Expected ColorStyle::Solid"),
        }

        // Test gradient with various NaN, Infinity, -Infinity, and null values
        let v_grad = json!({
            "stops": [
                ["NaN", [1.0, null, 0.0]],
                [0.5, [0.0, 0.0, "-Infinity"]]
            ],
            "angle": "NaN",
            "start_pos": [null, 0.0],
            "end_pos": [1.0, "Infinity"]
        });
        let style: ColorStyle = serde_json::from_value(v_grad).unwrap();
        match style {
            ColorStyle::LinearGradient(grad) => {
                assert_eq!(grad.stops.len(), 2);
                assert_eq!(grad.stops[0].0, 0.0);
                assert_eq!(grad.stops[0].1, [1.0, 0.0, 0.0]);
                assert_eq!(grad.stops[1].0, 0.5);
                assert_eq!(grad.stops[1].1, [0.0, 0.0, 0.0]);
                assert_eq!(grad.start_pos[0], 0.0);
                assert_eq!(grad.start_pos[1], 0.0);
                assert_eq!(grad.end_pos[0], 1.0);
                assert_eq!(grad.end_pos[1], 1.0);
            }
            _ => panic!("Expected ColorStyle::LinearGradient"),
        }
    }

    #[test]
    fn test_empty_stops_error() {
        let json = r#"{"stops": [], "start_pos": [0.0, 0.5], "end_pos": [1.0, 0.5]}"#;
        let res: Result<GradientData, _> = serde_json::from_str(json);
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("LinearGradient stops cannot be empty"));
    }

    #[test]
    fn test_omitted_start_pos_default() {
        let json =
            r#"{"stops": [[0.0, [0.0, 0.0, 0.0]], [1.0, [1.0, 1.0, 1.0]]], "end_pos": [1.0, 0.5]}"#;
        let style: ColorStyle = serde_json::from_str(json).unwrap();
        match style {
            ColorStyle::LinearGradient(grad) => {
                assert_eq!(grad.start_pos, [0.0, 0.5]);
            }
            _ => panic!("Expected ColorStyle::LinearGradient"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

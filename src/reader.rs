//! Types and functions for the parser internals.
use std::io::{BufRead, BufReader, Read};
use std::num::{ParseFloatError, ParseIntError};

use crate::Tsi;

pub const EXPECTED_VERSION: &str = "1.1";

/// The error type for problems while parsing a `tsi` file.
#[derive(Debug)]
pub enum TsiError {
    Io(std::io::Error),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Missing(MissingItem),
    InvalidVersion(String),
    IndexMismatch { found: u32, expected: u32, thing: &'static str },
    UnexpectedKeyword(String),
}

const fn missing_item_value(s: &'static str) -> TsiError {
    TsiError::Missing(MissingItem::Value(s))
}

/// Description of a missing item while parsing a `tsi` file.
#[derive(Debug)]
pub enum MissingItem {
    Value(&'static str),
    Definition(&'static str),
    Vertex(u32),
    Triangle(u32),
    Inclusion(u32),
    Exclusion(u32),
}

impl std::error::Error for TsiError {}

impl std::fmt::Display for TsiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::ParseInt(e) => write!(f, "integer parse error: {e}"),
            Self::ParseFloat(e) => write!(f, "float parse error: {e}"),
            Self::Missing(item) => write!(f, "missing data: {item}"),
            Self::InvalidVersion(found) => {
                write!(f, "unsupported version {found:?}, expected {EXPECTED_VERSION:?}")
            }
            Self::IndexMismatch { found, expected, thing } => {
                write!(f, "incorrect {thing} index: found {found}, expected {expected}")
            }
            Self::UnexpectedKeyword(k) => write!(f, "encountered unknown keyword: {k}"),
        }
    }
}

impl std::fmt::Display for MissingItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingItem::Value(value) => write!(f, "expected value for {value}"),
            MissingItem::Definition(value) => write!(f, "expected definition for {value}"),
            MissingItem::Vertex(idx) => write!(f, "vertex line for index {idx}"),
            MissingItem::Triangle(idx) => write!(f, "triangle line for index {idx}"),
            MissingItem::Inclusion(idx) => write!(f, "inclusion line for index {idx}"),
            MissingItem::Exclusion(idx) => write!(f, "exclusion line for index {idx}"),
        }
    }
}

impl From<std::io::Error> for TsiError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ParseIntError> for TsiError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<ParseFloatError> for TsiError {
    fn from(e: ParseFloatError) -> Self {
        Self::ParseFloat(e)
    }
}

trait ParseValue<T> {
    fn parse_value(self, desc: &'static str) -> Result<T, TsiError>;
}

impl<T: std::str::FromStr> ParseValue<T> for Option<&str>
where
    TsiError: From<<T as std::str::FromStr>::Err>,
{
    /// Shorthand notation for expecting some value and parsing it as a `T`.
    fn parse_value(self, desc: &'static str) -> Result<T, TsiError> {
        Ok(self.ok_or(missing_item_value(desc))?.parse()?)
    }
}

mod items {
    use super::{ParseValue, TsiError};
    use crate::{Exclusion, Inclusion, Triangle, Vertex};

    const fn check_index(thing: &'static str, found: u32, expected: u32) -> Result<(), TsiError> {
        if found == expected {
            Ok(())
        } else {
            Err(TsiError::IndexMismatch { found, expected, thing })
        }
    }

    pub fn parse_vertex_line(line: &str, expected_idx: u32) -> Result<Vertex, TsiError> {
        let mut words = line.split_whitespace();
        let found_idx = words.next().parse_value("vertex index")?;
        check_index("vertex", found_idx, expected_idx)?;

        let x = words.next().parse_value("vertex x")?;
        let y = words.next().parse_value("vertex y")?;
        let z = words.next().parse_value("vertex z")?;
        // The domain may be absent, implying it is set to 0.
        let domain = words.next().map(|v| v.parse()).transpose()?.unwrap_or(0);

        Ok(Vertex { position: [x, y, z], domain })
    }

    pub fn parse_triangle_line(line: &str, expected_idx: u32) -> Result<Triangle, TsiError> {
        let mut words = line.split_whitespace();
        let found_idx = words.next().parse_value("triangle index")?;
        check_index("triangle", found_idx, expected_idx)?;

        let a = words.next().parse_value("first triangle vertex index")?;
        let b = words.next().parse_value("second triangle vertex index")?;
        let c = words.next().parse_value("third triangle vertex index")?;

        Ok(Triangle { vertices: [a, b, c] })
    }

    pub fn parse_inclusion_line(line: &str, expected_idx: u32) -> Result<Inclusion, TsiError> {
        let mut words = line.split_whitespace();
        let found_idx = words.next().parse_value("inclusion index")?;
        check_index("inclusion", found_idx, expected_idx)?;

        let ty = words.next().parse_value("inclusion type")?;
        let vertex_index = words.next().parse_value("inclusion vertex index")?;
        let x: f32 = words.next().parse_value("inclusion vector x")?;
        let y: f32 = words.next().parse_value("inclusion vector y")?;
        let norm = f32::sqrt(x.powi(2) + y.powi(2));
        let vector = if norm > 0.0 { [x / norm, y / norm] } else { [0.0, 0.0] };

        Ok(Inclusion { ty, vertex_index, vector })
    }

    pub fn parse_exclusion_line(line: &str, expected_idx: u32) -> Result<Exclusion, TsiError> {
        let mut words = line.split_whitespace();
        let found_idx = words.next().parse_value("exclusion index")?;
        check_index("exclusion", found_idx, expected_idx)?;

        let vertex_index = words.next().parse_value("exclusion vertex index")?;
        let radius = words.next().parse_value("exclusion radius")?;

        Ok(Exclusion { vertex_index, radius })
    }
}

pub trait ReadTsi {
    fn parse(reader: impl Read) -> Result<Self, TsiError>
    where
        Self: Sized;
}

impl ReadTsi for Tsi {
    fn parse(reader: impl Read) -> Result<Self, TsiError> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        let mut version = None;
        let mut dimensions = None;
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        let mut inclusions = Vec::new();
        let mut exclusions = Vec::new();

        while let Some(line_result) = lines.next() {
            let line = line_result?;
            let mut words = line.split_whitespace();
            let keyword = match words.next() {
                Some(k) => k,
                None => return Err(missing_item_value("section keyword")),
            };

            match keyword {
                "version" => {
                    version =
                        Some(words.next().ok_or(missing_item_value("version tag"))?.to_string());
                }
                "box" => {
                    let x = words.next().parse_value("box x")?;
                    let y = words.next().parse_value("box y")?;
                    let z = words.next().parse_value("box z")?;
                    dimensions = Some([x, y, z]);
                }
                "vertex" => {
                    let n: u32 = words.next().parse_value("vertex count")?;
                    vertices = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line =
                            lines.next().ok_or(TsiError::Missing(MissingItem::Vertex(idx)))??;
                        let vertex = items::parse_vertex_line(&line, idx)?;
                        vertices.push(vertex);
                    }
                }
                "triangle" => {
                    let n: u32 = words.next().parse_value("triangle count")?;
                    triangles = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line =
                            lines.next().ok_or(TsiError::Missing(MissingItem::Triangle(idx)))??;
                        let triangle = items::parse_triangle_line(&line, idx)?;
                        triangles.push(triangle);
                    }
                }
                "inclusion" => {
                    let n: u32 = words.next().parse_value("inclusion count")?;
                    inclusions = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line = lines
                            .next()
                            .ok_or(TsiError::Missing(MissingItem::Inclusion(idx)))??;
                        let inclusion = items::parse_inclusion_line(&line, idx)?;
                        inclusions.push(inclusion);
                    }
                }
                "exclusion" => {
                    let n: u32 = words.next().parse_value("exclusion count")?;
                    exclusions = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line = lines
                            .next()
                            .ok_or(TsiError::Missing(MissingItem::Exclusion(idx)))??;
                        let exclusion = items::parse_exclusion_line(&line, idx)?;
                        exclusions.push(exclusion);
                    }
                }
                unknown => return Err(TsiError::UnexpectedKeyword(unknown.to_string())),
            }
        }

        match version {
            Some(version) if version == EXPECTED_VERSION => {}
            Some(found) => return Err(TsiError::InvalidVersion(found)),
            None => return Err(TsiError::Missing(MissingItem::Definition("version"))),
        }

        let dimensions = dimensions.ok_or(TsiError::Missing(MissingItem::Definition("box")))?;

        Ok(Tsi { dimensions, vertices, triangles, inclusions, exclusions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const VALID_TSI: &str = "\
version 1.1
box 50.0 50.0 50.0
vertex 3
0 21.4 33.8 32.7 0
1 38.1 26.1 32.3 0
2 40.9 24.2 19.9 0
triangle 1
0 1 0 2 1
inclusion 1
0 1 2 0 1";

    #[test]
    fn parse_valid_sample() {
        let result = Tsi::parse(Cursor::new(VALID_TSI));
        assert!(result.is_ok(), "Should parse valid TSI data: {:?}", result.err());
        let tsi = result.unwrap();
        assert_eq!(tsi.dimensions, [50.0, 50.0, 50.0]);
        assert_eq!(tsi.vertices.len(), 3);
        assert_eq!(tsi.inclusions.len(), 1);
    }

    #[test]
    fn normalization_safety() {
        // Testing 0.0 0.0 vector to ensure no NaN propagation.
        let zero_vector_line = "0 1 2 0.0 0.0";
        let inclusion = items::parse_inclusion_line(zero_vector_line, 0).unwrap();
        assert_eq!(inclusion.vector, [0.0, 0.0]);
        assert!(!inclusion.vector[0].is_nan());
    }

    #[test]
    fn index_mismatch() {
        let bad_index_line = "5 21.4 33.8 32.7 0"; // Expected 0, found 5.
        let result = items::parse_vertex_line(bad_index_line, 0);
        match result {
            Err(TsiError::IndexMismatch { found, expected, .. }) => {
                assert_eq!(found, 5);
                assert_eq!(expected, 0);
            }
            _ => panic!("Expected IndexMismatch error"),
        }
    }

    #[test]
    fn invalid_version() {
        let invalid_version = VALID_TSI.replace("version 1.1", "version 2.0");
        let result = Tsi::parse(Cursor::new(invalid_version));
        assert!(matches!(result, Err(TsiError::InvalidVersion(_))));
    }
}

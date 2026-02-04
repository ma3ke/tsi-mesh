use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Tsi {
    /// Box dimensions in nm.
    dimensions: [f32; 3],
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    inclusions: Vec<Inclusion>,
    exclusions: Vec<Exclusion>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    position: [f32; 3],
    domain: i32,
}

// In the TS2CG implementation, this is an `int`.
type VertexIndex = u32;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub struct Triangle {
    vertices: [VertexIndex; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Inclusion {
    ty: i32,
    vertex_index: VertexIndex,
    vector: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Exclusion {
    vertex_index: VertexIndex,
    radius: f32,
}

impl Tsi {
    pub fn parse(reader: impl Read) -> std::io::Result<Self> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        let mut version = None;
        let mut dimensions = None;
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        let mut inclusions = Vec::new();
        let mut exclusions = Vec::new();
        loop {
            let Some(line) = lines.next().transpose()? else { break };
            let mut words = line.split_whitespace();
            let keyword = words.next().expect("expected keyword");

            match keyword {
                "version" => version = Some(words.next().expect("tsi version tag").to_string()),
                "box" => {
                    let x = words
                        .next()
                        .expect("box dimensions x value")
                        .parse()
                        .expect("could not parse box dimensions x value");
                    let y = words
                        .next()
                        .expect("box dimensions y value")
                        .parse()
                        .expect("could not parse box dimensions y value");
                    let z = words
                        .next()
                        .expect("box dimensions z value")
                        .parse()
                        .expect("could not parse box dimensions z value");
                    dimensions = Some([x, y, z]);
                }

                // Find out what section is coming up.
                "vertex" => {
                    let n = words
                        .next()
                        .expect("number of vertices")
                        .parse()
                        .expect("could not parse number of vertices");
                    vertices = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line = lines.next().expect("vertex line")?;
                        let mut words = line.split_whitespace();
                        let found_idx = words
                            .next()
                            .expect("vertex index")
                            .parse()
                            .expect("could not parse vertex index");
                        assert_eq!(
                            idx, found_idx,
                            "incorrectly indexed vertex: found {found_idx}, expected {idx}"
                        );
                        let x = words
                            .next()
                            .expect("vertex position x value")
                            .parse()
                            .expect("could not parse vertex position x value");
                        let y = words
                            .next()
                            .expect("vertex position y value")
                            .parse()
                            .expect("could not parse vertex position y value");
                        let z = words
                            .next()
                            .expect("vertex position z value")
                            .parse()
                            .expect("could not parse vertex position z value");
                        let position = [x, y, z];
                        let domain = words
                            .next()
                            .map(|v| v.parse().expect("could not parse vertex domain value"))
                            .unwrap_or_default();
                        vertices.push(Vertex { position, domain });
                    }
                }
                "triangle" => {
                    let n = words
                        .next()
                        .expect("number of triangles")
                        .parse()
                        .expect("could not parse number of triangles");
                    {
                        triangles = Vec::with_capacity(n as usize);
                        for idx in 0..n {
                            let line = lines.next().expect("triangle line")?;
                            let mut words = line.split_whitespace();
                            let found_idx = words
                                .next()
                                .expect("triangle index")
                                .parse()
                                .expect("could not parse triangle index");
                            assert_eq!(
                                idx, found_idx,
                                "incorrectly indexed triangle: found {found_idx}, expected {idx}"
                            );
                            let a = words
                                .next()
                                .expect("triangle vertex index")
                                .parse()
                                .expect("could not parse triangle vertex index");
                            let b = words
                                .next()
                                .expect("second triangle vertex index")
                                .parse()
                                .expect("could not parse second triangle vertex index");
                            let c = words
                                .next()
                                .expect("third triangle vertex index")
                                .parse()
                                .expect("could not parse third triangle vertex index");
                            let vertices = [a, b, c];
                            triangles.push(Triangle { vertices });
                        }
                    }
                }
                "inclusion" => {
                    let n = words
                        .next()
                        .expect("number of inclusions")
                        .parse()
                        .expect("could not parse number of inclusions");
                    inclusions = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line = lines.next().expect("inclusion line")?;
                        let mut words = line.split_whitespace();
                        let found_idx = words
                            .next()
                            .expect("inclusion index")
                            .parse()
                            .expect("could not parse inclusion index");
                        assert_eq!(
                            idx, found_idx,
                            "incorrectly indexed inclusion: found {found_idx}, expected {idx}"
                        );
                        let ty = words
                            .next()
                            .expect("inclusion type index")
                            .parse()
                            .expect("could not parse inclusion type index");
                        let vertex_index = words
                            .next()
                            .expect("inclusino vertex index")
                            .parse()
                            .expect("could not parse inclusino vertex index");
                        let x = words
                            .next()
                            .expect("inclusion vector x value")
                            .parse::<f32>()
                            .expect("could not parse inclusion vector x value");
                        let y = words
                            .next()
                            .expect("inclusion vector x value")
                            .parse::<f32>()
                            .expect("could not parse inclusion vector y value");
                        let norm = f32::sqrt(x.powi(2) + y.powi(2));
                        let vector = [x / norm, y / norm];
                        inclusions.push(Inclusion { ty, vertex_index, vector });
                    }
                }
                "exclusion" => {
                    let n = words
                        .next()
                        .expect("number of exclusions")
                        .parse()
                        .expect("could not parse number of exclusions");
                    exclusions = Vec::with_capacity(n as usize);
                    for idx in 0..n {
                        let line = lines.next().expect("exclusion line")?;
                        let mut words = line.split_whitespace();
                        let found_idx = words
                            .next()
                            .expect("exclusion vertex index")
                            .parse()
                            .expect("could not parse inclusion exclusion index");
                        assert_eq!(
                            idx, found_idx,
                            "incorrectly indexed exclusion: found {found_idx}, expected {idx}"
                        );
                        let vertex_index = words
                            .next()
                            .expect("vertex index")
                            .parse()
                            .expect("could not parse exclusion vertex index");
                        let radius = words
                            .next()
                            .expect("exclusion exclusion radius")
                            .parse()
                            .expect("could not parse exclusion radius");
                        exclusions.push(Exclusion { vertex_index, radius });
                    }
                }
                unknown => panic!("encountered unknown keyword: {unknown}"),
            }
        }

        const VERSION: &str = "1.1";
        match version {
            Some(version) if version == VERSION => {}
            Some(version) => {
                panic!("found unsupported version {version}, expected version {VERSION}")
            }
            None => panic!("version must be specified, expected version {VERSION}"),
        }
        Ok(Tsi {
            dimensions: dimensions.expect("box dimensions must be specified"),
            vertices,
            triangles,
            inclusions,
            exclusions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let src = "version 1.1
box 50.000     50.000     50.000
vertex 3
0       21.4    33.8    32.7    0
1       38.1    26.1    32.3    0
2       40.9    24.2    19.9    0
triangle 1
0          1       2       0    1
inclusion 3
0         1       22       0    1
1         1        5       0    1
2         2       30       0    1";
        let tsi = Tsi::parse(src.as_bytes()).unwrap();
        dbg!(tsi);
        panic!();
    }
}

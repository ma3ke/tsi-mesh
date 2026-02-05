use crate::{Exclusion, Inclusion, Triangle, Tsi, Vertex};
use std::io::{Result, Write};

/// Round values to sufficient spatial precision.
const fn round_to_precision(v: f32) -> f32 {
    // A 1/1000th of a nanometer ought to be enough.
    const PRECISION: f32 = 1e3;
    (v * PRECISION).round() / PRECISION
}

pub trait WriteTsi {
    fn write(&self, writer: impl Write) -> Result<()>;
}

impl WriteTsi for Tsi {
    fn write(&self, mut writer: impl Write) -> Result<()> {
        // Front matter.
        {
            writeln!(writer, "version 1.1")?;
            let [x, y, z] = self.dimensions.map(round_to_precision);
            writeln!(writer, "box {x} {y} {z}",)?;
        }

        // Vertices.
        writeln!(writer, "vertex {}", self.vertices.len())?;
        for (i, vertex) in self.vertices.iter().enumerate() {
            let Vertex { position, domain } = vertex;
            let [x, y, z] = position.map(round_to_precision);
            writeln!(writer, "{i} {x} {y} {z} {domain}")?;
        }

        // Triangles.
        writeln!(writer, "triangle {}", self.triangles.len())?;
        for (i, triangle) in self.triangles.iter().enumerate() {
            let Triangle { vertices: [v1, v2, v3] } = triangle;
            writeln!(writer, "{i} {v1} {v2} {v3}")?;
        }

        // Inclusions.
        writeln!(writer, "inclusion {}", self.inclusions.len())?;
        for (i, inclusion) in self.inclusions.iter().enumerate() {
            let Inclusion { ty, vertex_index, vector } = inclusion;
            let [vx, vy] = vector.map(round_to_precision);
            writeln!(writer, "{i} {ty} {vertex_index} {vx} {vy}",)?;
        }

        // Exclusions.
        writeln!(writer, "exclusion {}", self.exclusions.len())?;
        for (i, &exclusion) in self.exclusions.iter().enumerate() {
            let Exclusion { vertex_index, radius } = exclusion;
            let radius = round_to_precision(radius);
            writeln!(writer, "{i} {vertex_index} {radius}",)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::ReadTsi;
    use crate::{Exclusion, Inclusion, Triangle, Tsi, Vertex, WriteTsi};
    use std::io::Cursor;

    fn create_sample_tsi() -> Tsi {
        Tsi {
            dimensions: [50.0, 50.0, 50.0],
            vertices: vec![
                Vertex { position: [21.4, 33.8, 32.7], domain: 0 },
                Vertex { position: [38.1, 26.1, 32.3], domain: 0 },
            ],
            triangles: vec![Triangle { vertices: [0, 1, 2] }],
            inclusions: vec![Inclusion { ty: 1, vertex_index: 2, vector: [0.0, 1.0] }],
            exclusions: vec![Exclusion { vertex_index: 0, radius: 5.0 }],
        }
    }

    #[test]
    fn writer_formatting() {
        let tsi = create_sample_tsi();
        let mut buffer = Vec::new();

        tsi.write(&mut buffer).unwrap(); // Safe, because writing to a Vec can't fail.
        let output = String::from_utf8(buffer).expect("output should be valid UTF-8");

        // Verify front matter.
        assert!(output.contains("version 1.1"));
        assert!(output.contains("box 50 50 50"));

        // Verify triangle format (v1 v2 v3).
        assert!(output.contains("0 0 1 2"));

        // Verify exclusion precision.
        assert!(output.contains("0 0 5"));

        // Also verify how more precise values are written out.
        let mut tsi = tsi;
        tsi.dimensions = [50.12, 50.123, 50.123456];

        let mut buffer = Vec::new();
        tsi.write(&mut buffer).unwrap(); // Safe, because writing to a Vec can't fail.
        let output = String::from_utf8(buffer).expect("output should be valid UTF-8");

        assert!(output.contains("box 50.12 50.123 50.123"));
    }

    /// Prove that `parse(write(data)) == data`.
    #[test]
    fn tsi_round_trip() {
        let original = create_sample_tsi();

        // Write to memory.
        let mut buffer = Vec::new();
        original.write(&mut buffer).unwrap(); // Safe, because writing to a Vec can't fail.

        // Read back from memory.
        let reader = Cursor::new(buffer);
        let recovered = Tsi::parse(reader).expect("failed to parse back serialized data");

        assert_eq!(original.dimensions, recovered.dimensions);
        assert_eq!(original.vertices.len(), recovered.vertices.len());
        assert_eq!(original.triangles.len(), recovered.triangles.len());
        assert_eq!(original.inclusions.len(), recovered.inclusions.len());
        assert_eq!(original.exclusions.len(), recovered.exclusions.len());

        // Detailed check on specific values
        assert_eq!(original.vertices[0].position, recovered.vertices[0].position);
        assert_eq!(original.inclusions[0].vector, recovered.inclusions[0].vector);
    }
}

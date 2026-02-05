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

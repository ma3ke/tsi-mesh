use crate::{Exclusion, Inclusion, Triangle, Tsi, Vertex};
use std::io::{Result, Write};

pub trait WriteTsi {
    fn write(&self, writer: impl Write) -> Result<()>;
}

impl WriteTsi for Tsi {
    fn write(&self, mut writer: impl Write) -> Result<()> {
        // Front matter.
        {
            writeln!(writer, "version 1.1")?;
            let [x, y, z] = self.dimensions;
            writeln!(writer, "box {x:.3} {y:.3} {z:.3}",)?;
        }

        // Vertices.
        writeln!(writer, "vertex {}", self.vertices.len())?;
        for (i, vertex) in self.vertices.iter().enumerate() {
            let Vertex { position: [x, y, z], domain } = vertex;
            writeln!(writer, "{i} {x:.3} {y:.3} {z:.3} {domain}")?;
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
            let Inclusion { ty, vertex_index, vector: [vx, vy] } = inclusion;
            writeln!(writer, "{i} {ty} {vertex_index} {vx} {vy}",)?;
        }

        // Exclusions.
        writeln!(writer, "exclusion {}", self.exclusions.len())?;
        for (i, exclusion) in self.exclusions.iter().enumerate() {
            let Exclusion { vertex_index, radius } = exclusion;
            writeln!(writer, "{i} {vertex_index} {radius:.3}",)?;
        }

        Ok(())
    }
}

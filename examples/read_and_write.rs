use std::io::{BufReader, BufWriter};

use tsi::{ReadTsi, Tsi, WriteTsi, reader::TsiError};

fn main() -> Result<(), TsiError> {
    let path = "membrane.tsi";
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    // Read a tsi file.
    let mesh = Tsi::parse(reader)?;

    println!("Successfully parsed {path:?}.");
    println!("         box: {:?} nm", mesh.dimensions);
    println!("    vertices: {}", mesh.vertices.len());
    println!("   triangles: {}", mesh.triangles.len());
    println!("  inclusions: {}", mesh.inclusions.len());
    println!("  exclusions: {}", mesh.exclusions.len());

    // Change some value, say the dimensions.
    let mesh = {
        let mut mesh = mesh;
        for dim in &mut mesh.dimensions {
            *dim *= 2.0;
        }
        mesh
    };

    // Write a tsi-formatted string.
    let mut buffer = Vec::new();
    mesh.write(&mut buffer)?;
    let s = String::from_utf8(buffer).expect("should be valid UTF-8");
    println!("{s}");

    // Or to a file, directly.
    let file = std::fs::File::create("output.tsi")?;
    let mut writer = BufWriter::new(file);
    mesh.write(&mut writer)?;

    Ok(())
}

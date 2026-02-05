use std::io::{BufReader, BufWriter};

use tsi::{ReadTsi, Tsi, WriteTsi, reader::TsiError};

fn main() -> Result<(), TsiError> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or("membrane.tsi".to_string());
    let file = std::fs::File::open(&path)?;
    let reader = BufReader::new(file);

    // Read a tsi file.
    let start = std::time::Instant::now();
    let mesh = Tsi::parse(reader)?;

    let seconds = start.elapsed().as_secs_f64();
    println!("Successfully parsed {path:?} in {seconds:.3} s.");
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
    let start = std::time::Instant::now();
    mesh.write(&mut buffer)?;
    let s = String::from_utf8(buffer).expect("should be valid UTF-8");
    let nlines = s.lines().count();
    let seconds = start.elapsed().as_secs_f64();
    println!("Successfully serialized {nlines} lines in {seconds:.3} s.");

    // Or to a file, directly.
    let path = &args.next().unwrap_or("output.tsi".to_string());
    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    let start = std::time::Instant::now();
    mesh.write(&mut writer)?;
    let seconds = start.elapsed().as_secs_f64();
    println!("Successfully wrote to {path:?} in {seconds:.3} s.");

    Ok(())
}

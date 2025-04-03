use core::f32;
use std::{fs, path::Path, thread::sleep, time::Duration};

use anyhow::Result;
use pcd_rs::{PcdDeserialize, Reader};
use rerun::{Color, Points3D, RecordingStreamBuilder};

#[derive(Debug, PcdDeserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// HSVからRGBへの変換関数
/// h: Hue (0.0〜360.0), s: Saturation (0.0〜1.0), v: Value (0.0〜1.0)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    [
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    ]
}

fn load_pcd(path: &str) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    let reader = match Reader::open(path) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return Err(e.into());
        }
    };

    let points: Vec<Point> = match reader.collect() {
        Ok(points) => points,
        Err(e) => {
            eprintln!("Error reading points: {}", e);
            return Err(e.into());
        }
    };
    // println!("There are {} points", points.len());

    Ok(points)
}

fn load_pcd_paths(dir_path: &str, data_type: &str) -> Result<Vec<String>, anyhow::Error> {
    let mut paths: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir_path) {
        println!("Reading directory: {}", dir_path);
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                // println!("Data: {}", path.to_str().unwrap());

                if let Some(ext) = path.extension() {
                    if ext == "pcd" {
                        if let Some(path_str) = path.to_str() {
                            paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Error reading directory: {}", dir_path);
        return Err(anyhow::anyhow!("Failed to read directory").into());
    }

    paths.sort_by_key(|path| {
        let filename = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        filename
            .strip_prefix(&format!("{}_", data_type))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    });
    // println!("Sorted paths: {:?}", paths);

    Ok(paths)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cr_paths = match load_pcd_paths(
        "/Users/kenji/workspace/Rust/rerun-sample/data/cr/voxelization",
        "cr",
    ) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error loading paths: {}", e);
            return Err(e.into());
        }
    };

    // let laser_map_paths = match load_pcd_paths(
    //     "/Users/kenji/workspace/Rust/rerun-sample/data/Laser_map/voxelization",
    //     "Laser_map",
    // ) {
    //     Ok(paths) => paths,
    //     Err(e) => {
    //         eprintln!("Error loading paths: {}", e);
    //         return Err(e.into());
    //     }
    // };

    let mut cr_points: Vec<Points3D> = Vec::new();
    let mut laser_map_points: Vec<Points3D> = Vec::new();

    /*  Transform .pcd to Points3D */
    // For cloud_registered
    for path in cr_paths.iter() {
        let points_vec = match load_pcd(path) {
            Ok(points) => points,
            Err(e) => {
                eprintln!("Error loading PCD file: {}", e);
                return Err(e.into());
            }
        };

        let points_tuple: Vec<(f32, f32, f32)> =
            points_vec.iter().map(|pt| (pt.x, pt.y, pt.z)).collect();

        cr_points.push(Points3D::new(points_tuple.into_iter()));
    }

    // For Laser_map
    match load_pcd(
        "/Users/kenji/workspace/Rust/rerun-sample/data/Laser_map/voxelization/Laser_map_110.pcd",
    ) {
        Ok(points_vec) => {
            let points_tuple: Vec<(f32, f32, f32)> =
                points_vec.iter().map(|pt| (pt.x, pt.y, pt.z)).collect();

            laser_map_points.push(Points3D::new(points_tuple.into_iter()));
        }
        Err(e) => {
            eprintln!("Error loading PCD file: {}", e);
            return Err(e.into());
        }
    };

    let rec = RecordingStreamBuilder::new("rerun_example_pcd").spawn()?;

    println!("Sending points to Rerun...");
    rec.log(
        "Laser_map",
        &laser_map_points[0]
            .clone()
            .with_colors([Color::from_rgb(0, 255, 0)])
            .with_radii([0.005]),
    )?;

    for cr_point in cr_points.iter() {
        rec.log(
            "cloud_registered",
            &cr_point
                .clone()
                .with_colors([Color::from_rgb(255, 0, 255)])
                .with_radii([0.03]),
        )?;

        sleep(Duration::from_millis(300));
    }
    println!("Finished to send points to Rerun...");

    Ok(())
}

use core::f32;

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
    println!("There are {} points", points.len());

    Ok(points)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pcd_path01 =
        "/home/kenji/workspace/Rust/pcd-sample/data/Laser_map/voxelization/Laser_map_45.pcd";

    let pcd_path02 = "/home/kenji/workspace/Rust/pcd-sample/data/cr/voxelization/cr_45.pcd";

    let points_01 = match load_pcd(pcd_path01) {
        Ok(points) => points,
        Err(e) => {
            eprintln!("Error loading PCD file: {}", e);
            return Err(e.into());
        }
    };

    let points_02 = match load_pcd(pcd_path02) {
        Ok(points) => points,
        Err(e) => {
            eprintln!("Error loading PCD file: {}", e);
            return Err(e.into());
        }
    };

    let point_cloud01: Vec<(f32, f32, f32)> =
        points_01.iter().map(|pt| (pt.x, pt.y, pt.z)).collect();

    let point_cloud02: Vec<(f32, f32, f32)> =
        points_02.iter().map(|pt| (pt.x, pt.y, pt.z)).collect();

    let min_z = points_01
        .iter()
        .map(|pt| pt.z)
        .fold(f32::INFINITY, f32::min);
    let max_z = points_01
        .iter()
        .map(|pt| pt.z)
        .fold(f32::NEG_INFINITY, f32::max);

    let point_cloud_color: Vec<[u8; 3]> = points_01
        .iter()
        .map(|pt| {
            // z座標を [0, 1] に正規化。minとmaxが同じ場合は0.0とする
            let t = if max_z - min_z == 0.0 {
                0.0
            } else {
                (pt.z - min_z) / (max_z - min_z)
            };
            // 正規化値 t をもとに、Hue を 0〜360 度に割り当てる
            let hue = t * 360.0;
            // 彩度と明度は 1.0 固定で鮮やかな色を得る
            hsv_to_rgb(hue, 0.9, 1.0)
        })
        .collect();

    let positions01 = Points3D::new(point_cloud01.into_iter());
    let positions02 = Points3D::new(point_cloud02.into_iter());

    let rec = RecordingStreamBuilder::new("rerun_example_pcd").spawn()?;

    rec.log(
        "pcd",
        &positions01.with_colors(
            point_cloud_color
                .into_iter()
                .map(|color| Color::from_rgb(color[0], color[1], color[2])),
        ),
    )?;

    rec.log(
        "pcd02",
        &positions02.with_colors([Color::from_rgb(255, 0, 0)]),
    )?;

    Ok(())
}

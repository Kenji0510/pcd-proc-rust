use pcd_rs::DynReader;

fn main() {
    let reader = DynReader::open(
        "/home/kenji/workspace/Rust/pcd-sample/data/Laser_map/voxelization/Laser_map_45.pcd",
    )
    .unwrap();

    let points: Result<Vec<_>, _> = reader.collect();
    println!("There are {} points", points.unwrap().len());
}

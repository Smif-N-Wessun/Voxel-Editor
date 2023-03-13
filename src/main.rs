mod octree;
mod camera;
mod mouse;
mod cursor;
mod vulkan;

use nalgebra::Vector3;
use octree::Octree;
use vulkan::App;

fn main() {
    let voxels = vec![
        Vector3::new(8.0, 8.0, 8.0),
        Vector3::new(15.0, 8.0, 9.0),
        Vector3::new(8.0, 15.0, 10.0),
        Vector3::new(15.0, 15.0, 11.0),
        Vector3::new(11.0, 12.0, 12.0),
        Vector3::new(12.0, 11.0, 16.0)
    ];

    let octree = Octree::new(Vector3::new(8, 8, 8), 16, voxels);

    let app = App::new();
    app.prepare(octree);
    app.run();
}
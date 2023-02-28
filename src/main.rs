mod octree;
mod camera;
mod vulkan;

use nalgebra::Vector3;
use vulkan::App;
use octree::Octree;

fn main() {
    let voxels = vec![
        Vector3::new(8.0, 8.0, 8.0),
        Vector3::new(8.0, 15.0, 8.0),
        Vector3::new(15.0, 15.0, 8.0),
        Vector3::new(15.0, 8.0, 8.0)
    ];

    let app = App::new();
    app.prepare(Octree::new(Vector3::new(8, 8, 8), 16, voxels));
    app.run();
}
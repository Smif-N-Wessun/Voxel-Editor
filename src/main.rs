mod octree;
mod camera;
mod mouse;
mod cursor;
mod vulkan;

use vulkan::App;

fn main() {
    //let voxels = vec![
    //    Vector3::new(8.0, 8.0, 8.0),
    //    Vector3::new(8.0, 15.0, 8.0),
    //    Vector3::new(15.0, 15.0, 8.0),
    //    Vector3::new(15.0, 8.0, 8.0)
    //];

    let app = App::new();
    app.prepare();
    app.run();
}
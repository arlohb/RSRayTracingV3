use crate::ray_tracer::*;

/// Holds the world space positions of the edges of the viewport.
struct ViewportPlane {
  pub left: Vec3,
  pub right: Vec3,
  pub bottom: Vec3,
  pub top: Vec3,
}

impl ViewportPlane {
  pub fn center(&self) -> Vec3 {
    (self.left + self.right) / 2.
  }
}

/// Holds data about the viewport used to make the rays.
pub struct Viewport {
  // these are stored in the camera struct, but are stored here for convenience
  pub camera_position: Vec3,
  pub camera_vectors: (Vec3, Vec3, Vec3), // (forward, right, up)

  pub top_left: Vec3,
  pub width: f64,
  pub height: f64,
}

impl Viewport {
  pub fn new(camera: &Camera, aspect_ratio: f64) -> Viewport {
    let (forward, right, up) = camera.get_vectors_fru();

    // calculate the viewport plane
    let viewport_plane = {
      // working for this is in whiteboard
      let fov_rad = camera.fov * (std::f64::consts::PI / 180.);
      let width = 2. * f64::tan(fov_rad / 2.);
      let half_width = width / 2.;

      let height = width * aspect_ratio;
      let half_height = height / 2.;

      // the image plane is 1 unit away from the camera
      // this is - not + because the camera point in the -forward direction
      let center = camera.position - forward;

      ViewportPlane {
        left: center - (right * half_width),
        right: center + (right * half_width),
        bottom: center - (up * half_height),
        top: center + (up * half_height),
      }
    };

    // calculate the needed things

    let top_left = viewport_plane.left + viewport_plane.top - viewport_plane.center();

    let width = (viewport_plane.right - viewport_plane.left).length();
    let height = (viewport_plane.top - viewport_plane.bottom).length();

    Viewport {
      camera_position: camera.position,
      camera_vectors: (forward, right, up),
      top_left,
      width,
      height,
    }
  }

  /// Creates a ray from the camera origin to the given point on the viewport.
  pub fn create_ray(&self, x_screen_space: f64, y_screen_space: f64) -> Ray {
    let x_offset = self.camera_vectors.1 * (x_screen_space * self.width);
    // mul -1 because it's offset down
    let y_offset = -self.camera_vectors.2 * (y_screen_space * self.height);

    let pixel_world_space = self.top_left + x_offset + y_offset;

    let direction = (pixel_world_space - self.camera_position).normalize();

    Ray {
      origin: self.camera_position,
      direction
    }
  }
}

use super::{Camera, Geometry, Light, Material, Object, Vec3};
use crate::utils::bytes::{
    bytes_concat_fixed_in_n_iter, bytes_concat_n, tuple_bytes, AsBytes as _,
};
use rand::{Rng, SeedableRng};
use rand_distr::Distribution;

/// Stores all the information about a scene
#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
    pub background_colour: (f32, f32, f32),
    pub ambient_light: (f32, f32, f32),
    pub reflection_limit: u32,
    pub do_objects_spin: bool,
}

impl Scene {
    pub const BUFFER_SIZE: (usize, usize, usize) =
        (Object::BUFFER_SIZE * 80, Light::BUFFER_SIZE * 2, 112);

    /// Returns a simple scene with a single sphere and light
    #[must_use]
    pub fn simple() -> Self {
        Self {
            objects: vec![Object::new(
                "Sphere",
                Material {
                    colour: (1., 0., 0.),
                    emission: (0., 0., 0.),
                    emission_strength: 0.,
                    roughness: 0.5,
                    metallic: 1.,
                },
                Geometry::Sphere {
                    center: Vec3::new(0., 0., 0.),
                    radius: 1.,
                },
            )],
            lights: vec![Light::Point {
                intensity: (1., 1., 1.),
                position: Vec3::new(0., 2., 0.),
            }],
            camera: Camera {
                position: Vec3::new(0., 0., -5.),
                rotation: Vec3::new(0., 0., 0.),
                fov: 70.,
            },
            background_colour: (0.5, 0.8, 1.),
            ambient_light: (0.2, 0.2, 0.2),
            reflection_limit: 4,
            do_objects_spin: false,
        }
    }

    /// Randomly fills a scene with spheres using default parameters.
    ///
    /// This will be more / less intensive depending on how many CPU cores are available.
    #[must_use]
    pub fn random_spheres_default_config() -> Self {
        Self::random_spheres(
            3.,
            8.,
            if num_cpus::get() > 2 { 50. } else { 20. },
            if num_cpus::get() > 2 { 100 } else { 5 },
            Some(42),
        )
    }

    #[must_use]
    pub fn random_sphere<R: Rng>(
        mut rng: &mut R,
        name: impl Into<String>,
        min_radius: f32,
        max_radius: f32,
        placement_radius: f32,
        is_valid: impl Fn(&Geometry) -> bool,
    ) -> Option<Object> {
        // if it failed 100 times, then there's probably no space left
        for _ in 0..100 {
            let radius: f32 = rng
                .gen::<f32>()
                .mul_add(max_radius - min_radius, min_radius);
            let [x, y]: [f32; 2] = rand_distr::UnitDisc.sample(&mut rng);
            let x = x * placement_radius;
            let y = y * placement_radius;
            let position = Vec3::new(x, radius, y);

            let geometry = Geometry::Sphere {
                center: position,
                radius,
            };

            if is_valid(&geometry) {
                let material = Material {
                    colour: (rng.gen(), rng.gen(), rng.gen()),
                    emission: [
                        (1., 1., 1.),
                        (1., 0., 0.),
                        (1., 1., 0.),
                        (0., 1., 0.),
                        (0., 1., 1.),
                        (0., 0., 1.),
                        (1., 0., 1.),
                    ][rng.gen_range(0..7)],
                    emission_strength: if rng.gen::<f32>() > 0.85 {
                        rng.gen_range(5.0..=15.)
                    } else {
                        0.
                    },
                    metallic: rng.gen(),
                    roughness: if rng.gen::<f32>() < 0.2 {
                        0.
                    } else {
                        rng.gen()
                    },
                };

                return Some(Object::new(name, material, geometry));
            }
        }

        None
    }

    /// Randomly fills a scene with spheres using the given parameters.
    #[must_use]
    pub fn random_spheres(
        min_radius: f32,
        max_radius: f32,
        placement_radius: f32,
        sphere_count: u32,
        seed: Option<u64>,
    ) -> Self {
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut objects: Vec<Object> = vec![];

        for i in 0u32..sphere_count {
            if let Some(object) = Self::random_sphere(
                &mut rng,
                format!("Sphere {i}"),
                min_radius,
                max_radius,
                placement_radius,
                |geometry| {
                    if let &Geometry::Sphere { center, radius } = geometry {
                        !objects.iter().any(|object| {
                            let Geometry::Sphere {
                                radius: other_radius,
                                ..
                            } = object.geometry
                            else {
                                return false;
                            };
                            let min_dst = radius + other_radius;
                            (*object.geometry.position() - center).magnitude() < min_dst
                        })
                    } else {
                        false
                    }
                },
            ) {
                objects.push(object);
            }
        }

        objects.push(Object::new(
            "Plane",
            Material {
                colour: (0.5, 0.5, 0.5),
                emission: (0., 0., 0.),
                emission_strength: 0.,
                metallic: 0.2,
                roughness: 0.5,
            },
            Geometry::Plane {
                center: Vec3::new(0., 0., 0.),
                normal: Vec3::new(0., 1., 0.),
                size: 100_000.,
            },
        ));

        Self {
            camera: Camera {
                position: Vec3::new(55., 65., 55.),
                rotation: Vec3::new(0.8, -0.8, 0.),
                fov: 70.,
            },
            objects,
            lights: vec![
                Light::Direction {
                    intensity: (0.4, 0.4, 0.4),
                    direction: Vec3::new(-1., -1.5, -0.5).normalize(),
                },
                Light::Point {
                    intensity: (0.4, 0.4, 0.4),
                    position: Vec3::new(0., 2., 0.),
                },
            ],
            background_colour: (0.5, 0.8, 1.),
            ambient_light: (0.2, 0.2, 0.2),
            reflection_limit: 3,
            do_objects_spin: false,
        }
    }

    // Can't implement `AsBytes` because this maps to 3 separate buffers
    #[must_use]
    pub fn as_bytes(
        &self,
        width: u32,
        height: u32,
    ) -> (
        [u8; Self::BUFFER_SIZE.0],
        [u8; Self::BUFFER_SIZE.1],
        [u8; Self::BUFFER_SIZE.2],
    ) {
        puffin::profile_function!();

        let vectors = self.camera.get_vectors_fru();

        (
            bytes_concat_fixed_in_n_iter(self.objects.iter().map(Object::as_bytes)),
            bytes_concat_fixed_in_n_iter(self.lights.iter().map(Light::as_bytes)),
            bytes_concat_n(&[
                &self.camera.position.as_bytes() as &[_; 16],
                &vectors.0.as_bytes() as &[_; 16],
                &vectors.1.as_bytes() as &[_; 16],
                &vectors.2.as_bytes() as &[_; 16],
                &tuple_bytes::<16>(self.background_colour),
                &tuple_bytes::<12>(self.ambient_light),
                &self.camera.fov.to_le_bytes(),
                &self.reflection_limit.to_le_bytes(),
                &width.to_le_bytes(),
                &height.to_le_bytes(),
            ]),
        )
    }
}

impl PartialEq for Scene {
    fn eq(&self, other: &Self) -> bool {
        puffin::profile_function!();

        (self.camera == other.camera)
            && {
                if self.objects.len() == other.objects.len() {
                    for (i, object) in self.objects.iter().enumerate() {
                        if *object != other.objects[i] {
                            return false;
                        }
                    }

                    true
                } else {
                    false
                }
            }
            && {
                if self.lights.len() == other.lights.len() {
                    for (i, light) in self.lights.iter().enumerate() {
                        if *light != other.lights[i] {
                            return false;
                        }
                    }

                    true
                } else {
                    false
                }
            }
            && (self.background_colour == other.background_colour)
            && (self.ambient_light == other.ambient_light)
            && (self.reflection_limit == other.reflection_limit)
            && (self.do_objects_spin == other.do_objects_spin)
    }
}

use super::Vec3;

use crate::utils::bytes::{bytes_concat_n, AsBytes};

/// These parameters influence how light interacts with the object.
#[derive(Clone, PartialEq)]
pub struct Material {
    /// The albedo colour.
    /// RGB from 0..1.
    pub colour: Vec3,
    /// The emissive colour.
    /// RGB from 0..1.
    pub emission: Vec3,
    /// The emission strength.
    pub emission_strength: f32,
    /// How much of the object's colour is a reflection of the environment.
    ///
    /// In the range 0..1.
    pub metallic: f32,
    /// How rough the reflection is
    pub roughness: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            colour: Vec3::new(1., 0., 0.),
            emission: Vec3::new(0., 0., 0.),
            emission_strength: 0.,
            metallic: 0.5,
            roughness: 0.5,
        }
    }
}

impl Material {
    pub const BUFFER_SIZE: usize = 48;
}

impl AsBytes<{ Self::BUFFER_SIZE }> for Material {
    fn as_bytes(&self) -> [u8; Self::BUFFER_SIZE] {
        puffin::profile_function!();

        bytes_concat_n(
            [
                &self.colour.as_bytes(),
                [0u8; 4].as_slice(),
                &self.emission.as_bytes(),
                &self.emission_strength.to_le_bytes(),
                &self.metallic.to_le_bytes(),
                &self.roughness.to_le_bytes(),
            ]
            .into_iter(),
        )
    }
}

/// Stores the geometry of an object.
///
/// Each type has it's own parameters.
///
/// Different types are:
/// - Sphere
/// - Plane
#[derive(Clone, PartialEq)]
pub enum Geometry {
    /// A sphere.
    Sphere {
        /// The center of the sphere.
        center: Vec3,
        /// The radius of the sphere.
        radius: f32,
    },
    /// A plane
    Plane {
        /// The center of the plane.
        center: Vec3,
        /// The normal the plane faces towards.
        normal: Vec3,
        /// The length of each side of the plane.
        size: f32,
    },
}

impl Geometry {
    pub const BUFFER_SIZE: usize = 64;

    #[must_use]
    pub const fn default_sphere() -> Self {
        Self::Sphere {
            center: Vec3::new(0., 0., 0.),
            radius: 1.,
        }
    }

    #[must_use]
    pub const fn default_plane() -> Self {
        Self::Plane {
            center: Vec3::new(0., 0., 0.),
            normal: Vec3::new(0., 1., 0.),
            size: 1.,
        }
    }

    /// Gets the position of the object to show in the editor.
    #[must_use]
    pub const fn position(&self) -> &Vec3 {
        match self {
            Self::Plane { center, .. } | Self::Sphere { center, .. } => center,
        }
    }

    /// Gets the position of the object to show in the editor.
    pub fn position_as_mut(&mut self) -> &mut Vec3 {
        match self {
            Self::Plane { center, .. } | Self::Sphere { center, .. } => center,
        }
    }
}

impl AsBytes<{ Self::BUFFER_SIZE }> for Geometry {
    fn as_bytes(&self) -> [u8; Self::BUFFER_SIZE] {
        puffin::profile_function!();

        match self {
            Self::Sphere { center, radius } => bytes_concat_n(
                [
                    &0u32.to_le_bytes(),
                    [0u8; 12].as_slice(),
                    &center.as_bytes(),
                    &[0u8; 16],
                    &radius.to_le_bytes(),
                ]
                .into_iter(),
            ),
            Self::Plane {
                center,
                normal,
                size,
            } => bytes_concat_n(
                [
                    &1u32.to_le_bytes(),
                    [0u8; 12].as_slice(),
                    &center.as_bytes(),
                    &[0u8; 4],
                    &normal.as_bytes(),
                    &size.to_le_bytes(),
                ]
                .into_iter(),
            ),
        }
    }
}

/// Stores all the information about an object.
#[derive(Clone, PartialEq)]
pub struct Object {
    /// The id of the object.
    /// Has to be unique.
    pub id: u128,
    /// The name of the object.
    ///
    /// This doesn't have to be unique, it's just for the editor.
    pub name: String,
    /// The material of the object.
    pub material: Material,
    /// The geometry of the object.
    pub geometry: Geometry,
}

impl Object {
    pub const BUFFER_SIZE: usize = Material::BUFFER_SIZE + Geometry::BUFFER_SIZE;

    #[must_use]
    pub fn default_sphere() -> Self {
        Self::new("sphere", Material::default(), Geometry::default_sphere())
    }

    #[must_use]
    pub fn default_plane() -> Self {
        Self::new("plane", Material::default(), Geometry::default_plane())
    }

    /// Creates a new object.
    pub fn new(name: impl Into<String>, material: Material, geometry: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4().as_u128(),
            name: name.into(),
            material,
            geometry,
        }
    }
}

impl AsBytes<{ Self::BUFFER_SIZE }> for Object {
    fn as_bytes(&self) -> [u8; Self::BUFFER_SIZE] {
        puffin::profile_function!();

        bytes_concat_n(
            [
                &self.material.as_bytes(),
                self.geometry.as_bytes().as_slice(),
            ]
            .into_iter(),
        )
    }
}

/// Stores the information about a light.
///
/// The different types are:
/// - Direction
/// - Point
#[derive(Clone, PartialEq)]
pub enum Light {
    Direction { intensity: Vec3, direction: Vec3 },
    Point { intensity: Vec3, position: Vec3 },
}

impl Light {
    pub const BUFFER_SIZE: usize = 48;
}

impl AsBytes<{ Self::BUFFER_SIZE }> for Light {
    fn as_bytes(&self) -> [u8; Self::BUFFER_SIZE] {
        puffin::profile_function!();

        match self {
            Self::Direction {
                intensity,
                direction,
            } => bytes_concat_n(
                [
                    &0u32.to_le_bytes(),
                    [0u8; 12].as_slice(),
                    &intensity.as_bytes(),
                    &[0u8; 4],
                    &direction.as_bytes(),
                ]
                .into_iter(),
            ),
            Self::Point {
                intensity,
                position,
            } => bytes_concat_n(
                [
                    &1u32.to_le_bytes(),
                    [0u8; 12].as_slice(),
                    &intensity.as_bytes(),
                    &[0u8; 4],
                    &position.as_bytes(),
                ]
                .into_iter(),
            ),
        }
    }
}

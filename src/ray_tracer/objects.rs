use super::Vec3;

use crate::utils::bytes::*;

/// These parameters influence how light interacts with the object.
#[derive(Clone, PartialEq)]
pub struct Material {
    /// The albedo colour.
    /// RGB from 0..1.
    pub colour: (f32, f32, f32),
    /// The emissive colour.
    /// RGB from 0..1.
    pub emission: (f32, f32, f32),
    /// The emission strength.
    pub emission_strength: f32,
    /// How much of the object's colour is a reflection of the environment.
    ///
    /// In the range 0..1.
    pub metallic: f32,
    /// How rough the reflection is
    pub roughness: f32,
}

impl Material {
    pub const BUFFER_SIZE: usize = 48;

    /// Get the byte representation of the object.
    pub fn as_bytes(&self) -> [u8; Material::BUFFER_SIZE] {
        bytes_concat_n(&[
            &tuple_bytes::<16>(self.colour),
            &tuple_bytes::<12>(self.emission),
            &self.emission_strength.to_le_bytes(),
            &self.metallic.to_le_bytes(),
            &self.roughness.to_le_bytes(),
        ])
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

    /// Get the byte representation of the object.
    pub fn as_bytes(&self) -> [u8; Geometry::BUFFER_SIZE] {
        match self {
            Geometry::Sphere { center, radius } => bytes_concat_n(&[
                &0u32.to_le_bytes(),
                &[0u8; 12],
                &center.as_bytes::<16>(),
                &[0u8; 12],
                &radius.to_le_bytes(),
            ]),
            Geometry::Plane {
                center,
                normal,
                size,
            } => bytes_concat_n(&[
                &1u32.to_le_bytes(),
                &[0u8; 12],
                &center.as_bytes::<16>(),
                &normal.as_bytes::<12>(),
                &size.to_le_bytes(),
            ]),
        }
    }

    /// Gets the position of the object to show in the editor.
    pub fn position(&self) -> &Vec3 {
        match self {
            Geometry::Sphere { center, radius: _ } => center,
            Geometry::Plane {
                center,
                normal: _,
                size: _,
            } => center,
        }
    }

    /// Gets the position of the object to show in the editor.
    pub fn position_as_mut(&mut self) -> &mut Vec3 {
        match self {
            Geometry::Sphere { center, radius: _ } => center,
            Geometry::Plane {
                center,
                normal: _,
                size: _,
            } => center,
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

    /// Get the byte representation of the object.
    pub fn as_bytes(&self) -> [u8; Object::BUFFER_SIZE] {
        bytes_concat_n(&[&self.material.as_bytes(), &self.geometry.as_bytes()])
    }

    /// Creates a new object.
    pub fn new(name: impl ToString, material: Material, geometry: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4().as_u128(),
            name: name.to_string(),
            material,
            geometry,
        }
    }
}

/// Stores the information about a light.
///
/// The different types are:
/// - Direction
/// - Point
#[derive(Clone, PartialEq)]
pub enum Light {
    Direction {
        intensity: (f32, f32, f32),
        direction: Vec3,
    },
    Point {
        intensity: (f32, f32, f32),
        position: Vec3,
    },
}

impl Light {
    pub const BUFFER_SIZE: usize = 48;

    /// Get the byte representation of the object.
    pub fn as_bytes(&self) -> [u8; Light::BUFFER_SIZE] {
        match self {
            Light::Direction {
                intensity,
                direction,
            } => bytes_concat_n(&[
                &0u32.to_le_bytes(),
                &[0u8; 12],
                &tuple_bytes::<16>(*intensity),
                &direction.as_bytes::<12>(),
            ]),
            Light::Point {
                intensity,
                position,
            } => bytes_concat_n(&[
                &1u32.to_le_bytes(),
                &[0u8; 12],
                &tuple_bytes::<16>(*intensity),
                &position.as_bytes::<12>(),
            ]),
        }
    }
}

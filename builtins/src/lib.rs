use macros;

pub use macros::generate_builtin_fn;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

unsafe impl bytemuck::Pod for Vec3 {}
unsafe impl bytemuck::Zeroable for Vec3 {}

pub mod camera;
pub mod mesh;
pub mod opengl;
pub mod transform;

use camera::Camera;
use cgmath::prelude::*;
use cgmath::{Deg, PerspectiveFov, Rad, Vector2, Vector3, Vector4};
use transform::Transform;

use std::path::PathBuf;

use std::time::*;

fn edge(p: Vector2<f32>, v0: Vector2<f32>, v1: Vector2<f32>) -> f32 {
    (p.x - v0.x) * (v1.y - v0.y) - (p.y - v0.y) * (v1.x - v0.x)
}

type Vector2f = Vector2<f32>;
type Vector3f = Vector3<f32>;

#[derive(Copy, Clone, Debug)]
struct Tri2(Vector2f, Vector2f, Vector2f);
#[derive(Copy, Clone, Debug)]
struct Tri3(Vector3f, Vector3f, Vector3f);

impl Tri3 {
    pub fn truncate(self) -> Tri2 {
        Tri2(self.0.truncate(), self.1.truncate(), self.2.truncate())
    }
}

#[derive(Copy, Clone)]
struct Bounds2 {
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
}

fn calculate_triangle_bounds(tri: Tri2) -> Bounds2 {
    let points = [tri.0, tri.1, tri.2];

    let mut min_x = 42000.0;
    let mut max_x = 0.0;
    let mut min_y = 42000.0;
    let mut max_y = 0.0;

    for p in points.iter() {
        if p.x < min_x {
            min_x = p.x;
        }
        if p.x > max_x {
            max_x = p.x;
        }
        if p.y < min_y {
            min_y = p.y;
        }
        if p.y > max_y {
            max_y = p.y;
        }
    }

    Bounds2 {
        min_x,
        min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

///
///
///
fn rasterize_window_space<F>(tri: Tri3, mut cb: F)
where
    F: FnMut((u32, u32), (f32, f32, f32)) -> (),
{
    let bounds = calculate_triangle_bounds(tri.truncate());
    let rast_min_x = bounds.min_x as u32;
    let rast_max_x = (bounds.min_x + bounds.width + 1.0) as u32;
    let rast_min_y = bounds.min_y as u32;
    let rast_max_y = (bounds.min_y + bounds.height + 1.0) as u32;

    for x in rast_min_x..rast_max_x {
        for y in rast_min_y..rast_max_y {
            let p = Vector2::new(x as f32, y as f32);

            let v0 = tri.0.truncate();
            let v1 = tri.1.truncate();
            let v2 = tri.2.truncate();

            let area = edge(v0, v1, v2);

            let w0 = edge(p, v1, v2);
            let w1 = edge(p, v2, v0);
            let w2 = edge(p, v0, v1);

            if (w0 >= 0.0) && (w1 >= 0.0) && (w2 >= 0.0) {
                cb((x, y), (w0 / area, w1 / area, w2 / area))
            }
        }
    }
}

fn main() {
    // 1. The **winit::EventsLoop** for handling events.
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    // 2. Parameters for building the Window.
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Hello world");
    // 3. Parameters for building the OpenGL context.
    let cb = glium::glutin::ContextBuilder::new();
    // 4. Build the Display with the given window and OpenGL context parameters and register the
    //    window with the events_loop.
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let im_dims = (800, 600);

    let mut imgbuf = image::ImageBuffer::new(im_dims.0, im_dims.1);

    let mut camera = Camera::new(
        Transform::default(),
        PerspectiveFov {
            fovy: Rad::from(Deg(75.0)),
            aspect: 1280.0 / 720.0,
            near: 0.1,
            far: 1000.0,
        },
    );
    camera.transform.position.z = -3.0;

    let view = camera.get_view_matrix();
    let proj = camera.get_projection_matrix();

    let mesh = mesh::load_ply(PathBuf::from("monkey.ply"));

    let mut depth = vec![1.0; im_dims.0 as usize * im_dims.1 as usize];

    // clear
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);
    }

    let begin = std::time::Instant::now();
    for tri in mesh.triangles {
        let mut t1_ndc = [
            (proj * view * tri.0.position.extend(1.0)),
            (proj * view * tri.1.position.extend(1.0)),
            (proj * view * tri.2.position.extend(1.0)),
        ];

        for p in t1_ndc.iter_mut() {
            p.x = p.x / p.w;
            p.y = p.y / p.w;
            p.z = p.z / p.w;
        }

        let t1_ndc: Vec<_> = t1_ndc.iter().map(|v| Vector4::truncate(*v)).collect();

        struct Viewport {
            x: i32,
            y: i32,
            width: u32,
            height: u32,
        }

        let view = Viewport {
            x: 0,
            y: 0,
            width: im_dims.0,
            height: im_dims.1,
        };

        let near_val = 0.0;
        let far_val = 1.0;

        let ndc_to_wnd = |p: Vector3<f32>| {
            let (x_ndc, y_ndc, z_ndc) = p.into();
            Vector3::new(
                (view.width / 2) as f32 * x_ndc + view.x as f32 + (view.width / 2) as f32,
                (view.height / 2) as f32 * y_ndc + view.y as f32 + (view.height / 2) as f32,
                ((far_val - near_val) / 2.0) * z_ndc + ((far_val + near_val) / 2.0),
            )
        };

        let t1_wnd = [
            ndc_to_wnd(t1_ndc[0]),
            ndc_to_wnd(t1_ndc[1]),
            ndc_to_wnd(t1_ndc[2]),
        ];

        let t1_wnd = Tri3(t1_wnd[0], t1_wnd[1], t1_wnd[2]);

        rasterize_window_space(t1_wnd, |(x, y), (w0, w1, w2)| {
            let i = im_dims.0 * y + x;

            let d = t1_wnd.0.z * w0 + t1_wnd.1.z * w1 + t1_wnd.2.z * w2;

            let n = tri.0.normal * w0 + tri.1.normal * w1 + tri.2.normal * w2;

            if d < depth[i as usize] {
                let cosa = n.dot(Vector3::new(-0.5, 1.0, 1.0)).max(0.0);
                let color = Vector3::new(1.0, 0.5, 0.5);
                let out = color * cosa;

                *(imgbuf.get_pixel_mut(x, y)) = image::Rgb([
                    (out.x * 255.0) as u8,
                    (out.y * 255.0) as u8,
                    (out.z * 255.0) as u8,
                ]);
                depth[i as usize] = d;
            }
        })
    }

    println!("{:?}", Instant::now().duration_since(begin));

    imgbuf.save("output.png").unwrap();
}

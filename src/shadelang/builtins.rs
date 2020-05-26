use crate::shadelang::ast::TypeKind;
use crate::shadelang::vm::VirtualMachine;

use builtins::generate_builtin_fn;
use builtins::Vec3;

pub trait BuiltInType {
    fn stack_size() -> usize;
    fn type_kind() -> TypeKind;
}
impl BuiltInType for f32 {
    fn stack_size() -> usize {
        4
    }
    fn type_kind() -> TypeKind {
        TypeKind::F32
    }
}
impl BuiltInType for Vec3 {
    fn stack_size() -> usize {
        12
    }
    fn type_kind() -> TypeKind {
        TypeKind::Vec3
    }
}

pub trait BuiltInCallable {
    fn ident(&self) -> &str;
    fn vm_impl(&self, vm: &mut VirtualMachine);
    fn return_type(&self) -> TypeKind;
    fn arg_types(&self) -> Vec<TypeKind>;
}

#[generate_builtin_fn("Vec3")]
fn Vec3Constructor(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

#[generate_builtin_fn("normalize")]
fn Vec3Normalize(a: Vec3) -> Vec3 {
    let len = (a.x * a.x + a.y * a.y + a.z * a.z).sqrt();
    Vec3 {
        x: a.x / len,
        y: a.y / len,
        z: a.z / len,
    }
}

#[generate_builtin_fn("dot")]
fn Vec3Dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[generate_builtin_fn("foo")]
fn FloatFoo(a: f32) -> f32 {
    a * 2.0
}

#[generate_builtin_fn("foo")]
fn Vec3Foo(a: Vec3) -> Vec3 {
    a
}

const FUNCTIONS: &[&dyn BuiltInCallable] = &[
    &Vec3Constructor,
    &Vec3Normalize,
    &Vec3Dot,
    &FloatFoo,
    &Vec3Foo,
];

pub fn get_builtin_fn<'a>(
    id: &str,
    arg_types: &'a [TypeKind],
) -> Option<(usize, &'static dyn BuiltInCallable)> {
    for (i, f) in FUNCTIONS.iter().enumerate() {
        if f.ident() == id && f.arg_types().as_slice() == arg_types {
            return Some((i, *f));
        }
    }

    None
}

pub fn call_builtin_fn(func_id: usize, vm: &mut VirtualMachine) {
    FUNCTIONS[func_id].vm_impl(vm);
}

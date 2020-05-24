use crate::shadelang::ast::TypeKind;
use crate::shadelang::vm::VirtualMachine;

pub trait BuiltInCallable {
    fn ident(&self) -> &str;
    fn vm_impl(&self, vm: &mut VirtualMachine);
    fn return_type(&self) -> TypeKind;
}

pub struct Vec3Constructor;
impl BuiltInCallable for Vec3Constructor {
    fn ident(&self) -> &str {
        "Vec3"
    }
    fn vm_impl(&self, vm: &mut VirtualMachine) {
        // this is kinda magical, but a vec3 constructor is actually a noop
        // atleast assuming no padding etc. etc., but we know this is true for our vm
    }
    fn return_type(&self) -> TypeKind {
        TypeKind::Vec3
    }
}

pub struct Vec3Normalize;
impl BuiltInCallable for Vec3Normalize {
    fn ident(&self) -> &str {
        "normalize"
    }
    fn vm_impl(&self, vm: &mut VirtualMachine) {
        let z: f32 = unsafe { vm.pop_stack() };
        let y: f32 = unsafe { vm.pop_stack() };
        let x: f32 = unsafe { vm.pop_stack() };

        let len = (x * x + y * y + z * z).sqrt();

        vm.push_stack(x / len);
        vm.push_stack(y / len);
        vm.push_stack(z / len);
    }
    fn return_type(&self) -> TypeKind {
        TypeKind::Vec3
    }
}

pub struct Vec3Dot;
impl BuiltInCallable for Vec3Dot {
    fn ident(&self) -> &str {
        "dot"
    }
    fn vm_impl(&self, vm: &mut VirtualMachine) {
        let z1: f32 = unsafe { vm.pop_stack() };
        let y1: f32 = unsafe { vm.pop_stack() };
        let x1: f32 = unsafe { vm.pop_stack() };

        let z2: f32 = unsafe { vm.pop_stack() };
        let y2: f32 = unsafe { vm.pop_stack() };
        let x2: f32 = unsafe { vm.pop_stack() };

        let dot = z1 * z2 + y1 * y2 + x1 * x2;

        vm.push_stack(dot);
    }
    fn return_type(&self) -> TypeKind {
        TypeKind::F32
    }
}

#[test]
fn test_vec3_dot() {
    let p = crate::shadelang::vm::VMProgram::new();
    let mut vm = VirtualMachine::new(&p);
    vm.push_stack(1.0f32);
    vm.push_stack(0.0f32);
    vm.push_stack(0.0f32);

    vm.push_stack(1.0f32);
    vm.push_stack(0.0f32);
    vm.push_stack(0.0f32);

    (Vec3Dot {}).vm_impl(&mut vm);

    assert_eq!(unsafe { vm.pop_stack::<f32>() }, 1.0f32);
}

const FUNCTIONS: &[&dyn BuiltInCallable] = &[&Vec3Constructor, &Vec3Normalize, &Vec3Dot];

pub fn get_builtin_fn(id: &str) -> Option<(usize, &dyn BuiltInCallable)> {
    for (i, f) in FUNCTIONS.iter().enumerate() {
        if f.ident() == id {
            return Some((i, *f));
        }
    }

    None
}

pub fn call_builtin_fn(func_id: usize, vm: &mut VirtualMachine) {
    FUNCTIONS[func_id].vm_impl(vm);
}

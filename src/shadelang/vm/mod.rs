use crate::shadelang::compiler::*;
use std::mem;

pub struct VirtualMachine<'a> {
    program: &'a VMProgram,
    pub stack: Vec<u8>,
    isp: usize,
}

impl<'a> VirtualMachine<'a> {
    pub fn new(program: &'a VMProgram) -> VirtualMachine<'a> {
        VirtualMachine {
            program,
            stack: Vec::with_capacity(program.data.min_stack_size),
            isp: 0,
        }
    }

    pub fn set_in_float(&mut self, ident: &str, val: f32) {
        let offset = self.program.data.global_symbols.get(ident).unwrap().offset;

        unsafe {
            self.write_stack(offset, val);
        }
    }

    pub fn get_out_float(&mut self, ident: &str) -> f32 {
        let offset = self.program.data.global_symbols.get(ident).unwrap().offset;

        unsafe {
            self.load_stack(offset)
        }
    }

    pub fn run_fn(&mut self, id: &str) {
        let fnc = self.program.data.functions.get(id).unwrap();
        self.isp = fnc.address;
        let mut depth = 0;

        for _ in 0..(self.program.data.static_section_size / 4) {
            self.push_stack_raw(0);
        }

        let mut stack_base = self.stack.len();

        loop {
            let (op, p) = self.program.code[self.isp].get_inst();
            self.isp = self.isp + 1;

            match op {
                OpCode::Call => {
                    depth += 1;
                    self.push_stack_raw(self.isp as u32);
                    self.push_stack_raw(stack_base as u32);
                    stack_base = self.stack.len();
                    self.isp = p as usize;
                }
                OpCode::ConstF32 => {
                    self.push_stack_raw(self.program.code[self.isp].data);
                    self.isp = self.isp + 1;
                }
                OpCode::MulF32 => unsafe {
                    let f1 = self.pop_stack::<f32>();
                    let f2 = self.pop_stack::<f32>();

                    self.push_stack_raw(std::mem::transmute(f1 * f2));
                },
                OpCode::AddF32 => unsafe {
                    let f1 = self.pop_stack::<f32>();
                    let f2 = self.pop_stack::<f32>();

                    self.push_stack_raw(std::mem::transmute(f1 + f2));
                },
                OpCode::Mov4 => unsafe {
                    let val = self.pop_stack::<u32>();
                    self.write_stack(stack_base + p as usize, val);
                },
                OpCode::Mov4Global => unsafe {
                    let val = self.pop_stack::<u32>();
                    self.write_stack(p as usize, val);
                },
                OpCode::Load4 => unsafe {
                    let val = self.load_stack::<u32>(stack_base + p as usize);
                    self.push_stack_raw(std::mem::transmute(val));
                },
                OpCode::Load4Global => unsafe {
                    let val = self.load_stack::<u32>(p as usize);
                    self.push_stack_raw(std::mem::transmute(val));
                }
                OpCode::Void => self.push_stack_raw(0),
                OpCode::Ret => {
                    if depth == 0 {
                        // we are done
                        return;
                    }
                    depth -= 1;
                    let reta = unsafe {
                        let rv = self.pop_stack::<u32>();
                        for _ in 0..p {
                            self.pop_stack::<u8>();
                        }
                        let sb = self.pop_stack::<u32>();
                        stack_base = sb as usize;

                        let ra = self.pop_stack::<u32>();
                        self.push_stack_raw(rv);
                        ra
                    };
                    self.isp = reta as usize;
                }
                o => unimplemented!("{:?}", o),
            }
        }
    }

    pub unsafe fn write_stack<T: Copy + std::fmt::Debug>(&mut self, offset: usize, val: T) {
        let ptr = self.stack.as_mut_ptr().offset(offset as isize) as *mut T;

        *ptr = val;
    }

    pub unsafe fn load_stack<T: Copy + std::fmt::Debug>(&mut self, offset: usize) -> T {
        let ptr = self.stack.as_ptr().offset(offset as isize) as *const T;
        *ptr
    }

    pub fn push_stack_raw(&mut self, data: u32) {
        self.stack.extend_from_slice(bytemuck::bytes_of(&data));
    }

    pub unsafe fn pop_stack<T>(&mut self) -> T
    where
        T: Copy + std::fmt::Debug,
    {
        let ptr = self
            .stack
            .as_ptr()
            .offset((self.stack.len() - mem::size_of::<T>()) as isize);
        let v = *(ptr as *const T);

        let _ = self.stack.split_off(self.stack.len() - mem::size_of::<T>());
        v
    }
}

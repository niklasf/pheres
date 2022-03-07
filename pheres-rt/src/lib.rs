use std::mem::forget;
use std::{os::raw::c_char, ptr::NonNull};

#[repr(C)]
pub struct Context {
    _unused: (),
}

#[no_mangle]
pub extern "C" fn pheres_context_new() -> Context {
    println!("hello world");
    Context { _unused: () }
}

#[repr(C)]
pub enum RawValue {
    Integer(i64),
    Float(f64),
    String {
        ptr: *const u8,
        len: usize,
    },
    Term {
        functor_ptr: *const u8,
        functor_len: usize,
        args_ptr: *mut RawValue,
        args_len: usize,
        args_capacity: usize,
        annotations_ptr: *mut RawValue,
        annotations_len: usize,
        annotations_capacity: usize,
    },
}

#[no_mangle]
pub extern "C" fn pheres_value_new_integer(n: i64) -> RawValue {
    RawValue::Integer(n)
}

#[no_mangle]
pub extern "C" fn pheres_value_new_float(f: f64) -> RawValue {
    RawValue::Float(f)
}

#[no_mangle]
pub extern "C" fn pheres_value_new_string(ptr: *const u8, len: usize) -> RawValue {
    RawValue::String { ptr, len }
}

#[no_mangle]
pub extern "C" fn pheres_value_new_atom(ptr: *const u8, len: usize) -> RawValue {
    let mut args = Vec::new();
    let mut annotations = Vec::new();
    let atom = RawValue::Term {
        functor_ptr: ptr,
        functor_len: len,
        args_ptr: args.as_mut_ptr(),
        args_len: args.len(),
        args_capacity: args.capacity(),
        annotations_ptr: annotations.as_mut_ptr(),
        annotations_len: annotations.len(),
        annotations_capacity: annotations.capacity(),
    };
    forget(args);
    forget(annotations);
    atom
}

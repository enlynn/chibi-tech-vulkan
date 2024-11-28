// temporary
#![allow(unused)]

#![feature(concat_idents)]
#![feature(stmt_expr_attributes)]

extern crate vendor;

mod window;
mod renderer;
mod math;

pub mod core;

mod util;

use std::rc::Rc;

/*

A few thoughts on Pointers:

Box<T>: allows mutable and immutable borrows at compile time. Single Owner.
Rc<T>: allows for multiple owners of immutable data. Data *connot* be immutable. Multiple Immutable Owners.
Arc<T>: Same as Rc<T> but can be shared across threads.
RefCell<T>: allows for *interior mutability*. allows for mutable borrows at runtime. If value is invalid, then program will panic.
Cell<T>: allows for *interior mutability*. allows for mutable borrows at runtime by copying the value. programs will not panic.

RefCell and Cell do not allocate extra memory within a type. They instead are broken into:
    struct RefCell<T> {
        borrow_count: Cell<isize>,
        contents: T,
    }

When using Rc<T>, the pattern seems to be:

struct SomeType {
    mutable_typeA: RefCell<ComplexType>,
    mutable_typeB: Cell<ComplexCopyableType>,
    const_typeC:    ConstType,
}

value: Rc<SomeType> = Rc::new(SomeType{/* data here */});


Thoughts and expectations on performance:
- Rc<T> can be a container type for a *system* that might be shared across multiple systems. However, it should not
  wrap operations that are expected to be done many times a frame. What is the cost of deref'ing an Rc?
- RefCell<T> and Cell<T> should be used for interior mutability of containers when possible. This is to avoid memory costs
  and possible runtime checking of accessing the type.

*/

pub const ENGINE_VERSION_MAJOR: u32 = 0;
pub const ENGINE_VERSION_MINOR: u32 = 0;
pub const ENGINE_VERSION_PATCH: u32 = 1;
pub const ENGINE_VERSION: u32 = ENGINE_VERSION_MAJOR << 24 | ENGINE_VERSION_MINOR << 16 | ENGINE_VERSION_PATCH << 8;

pub fn new_engine() -> Rc<core::engine::Engine>{
    return Rc::new(core::engine::Engine::new());
}

pub fn make_app_version(major: u32, minor: u32, patch: u32) -> u32 {
    return major << 24 | minor << 16 | patch << 8;
}

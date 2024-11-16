// temporary
#![allow(unused)]

#![feature(concat_idents)]

mod window;
mod renderer;

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

/*

A few thoughts on Lifetimes:

*/

pub fn hello_engine() {
    println!("Hello, engine!");
}

pub fn new_engine() -> Rc<core::engine::Engine>{
    return Rc::new(core::engine::Engine::new());
}

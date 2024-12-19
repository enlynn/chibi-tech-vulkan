#![allow(unused)]

#![feature(iter_advance_by)]

#[cfg(feature = "importer")]
extern crate image;
#[cfg(feature = "importer")]
extern crate meshopt;

#[cfg(feature = "importer")]
pub mod importer;

pub mod mesh;
pub mod texture;
pub mod material;

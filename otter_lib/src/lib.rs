#![allow(unused_must_use)]
#![allow(clippy::single_match)]
#![allow(clippy::collapsible_else_if)]
// #![allow(unused_imports)]

pub mod builder;
pub mod config;
pub mod context;
pub mod structures;
pub mod types;

mod procedures;

mod generic;

pub mod dispatch;

pub mod db;
mod misc;
mod transient;

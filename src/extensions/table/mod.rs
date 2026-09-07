//! Table extension support
//!
//! This module provides extended table functionality:
//! - UMD-style tables with cell spanning and decorations (`umd`)
//! - GFM tables are handled by comrak; `gfm` only adds the default class
//!   and restores UMD tables extracted during preprocessing

pub mod gfm;
pub mod umd;

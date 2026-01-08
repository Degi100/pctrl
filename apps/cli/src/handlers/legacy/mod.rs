//! Legacy command handlers (SSH, Docker, Coolify, Git)
//!
//! These handlers manage connections and services that existed
//! before the v6 project-centric architecture.

pub mod coolify;
pub mod docker;
pub mod git;
pub mod ssh;

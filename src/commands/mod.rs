//! Commands module for the Nox agent ecosystem
//! 
//! This module contains the implementations of the CLI commands.

pub mod init;
pub mod start;
pub mod stop;
pub mod status;
pub mod health;
pub mod serve;

pub mod agent {
    pub mod add;
    pub mod list;
    pub mod show;
    pub mod update;
    pub mod delete;
    pub mod start;
    pub mod stop;
}

pub mod task {
    pub mod create;
    pub mod list;
    pub mod update;
    pub mod overview;
    pub mod cancel;
    pub mod execute;
    pub mod show;
}


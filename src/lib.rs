// ============================================================================
// lib.rs — Nüwa Backup 库入口
//
// 将二进制 crate 转换为"二进制 + 库"的混合模式
// 原因：Rust 集成测试只能访问库 crate 的公共 API
// 通过 lib.rs 和 main.rs 的分离，实现"一个 crate，两种入口"
// ============================================================================

pub mod backup;
pub mod checksum;
pub mod cli;
pub mod diskspace;
pub mod errors;
pub mod list;
pub mod manifest;
pub mod restore;
pub mod storage;
pub mod verify;

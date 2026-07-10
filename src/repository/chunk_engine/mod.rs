// ============================================================================
// mod.rs — Chunk Engine module entry point
// ============================================================================
//
// Phase S Wave 2: ChunkEngine splits data streams into blocks.
// See Architecture v1.0 §7.

pub mod engine;
pub mod policy;

pub use engine::{ChunkEngine, ChunkResult};
pub use policy::{ChunkPolicy, FixedChunkPolicy};

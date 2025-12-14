// 聚合所有 DAW 命令的子模块
pub mod clip;
pub mod global;
pub mod track;

// 可选：对常用命令进行重新导出以便上层直接 `use daw::commands::*`
pub use clip::*;
pub use global::*;
pub use track::*;

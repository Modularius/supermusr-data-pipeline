pub mod finite_difference;
pub mod find_baseline;

pub mod load_from_trace_file;
pub mod save_to_file;
pub mod to_trace;

use crate::Real;
pub type RealArray<const N: usize> = [Real; N];



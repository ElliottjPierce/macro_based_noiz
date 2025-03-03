//! Tiny utilities for just 1 dimension.

use crate::name_array;

name_array! {
    /// Represents a two directions along an axis.
    pub struct AxisDirections,
    /// Represents a direction along an axis.
    pub enum AxisDirection: u8, u8 {
        /// The negative direction along an axis.
        Negative,
        /// The positive direction along an axis.
        Positive,
    }
}

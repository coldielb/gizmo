//! Frame Rendering Utilities
//!
//! This module provides utilities for rendering Gizmo frames to different output
//! formats. Currently focused on ASCII text output for debugging purposes, but
//! designed to be extensible for other rendering backends.
//!
//! ## Core Functionality
//!
//! ### ASCII Rendering
//! Converts boolean pixel matrices into human-readable ASCII representations:
//! - `true` pixels (on) → `#` characters
//! - `false` pixels (off) → `.` characters
//!
//! This is primarily used for:
//! - Debugging pattern generation logic
//! - Command-line preview of animations
//! - Development and testing of frame content
//!
//! ## Design Philosophy
//!
//! The frame renderer is kept simple and focused:
//! - **Single Responsibility**: Only handles frame-to-text conversion
//! - **No Dependencies**: Uses only standard library functionality
//! - **Extensible**: Structure allows adding new rendering formats
//!
//! ## Usage
//!
//! ```rust
//! let renderer = FrameRenderer::new(128, 128);
//! let ascii_output = renderer.render_ascii(&frame);
//! println!("{}", ascii_output);
//! ```

use crate::ast::Frame;

/// ASCII renderer for Gizmo animation frames.
///
/// Provides utilities to convert frame data into human-readable text
/// representations for debugging and development purposes.
pub struct FrameRenderer {
    /// Expected frame width (for validation/context)
    pub width: usize,
    /// Expected frame height (for validation/context)
    pub height: usize,
}

impl FrameRenderer {
    /// Creates a new frame renderer with expected dimensions.
    ///
    /// The dimensions are stored for context but don't restrict the actual
    /// frames that can be rendered - frames of any size can be processed.
    ///
    /// # Arguments
    /// * `width` - Expected frame width
    /// * `height` - Expected frame height
    ///
    /// # Returns
    /// A new FrameRenderer instance
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
    
    /// Renders a frame as ASCII text for visual debugging.
    ///
    /// Converts the boolean pixel matrix into a text representation where:
    /// - `true` pixels become `#` (on/filled)
    /// - `false` pixels become `.` (off/empty)
    /// - Each row ends with a newline character
    ///
    /// This creates a visual representation that can be printed to console
    /// or saved to text files for inspection.
    ///
    /// # Arguments
    /// * `frame` - The frame to render
    ///
    /// # Returns
    /// A multi-line string representing the frame visually
    ///
    /// # Example Output
    /// ```text
    /// ...###...
    /// ..#...#..
    /// .#.....#.
    /// #.......#
    /// .#.....#.
    /// ..#...#..
    /// ...###...
    /// ```
    pub fn render_ascii(&self, frame: &Frame) -> String {
        let mut output = String::new();
        
        for row in &frame.pixels {
            for &pixel in row {
                output.push(if pixel { '#' } else { '.' });
            }
            output.push('\n');
        }
        
        output
    }
}
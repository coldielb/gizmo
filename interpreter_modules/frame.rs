use crate::ast::{Frame, Value};
use crate::error::{GizmoError, Result};

pub struct FrameRenderer {
    pub width: usize,
    pub height: usize,
}

impl FrameRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
    
    pub fn render_frame(&self, frame: &Frame) -> String {
        let mut output = String::new();
        
        for row in &frame.pixels {
            for &pixel in row {
                output.push(if pixel { '█' } else { ' ' });
            }
            output.push('\n');
        }
        
        output
    }
    
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

// Helper functions for frame manipulation
pub fn create_frame_from_value(value: &Value) -> Result<Frame> {
    match value {
        Value::Frame(frame) => Ok(frame.clone()),
        Value::Frames(frames) => {
            if frames.is_empty() {
                Err(GizmoError::RuntimeError("Cannot create frame from empty frames array".to_string()))
            } else {
                Ok(frames[0].clone())
            }
        }
        _ => Err(GizmoError::TypeError(
            format!("Cannot convert {} to frame", value.type_name())
        ))
    }
}

pub fn value_to_bool(value: &Value) -> Result<bool> {
    match value {
        Value::Boolean(b) => Ok(*b),
        Value::Number(n) => Ok(*n != 0.0),
        _ => Err(GizmoError::TypeError(
            format!("Cannot convert {} to boolean", value.type_name())
        ))
    }
}

pub fn normalize_frame_size(frame: &mut Frame, target_width: usize, target_height: usize) {
    if frame.width == target_width && frame.height == target_height {
        return;
    }
    
    let mut new_pixels = vec![vec![false; target_width]; target_height];
    
    // Copy existing pixels, centered
    let start_row = if target_height > frame.height {
        (target_height - frame.height) / 2
    } else {
        0
    };
    let start_col = if target_width > frame.width {
        (target_width - frame.width) / 2
    } else {
        0
    };
    
    for row in 0..frame.height.min(target_height) {
        for col in 0..frame.width.min(target_width) {
            let target_row = start_row + row;
            let target_col = start_col + col;
            
            if target_row < target_height && target_col < target_width {
                new_pixels[target_row][target_col] = frame.pixels[row][col];
            }
        }
    }
    
    frame.width = target_width;
    frame.height = target_height;
    frame.pixels = new_pixels;
}

// Animation utilities
pub struct AnimationState {
    pub current_frame: usize,
    pub frames: Vec<Frame>,
    pub loop_animation: bool,
    pub frame_duration: u64, // milliseconds
    pub last_frame_time: u64,
}

impl AnimationState {
    pub fn new(frames: Vec<Frame>, loop_animation: bool, frame_duration: u64) -> Self {
        Self {
            current_frame: 0,
            frames,
            loop_animation,
            frame_duration,
            last_frame_time: 0,
        }
    }
    
    pub fn update(&mut self, current_time: u64) -> Option<&Frame> {
        if self.frames.is_empty() {
            return None;
        }
        
        if current_time - self.last_frame_time >= self.frame_duration {
            self.last_frame_time = current_time;
            self.current_frame += 1;
            
            if self.current_frame >= self.frames.len() {
                if self.loop_animation {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                }
            }
        }
        
        self.frames.get(self.current_frame)
    }
    
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_frame_time = 0;
    }
    
    pub fn is_finished(&self) -> bool {
        !self.loop_animation && self.current_frame >= self.frames.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_creation() {
        let data = vec![
            vec![true, false, true],
            vec![false, true, false],
        ];
        
        let frame = Frame::from_array(data).unwrap();
        assert_eq!(frame.width, 3);
        assert_eq!(frame.height, 2);
        assert_eq!(frame.get_pixel(0, 0).unwrap(), true);
        assert_eq!(frame.get_pixel(1, 1).unwrap(), true);
        assert_eq!(frame.get_pixel(0, 1).unwrap(), false);
    }
    
    #[test]
    fn test_count_neighbors() {
        let data = vec![
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ];
        
        let frame = Frame::from_array(data).unwrap();
        
        // Center cell should have 4 neighbors
        assert_eq!(frame.count_neighbors(1, 1), 4);
        
        // Corner cell should have 1 neighbor
        assert_eq!(frame.count_neighbors(0, 0), 1);
    }
    
    #[test]
    fn test_frame_rotation() {
        let data = vec![
            vec![true, false],
            vec![false, false],
        ];
        
        let frame = Frame::from_array(data).unwrap();
        let rotated = frame.rotate_90();
        
        assert_eq!(rotated.width, 2);
        assert_eq!(rotated.height, 2);
        assert_eq!(rotated.get_pixel(0, 0).unwrap(), false);
        assert_eq!(rotated.get_pixel(0, 1).unwrap(), true);
    }
}
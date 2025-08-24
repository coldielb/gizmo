//! Gizmo - Pixel Art Desktop Buddy Application
//!
//! This is the main entry point for Gizmo, a cross-platform desktop application that displays
//! animated pixel art from custom .gzmo script files. The application features:
//!
//! - CLI interface with commands: `start`, `stop`, `restart`
//! - Background process management that survives terminal closure
//! - Cross-platform windowing with draggable, always-on-top behavior
//! - Custom scripting language with mathematical expressions and pattern generation
//! - High-performance animation support (1ms to 10000ms per frame)
//!
//! ## Architecture Overview
//!
//! The application consists of several key modules:
//! - **lexer**: Tokenizes .gzmo script files into lexical tokens
//! - **parser**: Parses tokens into an Abstract Syntax Tree using operator precedence
//! - **ast**: Defines the data structures for the language's syntax tree
//! - **interpreter**: Executes the AST and generates animation frames
//! - **builtin**: Implements built-in mathematical and animation functions
//! - **frame**: Handles frame rendering utilities
//! - **error**: Provides comprehensive error handling across all modules
//! - **daemon**: Manages background process lifecycle and state persistence
//!
//! ## Process Architecture
//!
//! Gizmo uses a dual-process architecture:
//! 1. **CLI Process**: Handles user commands and spawns the GUI process
//! 2. **GUI Process**: Runs the desktop window and animation loop in the background
//!
//! This separation allows the desktop buddy to persist even after the terminal is closed.

mod lexer;
mod parser;
mod ast;
mod interpreter;
mod builtin;
mod frame;
mod error;
mod daemon;

use std::{env, fs, path::Path, process, time::Duration, thread, rc::Rc};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use softbuffer::{Context, Surface};
use ast::Frame;

/// Main entry point for the Gizmo application.
///
/// Handles command-line argument parsing and dispatches to appropriate handlers:
/// - `--gui <file>`: Internal flag to run the desktop window (used by daemon)
/// - `start <file>`: Start Gizmo with specified .gzmo animation file
/// - `stop`: Stop the currently running Gizmo instance
/// - `restart`: Restart Gizmo with the last used animation file
///
/// The main function implements the CLI interface while delegating the actual
/// GUI functionality to a separate background process for persistence.
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "--gui" => {
            // This is the GUI process - run the desktop window directly
            if args.len() < 3 {
                eprintln!("Internal error: gui missing gzmo file argument");
                process::exit(1);
            }
            let gzmo_file = &args[2];
            if let Err(e) = run_desktop_window(gzmo_file) {
                eprintln!("Error running gizmo window: {}", e);
                // Clean up daemon state on exit
                let _ = daemon::cleanup_daemon_state();
                process::exit(1);
            }
        }
        "start" => {
            if args.len() < 3 {
                eprintln!("Usage: gizmo start <path-to-gzmo-file>");
                process::exit(1);
            }
            let gzmo_file = &args[2];
            if let Err(e) = start_gizmo(gzmo_file) {
                eprintln!("Error starting gizmo: {}", e);
                process::exit(1);
            }
        }
        "stop" => {
            if let Err(e) = stop_gizmo() {
                eprintln!("Error stopping gizmo: {}", e);
                process::exit(1);
            }
        }
        "restart" => {
            if let Err(e) = restart_gizmo() {
                eprintln!("Error restarting gizmo: {}", e);
                process::exit(1);
            }
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}

/// Prints usage information for the Gizmo CLI.
///
/// Displays the available commands and their descriptions to help users
/// understand how to interact with the application.
fn print_usage() {
    println!("Gizmo - Pixel Art Desktop Buddy");
    println!();
    println!("Usage:");
    println!("  gizmo start <path-to-gzmo-file>  Start gizmo with specified animation file");
    println!("  gizmo restart                    Restart current gizmo animation");
    println!("  gizmo stop                       Stop gizmo");
}

/// Starts a new Gizmo instance with the specified .gzmo animation file.
///
/// This function:
/// 1. Validates the input file exists and has the correct extension
/// 2. Saves the file path for future restart operations
/// 3. Checks that no Gizmo instance is already running
/// 4. Spawns a detached GUI process using nohup for background execution
/// 5. Saves the process ID for future stop/restart operations
///
/// # Arguments
/// * `gzmo_file` - Path to the .gzmo script file to execute
///
/// # Returns
/// * `Ok(())` if the Gizmo instance started successfully
/// * `Err` if file validation fails, daemon is already running, or process spawn fails
///
/// # Process Management
/// Uses nohup to detach the GUI process from the terminal, allowing it to persist
/// even after the terminal is closed. The process ID is saved for later management.
fn start_gizmo(gzmo_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Validate file exists and has .gzmo extension
    let path = Path::new(gzmo_file);
    if !path.exists() {
        return Err(format!("File not found: {}", gzmo_file).into());
    }
    
    if !gzmo_file.ends_with(".gzmo") {
        return Err("File must have .gzmo extension".into());
    }

    // Save current gzmo file for restart command
    daemon::save_current_file(gzmo_file)?;

    // Check if daemon is already running
    if daemon::is_daemon_running()? {
        return Err("Gizmo is already running. Use 'gizmo stop' first.".into());
    }

    println!("Starting Gizmo with: {}", gzmo_file);
    
    // Use nohup to detach the GUI process from the terminal
    let current_exe = std::env::current_exe()?;
    let absolute_gzmo_path = std::fs::canonicalize(gzmo_file)?;
    
    let child = process::Command::new("nohup")
        .arg(&current_exe)
        .arg("--gui")
        .arg(&absolute_gzmo_path)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .stdin(process::Stdio::null())
        .spawn()?;
    
    // Save the child PID directly
    let pid = child.id();
    daemon::save_daemon_pid(pid)?;
    
    // Give it a moment to start
    thread::sleep(Duration::from_millis(500));
    
    println!("Gizmo started in background (PID: {})", pid);
    
    Ok(())
}

/// Stops the currently running Gizmo instance.
///
/// Delegates to the daemon module to terminate the background GUI process
/// and clean up associated state files.
///
/// # Returns
/// * `Ok(())` if the daemon was stopped successfully
/// * `Err` if no daemon is running or termination fails
fn stop_gizmo() -> Result<(), Box<dyn std::error::Error>> {
    daemon::stop_daemon()?;
    Ok(())
}

/// Restarts Gizmo with the previously used animation file.
///
/// This function:
/// 1. Retrieves the last used .gzmo file path from daemon state
/// 2. Stops the current Gizmo instance if running
/// 3. Waits briefly for clean shutdown
/// 4. Starts a new instance with the saved file
///
/// # Returns
/// * `Ok(())` if restart completed successfully
/// * `Err` if no previous file is found, stop fails, or start fails
///
/// # Timing
/// Includes a 500ms delay between stop and start to ensure clean process termination.
fn restart_gizmo() -> Result<(), Box<dyn std::error::Error>> {
    let current_file = daemon::get_current_file()?;
    stop_gizmo()?;
    thread::sleep(Duration::from_millis(500)); // Give it time to stop
    start_gizmo(&current_file)
}

/// Runs the desktop window GUI process for displaying Gizmo animations.
///
/// This is the core GUI function that:
/// 1. Loads and parses the .gzmo script file into animation frames
/// 2. Creates a borderless, draggable window positioned at screen center
/// 3. Sets up platform-specific always-on-top behavior (macOS implementation included)
/// 4. Implements an optimized animation loop with two timing modes:
///    - **Polling mode**: For fast animations (<20ms) - continuous redraw requests
///    - **Wait mode**: For slower animations (≥20ms) - efficient sleep-based timing
/// 5. Handles mouse input for window dragging functionality
///
/// # Arguments
/// * `gzmo_file` - Path to the .gzmo script file to execute and display
///
/// # Returns
/// * `Ok(())` if the window ran and closed successfully
/// * `Err` if script loading fails, window creation fails, or runtime errors occur
///
/// # Platform Notes
/// - **macOS**: Uses Objective-C runtime to set window level for always-on-top behavior
/// - **Cross-platform**: Window dragging implemented using winit mouse events
///
/// # Performance Optimization
/// The animation timing system automatically switches between polling and wait modes
/// based on frame duration to balance responsiveness with CPU efficiency.
fn run_desktop_window(gzmo_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse the gizmo file
    let (animation_frames, frame_duration_ms) = load_gizmo_animation(gzmo_file)?;
    
    // Create window
    let event_loop = EventLoop::new()?;
    
    let window_size = 128;
    
    let window = Rc::new(WindowBuilder::new()
        .with_title("Gizmo")
        .with_inner_size(winit::dpi::LogicalSize::new(window_size, window_size))
        .with_resizable(false)
        .with_decorations(false) // Remove window borders and bars
        .with_visible(true)
        .build(&event_loop)?);

    // Back to exact center position that worked before
    let primary_monitor = event_loop.primary_monitor().unwrap();
    let screen_size = primary_monitor.size();
    
    let center_x = screen_size.width as i32 / 2 - window_size / 2;
    let center_y = screen_size.height as i32 / 2 - window_size / 2;
    
    window.set_outer_position(winit::dpi::LogicalPosition::new(center_x, center_y));

    // Set window to always be on top using platform-specific code
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use objc::runtime::Object;
        use objc::*;
        
        // SAFETY: This uses macOS-specific Objective-C runtime to set window level.
        // Level 3 corresponds to NSFloatingWindowLevel, making the window float above others.
        // This is safe because:
        // 1. We verify we have a valid AppKit handle before casting
        // 2. The NSView -> NSWindow relationship is guaranteed by winit
        // 3. setLevel: is a standard NSWindow method
        unsafe {
            if let Ok(handle) = window.window_handle() {
                if let RawWindowHandle::AppKit(appkit_handle) = handle.as_raw() {
                    let ns_view = appkit_handle.ns_view.as_ptr() as *mut Object;
                    let ns_window: *mut Object = msg_send![ns_view, window];
                    let _: () = msg_send![ns_window, setLevel: 3i64];
                }
            }
        }
    }

    // Make sure window is visible and focused
    window.set_visible(true);
    window.focus_window();
    
    // Initialize softbuffer
    let context = Context::new(window.as_ref())?;
    let mut surface = Surface::new(&context, window.as_ref())?;

    let mut frame_index = 0;
    let mut last_frame_time = std::time::Instant::now();
    let frame_duration = Duration::from_millis(frame_duration_ms);

    // Variables for dragging
    let mut is_dragging = false;
    let mut drag_start_pos: Option<winit::dpi::PhysicalPosition<f64>> = None;
    let mut window_start_pos: Option<winit::dpi::PhysicalPosition<i32>> = None;

    let window_clone = window.clone();
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                // Clean up daemon state when window is closed
                let _ = daemon::cleanup_daemon_state();
                elwt.exit();
            }
            // Handle mouse input for window dragging functionality
            Event::WindowEvent { event: WindowEvent::MouseInput { state, button, .. }, .. } => {
                if button == winit::event::MouseButton::Left {
                    match state {
                        winit::event::ElementState::Pressed => {
                            // Start dragging: prepare to track mouse movement
                            is_dragging = true;
                            drag_start_pos = None; // Will be set on first mouse move
                            if let Ok(pos) = window_clone.outer_position() {
                                window_start_pos = Some(pos);
                            }
                        }
                        winit::event::ElementState::Released => {
                            // End dragging: reset tracking state
                            is_dragging = false;
                            drag_start_pos = None;
                            window_start_pos = None;
                        }
                    }
                }
            }
            // Handle cursor movement for window dragging
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                if is_dragging {
                    // Initialize drag reference point on first movement
                    if drag_start_pos.is_none() {
                        drag_start_pos = Some(position);
                    }
                    
                    // Calculate and apply window position delta
                    if let (Some(drag_start), Some(window_start)) = (drag_start_pos, window_start_pos) {
                        let delta_x = position.x - drag_start.x;
                        let delta_y = position.y - drag_start.y;
                        
                        let new_x = window_start.x + delta_x as i32;
                        let new_y = window_start.y + delta_y as i32;
                        
                        // Move window to new position (ignore errors - non-critical)
                        let _ = window_clone.set_outer_position(winit::dpi::PhysicalPosition::new(new_x, new_y));
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } => {
                if window_id == window_clone.id() {
                    // Update animation frame
                    if last_frame_time.elapsed() >= frame_duration && !animation_frames.is_empty() {
                        frame_index = (frame_index + 1) % animation_frames.len();
                        last_frame_time = std::time::Instant::now();
                    }

                    // Render current frame
                    let (width, height) = {
                        let size = window_clone.inner_size();
                        (size.width, size.height)
                    };

                    surface.resize(width.try_into().unwrap(), height.try_into().unwrap()).unwrap();
                    let mut buffer = surface.buffer_mut().unwrap();

                    // Clear buffer to black
                    buffer.fill(0x000000);

                    // Draw current animation frame if available
                    if !animation_frames.is_empty() {
                        let current_frame = &animation_frames[frame_index];
                        draw_frame_to_buffer(&mut buffer, current_frame, width as usize, height as usize);
                    }

                    buffer.present().unwrap();
                }
            }
            Event::AboutToWait => {
                // Adaptive timing strategy based on animation speed:
                // Fast animations need continuous polling for smooth playback,
                // while slower animations can use efficient wait-based timing.
                
                if frame_duration_ms < 20 {
                    // POLLING MODE: For high-speed animations (>50 FPS)
                    // Continuously check for frame updates to ensure smooth playback.
                    // This trades CPU efficiency for animation smoothness.
                    elwt.set_control_flow(ControlFlow::Poll);
                    if last_frame_time.elapsed() >= frame_duration {
                        window_clone.request_redraw();
                    }
                } else {
                    // WAIT MODE: For normal-speed animations (≤50 FPS)
                    // Use event loop sleeping to reduce CPU usage while maintaining accuracy.
                    if last_frame_time.elapsed() >= frame_duration {
                        window_clone.request_redraw();
                    } else {
                        // Sleep until the next frame is due, minimizing CPU usage
                        let sleep_duration = frame_duration - last_frame_time.elapsed();
                        elwt.set_control_flow(ControlFlow::WaitUntil(
                            std::time::Instant::now() + sleep_duration
                        ));
                    }
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

/// Loads and processes a .gzmo script file into executable animation frames.
///
/// This function orchestrates the complete compilation pipeline:
/// 1. **File Loading**: Reads the .gzmo script file from disk
/// 2. **Lexical Analysis**: Tokenizes the source code into language tokens
/// 3. **Parsing**: Builds an Abstract Syntax Tree using operator precedence parsing
/// 4. **Interpretation**: Executes the script to generate animation frames
/// 5. **Frame Extraction**: Retrieves the final frames and timing information
///
/// # Arguments
/// * `gzmo_file` - Path to the .gzmo script file to process
///
/// # Returns
/// * `Ok((frames, duration_ms))` - Animation frames and timing on success
/// * `Err` - Compilation or execution error with descriptive message
///
/// # Error Handling
/// Provides detailed error messages for each phase of compilation:
/// - Lexical errors: Invalid characters or malformed tokens
/// - Parse errors: Syntax errors and malformed expressions
/// - Runtime errors: Script execution failures and type mismatches
///
/// # Fallback Behavior
/// If the script produces no animation frames, the function will:
/// 1. Try to use the interpreter's current frame state
/// 2. Fall back to a default smiley face pattern if nothing else is available
fn load_gizmo_animation(gzmo_file: &str) -> Result<(Vec<Frame>, u64), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(gzmo_file)?;
    
    // LEXICAL ANALYSIS PHASE
    // Convert source code into a stream of tokens for parsing
    let mut lexer = lexer::Lexer::new(&content);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexical analysis error: {}", e);
            return Err(format!("Script parsing failed: {}", e).into());
        }
    };
    
    // PARSING PHASE
    // Build Abstract Syntax Tree using operator precedence parsing
    let mut parser = parser::Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            return Err(format!("Script parsing failed: {}", e).into());
        }
    };
    
    // INTERPRETATION PHASE
    // Execute the AST to generate animation frames and extract timing
    let mut interpreter = interpreter::Interpreter::new();
    
    if let Err(e) = interpreter.execute(&ast) {
        eprintln!("Execution error: {}", e);
        return Err(format!("Script execution failed: {}", e).into());
    }
    
    // Extract animation frames and timing from interpreter
    let frames = interpreter.get_animation_frames();
    let frame_duration_ms = interpreter.get_frame_duration_ms();
    
    if frames.is_empty() {
        // If no animation, create a single frame from current state
        if let Some(current_frame) = interpreter.get_current_frame() {
            return Ok((vec![current_frame], frame_duration_ms));
        } else {
            // Create a default smiley face if nothing else
            return Ok((vec![create_default_smiley()], frame_duration_ms));
        }
    }
    
    Ok((frames, frame_duration_ms))
}

/// Creates a default smiley face animation frame as a fallback.
///
/// This function generates a simple 128x128 pixel smiley face pattern
/// that serves as a fallback when:
/// - A .gzmo script produces no frames
/// - The interpreter has no current frame state
/// - Script execution fails but the application should still display something
///
/// # Returns
/// A `Frame` containing a centered smiley face with:
/// - Two rectangular eyes (8x8 pixels each)
/// - A curved smile using horizontal and diagonal lines
///
/// # Pattern Details
/// The smiley is centered in the 128x128 canvas with:
/// - Eyes positioned at (50-58, 50-58) and (70-78, 50-58)
/// - Smile curve from (55-73, 75) with connecting diagonal lines
fn create_default_smiley() -> Frame {
    // Create a simple smiley face pattern
    let mut data = vec![vec![false; 128]; 128];
    
    // Simple smiley in the center
    let _center_x = 64;
    let _center_y = 64;
    
    // Eyes
    for x in 50..=58 {
        for y in 50..=58 {
            data[y][x] = true;
        }
    }
    for x in 70..=78 {
        for y in 50..=58 {
            data[y][x] = true;
        }
    }
    
    // Smile
    for x in 55..=73 {
        data[75][x] = true;
        data[80][x] = true;
    }
    data[76][55] = true;
    data[77][56] = true;
    data[78][57] = true;
    data[79][58] = true;
    
    data[76][73] = true;
    data[77][72] = true;
    data[78][71] = true;
    data[79][70] = true;
    
    Frame::new(data)
}

/// Renders a Gizmo frame to a pixel buffer for display.
///
/// This function handles the conversion from Gizmo's boolean pixel format
/// to the 32-bit ARGB format expected by the graphics system. It includes
/// automatic scaling to fit the frame content to the window size.
///
/// # Arguments
/// * `buffer` - Mutable slice of 32-bit pixels to write to (ARGB format)
/// * `frame` - The Gizmo frame containing boolean pixel data
/// * `width` - Target buffer width in pixels
/// * `height` - Target buffer height in pixels
///
/// # Scaling Behavior
/// - Automatically scales frame content to fit the window dimensions
/// - Maintains aspect ratio by using the same scaling factor for both axes
/// - Uses nearest-neighbor sampling for pixel-perfect scaling
///
/// # Color Mapping
/// - `true` pixels (on) → `0xFFFFFF` (white)
/// - `false` pixels (off) → `0x000000` (black)
///
/// # Safety
/// Uses bounds checking when writing to the buffer to prevent crashes
/// from mismatched buffer sizes.
fn draw_frame_to_buffer(buffer: &mut [u32], frame: &Frame, width: usize, height: usize) {
    let frame_data = frame.get_data();
    let frame_height = frame_data.len();
    let frame_width = if frame_height > 0 { frame_data[0].len() } else { 0 };
    
    // Calculate scaling factors to fit frame to window
    // Uses floating-point arithmetic for smooth scaling
    let scale_x = width as f32 / frame_width as f32;
    let scale_y = height as f32 / frame_height as f32;
    
    // Render each window pixel by sampling from the frame
    for y in 0..height {
        for x in 0..width {
            // Map window coordinates back to frame coordinates
            // Using nearest-neighbor sampling for pixel-perfect results
            let frame_x = (x as f32 / scale_x) as usize;
            let frame_y = (y as f32 / scale_y) as usize;
            
            if frame_y < frame_height && frame_x < frame_width {
                // Convert boolean pixel to 32-bit ARGB color
                let pixel = if frame_data[frame_y][frame_x] {
                    0xFFFFFF // White for "on" pixels
                } else {
                    0x000000 // Black for "off" pixels
                };
                
                // Safely write to buffer with bounds checking
                if let Some(buf_pixel) = buffer.get_mut(y * width + x) {
                    *buf_pixel = pixel;
                }
            }
        }
    }
}
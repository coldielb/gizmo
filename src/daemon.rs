//! Background Process Management for Gizmo
//!
//! This module handles the daemon functionality that allows Gizmo to run as a
//! persistent background process, independent of the terminal that launched it.
//! This enables the desktop buddy to continue running even after the terminal
//! is closed.
//!
//! ## Process Architecture
//!
//! Gizmo uses a dual-process model:
//!
//! 1. **CLI Process**: Handles user commands (`start`, `stop`, `restart`)
//! 2. **GUI Process**: Runs the desktop window and animation loop
//!
//! The CLI process spawns the GUI process using `nohup` to detach it from the
//! terminal, then exits. The GUI process continues running independently.
//!
//! ## State Management
//!
//! The daemon system maintains persistent state in the user's config directory:
//!
//! - **Current File** (`current.txt`): Path to the currently loaded .gzmo file
//! - **Process ID** (`daemon.pid`): PID of the running GUI process
//!
//! This state allows commands like `restart` to work without requiring the
//! user to specify the file path again.
//!
//! ## Process Control
//!
//! ### Starting
//! - Validates .gzmo file exists
//! - Spawns detached GUI process with `nohup`
//! - Saves process PID and file path
//!
//! ### Stopping
//! - Sends SIGTERM to GUI process
//! - Falls back to `pkill` if PID-based termination fails
//! - Cleans up state files
//!
//! ### Restarting
//! - Retrieves saved file path
//! - Stops current process
//! - Starts new process with same file
//!
//! ## Platform Compatibility
//!
//! Currently designed for Unix-like systems (macOS, Linux) with:
//! - `nohup` for process detachment
//! - `kill` for process termination
//! - `pkill` for fallback termination
//!
//! Future versions could extend support to Windows with equivalent mechanisms.

use std::fs;
use std::path::PathBuf;
use dirs;

/// Gets the Gizmo configuration directory, creating it if necessary.
///
/// Locates the user's standard configuration directory and creates a `gizmo`
/// subdirectory for storing daemon state files. The directory is automatically
/// created if it doesn't exist.
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the Gizmo config directory
/// * `Err` - If config directory can't be found or created
///
/// # Directory Location
/// - **macOS**: `~/Library/Application Support/gizmo/`
/// - **Linux**: `~/.config/gizmo/`
/// - **Windows**: `%APPDATA%\gizmo\` (if supported)
///
/// # Files Stored
/// - `current.txt` - Path to currently loaded .gzmo file
/// - `daemon.pid` - Process ID of running GUI instance
pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?;
    config_dir.push("gizmo");
    
    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    
    Ok(config_dir)
}

/// Saves the current .gzmo file path for future restart operations.
///
/// Stores the absolute path to the currently loaded .gzmo file so that
/// the `restart` command can reload the same file without requiring the
/// user to specify it again.
///
/// # Arguments
/// * `file_path` - Absolute path to the .gzmo file to save
///
/// # Returns
/// * `Ok(())` - File path saved successfully
/// * `Err` - I/O error writing to config file
///
/// # State File
/// The path is stored in `{config_dir}/current.txt` as plain text.
pub fn save_current_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let current_file_path = config_dir.join("current.txt");
    fs::write(current_file_path, file_path)?;
    Ok(())
}

/// Retrieves the currently saved .gzmo file path for restart operations.
///
/// Reads the file path that was saved by a previous `start` command,
/// enabling the `restart` command to reload the same animation.
///
/// # Returns
/// * `Ok(String)` - Absolute path to the saved .gzmo file
/// * `Err` - If no file is saved or I/O error reading config
///
/// # Error Cases
/// - No previous `start` command has been run
/// - Config file is corrupted or unreadable
/// - File system permissions prevent access
pub fn get_current_file() -> Result<String, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let current_file_path = config_dir.join("current.txt");
    
    if !current_file_path.exists() {
        return Err("No current file found. Use 'gizmo start <file>' first.".into());
    }
    
    let content = fs::read_to_string(current_file_path)?;
    Ok(content.trim().to_string())
}

/// Saves the GUI process ID for future process management.
///
/// Stores the PID of the detached GUI process so that `stop` and `restart`
/// commands can terminate it cleanly. The PID is saved immediately after
/// successful process spawn.
///
/// # Arguments
/// * `pid` - Process ID of the GUI process to track
///
/// # Returns
/// * `Ok(())` - PID saved successfully
/// * `Err` - I/O error writing to config file
///
/// # State File
/// The PID is stored in `{config_dir}/daemon.pid` as plain text.
pub fn save_daemon_pid(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let pid_file_path = config_dir.join("daemon.pid");
    fs::write(pid_file_path, pid.to_string())?;
    Ok(())
}

/// Retrieves the saved GUI process ID for process management.
///
/// Reads the PID that was saved when the GUI process was started,
/// enabling `stop` and `restart` commands to control the process.
///
/// # Returns
/// * `Ok(u32)` - Process ID of the running GUI process
/// * `Err` - If no PID is saved or parsing fails
///
/// # Error Cases
/// - No daemon is currently tracked (no start command run)
/// - PID file is corrupted or contains invalid data
/// - File system permissions prevent access
pub fn get_daemon_pid() -> Result<u32, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let pid_file_path = config_dir.join("daemon.pid");
    
    if !pid_file_path.exists() {
        return Err("No daemon PID found".into());
    }
    
    let content = fs::read_to_string(pid_file_path)?;
    let pid: u32 = content.trim().parse()?;
    Ok(pid)
}

/// Checks if a Gizmo daemon process is currently running.
///
/// Uses the saved PID to check if the GUI process is still alive.
/// This prevents starting multiple instances and provides accurate
/// status information.
///
/// # Returns
/// * `Ok(true)` - Daemon is running
/// * `Ok(false)` - No daemon running or process is dead
/// * `Err` - System error checking process status
///
/// # Implementation
/// Uses `kill -0 <pid>` which checks process existence without
/// sending any signal. This is a standard Unix technique for
/// testing process liveness.
pub fn is_daemon_running() -> Result<bool, Box<dyn std::error::Error>> {
    match get_daemon_pid() {
        Ok(pid) => {
            // Use kill -0 to test if process exists (doesn't send signal)
            use std::process::Command;
            let output = Command::new("kill")
                .arg("-0")  // Test signal - checks existence without killing
                .arg(pid.to_string())
                .output()?;
            Ok(output.status.success())
        }
        Err(_) => Ok(false),  // No PID file = no daemon running
    }
}

/// Stops the currently running Gizmo daemon process.
///
/// Attempts to gracefully terminate the GUI process using SIGTERM,
/// with fallback mechanisms for robust process cleanup.
///
/// # Returns
/// * `Ok(())` - Daemon stopped successfully
/// * `Err` - No daemon running or termination failed
///
/// # Termination Strategy
/// 1. **Primary**: Send SIGTERM to saved PID for clean shutdown
/// 2. **Fallback**: Use `pkill -f "gizmo --gui"` to kill by process name
/// 3. **Cleanup**: Remove state files regardless of method used
///
/// # Process Signals
/// - **SIGTERM (-TERM)**: Requests graceful termination, allows cleanup
/// - **SIGKILL** (not used): Would force termination without cleanup
///
/// The graceful approach allows the GUI process to clean up resources
/// like window handles and animation state before exiting.
pub fn stop_daemon() -> Result<(), Box<dyn std::error::Error>> {
    match get_daemon_pid() {
        Ok(pid) => {
            use std::process::Command;
            // Try graceful termination with SIGTERM
            let output = Command::new("kill")
                .arg("-TERM")  // Graceful termination signal
                .arg(pid.to_string())
                .output()?;
            
            if output.status.success() {
                cleanup_daemon_state()?;
                println!("Gizmo stopped (PID: {})", pid);
            } else {
                // Fallback: kill by process name pattern
                let _ = Command::new("pkill")
                    .arg("-f")  // Match full command line
                    .arg("gizmo --gui")
                    .output();
                cleanup_daemon_state()?;
                println!("Gizmo stopped");
            }
        }
        Err(_) => {
            // No saved PID - try fallback method anyway
            use std::process::Command;
            let output = Command::new("pkill")
                .arg("-f")
                .arg("gizmo --gui")
                .output()?;
            
            if output.status.success() {
                cleanup_daemon_state()?;
                println!("Gizmo stopped");
            } else {
                return Err("Gizmo is not running".into());
            }
        }
    }
    Ok(())
}

/// Cleans up daemon state files after process termination.
///
/// Removes the PID file to prevent stale state from interfering with
/// future daemon operations. Called automatically after successful
/// process termination.
///
/// # Returns
/// * `Ok(())` - State cleaned up successfully
/// * `Err` - I/O error removing state files
///
/// # Files Cleaned
/// - `daemon.pid` - Removed to indicate no process is running
/// - `current.txt` - Preserved to allow restart with same file
///
/// # Design Note
/// The current file path is intentionally preserved so that `restart`
/// can still work after a `stop` operation. Only the PID file is removed
/// since it represents active process state.
pub fn cleanup_daemon_state() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let pid_file_path = config_dir.join("daemon.pid");
    
    // Remove PID file if it exists
    if pid_file_path.exists() {
        fs::remove_file(pid_file_path)?;
    }
    
    // Note: current.txt is preserved for restart functionality
    
    Ok(())
}
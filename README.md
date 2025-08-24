# Gizmo

A desktop buddy application with a custom scripting language for creating procedural pixel art animations. Built in Rust.

Very much a work in progress. Feedback is appreciated via GitHub issues/discussions.

## Installation

```bash
git clone https://github.com/coldielb/gizmo.git
cd gizmo
./install.sh
```

Or build manually:
```bash
cargo build --release
```

## Usage

```bash
gizmo start <script.gzmo>    # Start animation
gizmo restart                # Restart current animation
gizmo stop                   # Stop animation
```

## Example

Create `test.gzmo`:
```gizmo
frames animation = [];

repeat 30 times do
    frame circle = pattern(64, 64) {
        center = 32;
        dx = col - center;
        dy = row - center;
        distance = sqrt(dx * dx + dy * dy);

        return distance < 20 ? 1 : 0;
    };

    add_frame(animation, circle);
end;

loop_speed(animation, 100);
```

Run it:
```bash
gizmo start test.gzmo
```

## Language Features

- Pattern generation with mathematical expressions
- Control flow (if/then/else, repeat loops)
- Mathematical functions (sin, cos, sqrt, atan2, abs, floor, ceil, random)
- Variables and assignments
- Animation control functions

## File Structure

```
src/
├── main.rs           # CLI and window management
├── lexer.rs          # Tokenization
├── parser.rs         # Parser
├── ast.rs            # Abstract syntax tree
├── interpreter.rs    # Script execution
├── builtin.rs        # Built-in functions
├── error.rs          # Error handling
├── frame.rs          # Frame utilities
└── daemon.rs         # Background process management

examples/             # Example scripts
syntax.md             # Language reference
```

## Dependencies

- winit 0.29 - Cross-platform windowing
- softbuffer 0.4 - Pixel buffer rendering
- serde - Configuration serialization
- dirs - Cross-platform directories
- objc, cocoa - macOS window management
- rand 0.8 - Random number generation

## Documentation

- Language reference: syntax.md
- Example scripts: examples/

## License

MIT License

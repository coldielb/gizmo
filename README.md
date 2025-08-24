# Gizmo - Procedural Pixel Art Desktop Buddy

A complete desktop buddy application featuring a powerful custom scripting language for creating procedural pixel art animations. Built in pure Rust with high-performance animation support and cross-platform windowing.

![Gizmo Logo](https://img.shields.io/badge/Gizmo-Desktop%20Buddy-blue) 
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Language](https://img.shields.io/badge/language-Rust-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## 🎯 Features

### ✨ Complete Desktop Application
- **128x128 pixel window** with always-on-top behavior
- **Draggable interface** for perfect desktop buddy positioning
- **Cross-platform support** (Windows, macOS, Linux)
- **Background process management** that survives terminal closure
- **High-performance animation** supporting 1ms to 10000ms frame rates

### 🎨 Powerful Scripting Language
- **Procedural pattern generation** using mathematical expressions
- **Complete programming language** with variables, functions, and control flow
- **Real-time animation creation** with dynamic frame generation
- **Mathematical functions** including trigonometry, randomization, and more
- **Logical operations** with proper operator precedence

### 🛠 Professional Features
- **Comprehensive error handling** with detailed debugging information
- **Hot reloading** support for script development
- **Memory safe** implementation in Rust
- **Extensive documentation** and examples
- **CLI management** with simple start/stop/restart commands

## 🚀 Quick Start

### Installation

**Option 1: Easy Install (Recommended)**
```bash
git clone https://github.com/yourusername/gizmo.git
cd gizmo
./install.sh
```
The install script will build the release binary and install `gizmo` to your PATH so you can use it from anywhere.

**Option 2: Manual Build**
```bash
git clone https://github.com/yourusername/gizmo.git
cd gizmo
cargo build --release
# Binary will be at target/release/gizmo
```

### Running Your First Animation

```bash
# Start with a built-in example
cargo run -- start examples/spinner.gzmo

# Stop the animation
cargo run -- stop

# Restart current animation
cargo run -- restart
```

### Create Your First Script

Create a file called `my_animation.gzmo`:

```gizmo
frames animation = [];

repeat 30 times do
    frame circle_frame = pattern(64, 64) {
        center = 32;
        dx = col - center;
        dy = row - center;
        distance = sqrt(dx * dx + dy * dy);
        
        pulse = sin(time * 0.2) * 5;
        
        return distance >= 15 - pulse and distance <= 20 - pulse ? 1 : 0;
    };
    
    add_frame(animation, circle_frame);
end;

loop_speed(animation, 100);
```

Run it:
```bash
cargo run -- start my_animation.gzmo
```

## 📚 Documentation

- **[Language Reference](syntax.md)** - Complete Gizmo scripting language documentation
- **[Examples Directory](examples/)** - Ready-to-run animation scripts
- **[API Documentation](#)** - Generated from source code comments

## 🎨 Example Animations

### Rotating Spiral
```gizmo
frames spiral = [];

repeat 48 times do
    frame f = pattern(64, 64) {
        dx = col - 32;
        dy = row - 32;
        distance = sqrt(dx * dx + dy * dy);
        angle = atan2(dy, dx);
        
        rotation = time * 0.13;
        spiral_wave = sin(distance * 0.2 - rotation) + 
                     sin(angle * 3 + rotation * 2);
                     
        return spiral_wave > 0.3 ? 1 : 0;
    };
    
    add_frame(spiral, f);
end;

loop_speed(spiral, 80);
```

### Pulsing Heart
```gizmo
frames heart_animation = [];

repeat 20 times do
    frame heart = pattern(32, 32) {
        x = col - 16;
        y = row - 16;
        
        pulse = 1.0 + sin(time * 0.5) * 0.3;
        
        heart_shape = ((x*x + y*y - 100*pulse) * (x*x + y*y - 100*pulse) * 
                      (x*x + y*y - 100*pulse)) - 
                      (x*x * y*y*y * pulse * pulse);
                      
        return heart_shape <= 0 ? 1 : 0;
    };
    
    add_frame(heart_animation, heart);
end;

loop_speed(heart_animation, 150);
```

## 🏗 Architecture

### Core Components
```
gizmo/
├── src/
│   ├── main.rs           # CLI and window management
│   ├── lexer.rs          # Tokenization and lexical analysis  
│   ├── parser.rs         # Recursive descent parser
│   ├── ast.rs            # Abstract syntax tree definitions
│   ├── interpreter.rs    # Script execution engine
│   ├── builtin.rs        # Built-in functions (math, animation)
│   ├── error.rs          # Comprehensive error handling
│   ├── frame.rs          # Frame rendering utilities  
│   └── daemon.rs         # Background process management
├── examples/             # Example animation scripts
├── syntax.md             # Language documentation
└── README.md            # This file
```

### Language Pipeline
```
.gzmo file → Lexer → Parser → AST → Interpreter → Animation Frames → Desktop Window
```

### Built-in Functions

**Mathematical:**
- `sin(x)`, `cos(x)`, `sqrt(x)`, `atan2(y, x)`  
- `abs(x)`, `floor(x)`, `ceil(x)`, `random()`

**Animation:**
- `add_frame(array, frame)` - Add frame to animation sequence
- `loop_speed(array, ms)` - Set timing and start animation
- `play(array)`, `loop(array)` - Animation control

## ⚡ Performance

- **High-speed animations**: Optimized for 1ms frame rates
- **Efficient parsing**: Single-pass lexer and recursive descent parser  
- **Memory efficient**: Frames cached and reused during playback
- **Adaptive timing**: Automatically adjusts between polling and waiting modes

## 🎛 CLI Commands

```bash
gizmo start <script.gzmo>    # Start animation with script
gizmo restart                # Restart current animation
gizmo stop                   # Stop animation and exit
```

## 🔧 Development

### Building from Source
```bash
cargo build                  # Debug build
cargo build --release       # Optimized build
cargo test                   # Run tests
cargo doc --open            # Generate and view documentation
```

### Dependencies
- `winit 0.29` - Cross-platform windowing
- `softbuffer 0.4` - Pixel buffer rendering  
- `serde` - Configuration serialization
- `dirs` - Cross-platform directories
- `objc`, `cocoa` - macOS-specific window management
- `rand 0.8` - Random number generation

## 🐛 Troubleshooting

### Common Issues

**Animation not visible:**
- Window may be off-screen - the window is draggable, try different screen areas
- Check if process is running: look for gizmo processes in Activity Monitor/Task Manager

**Parse errors:**
- Check syntax.md for complete language reference
- Ensure proper `end` statements for `if` and `repeat` blocks
- Verify all function calls have correct number of arguments

**Performance issues:**
- Large patterns (128x128+) with complex math may impact performance
- Try reducing pattern size or simplifying mathematical expressions
- Use appropriate `loop_speed()` values (50-200ms for complex patterns)

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with appropriate tests and documentation
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

### Development Guidelines

- **Code Style**: Follow standard Rust formatting (`cargo fmt`)
- **Testing**: Add tests for new features (`cargo test`)
- **Documentation**: Update documentation for API changes
- **Performance**: Consider performance impact of changes
- **Compatibility**: Maintain cross-platform compatibility

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with the amazing Rust ecosystem
- Inspired by classic desktop buddy applications
- Mathematical art patterns inspired by demoscene and creative coding communities

## 📈 Roadmap

### Upcoming Features
- **Sound support** - Audio synchronized animations
- **Mouse interaction** - Click-responsive animations  
- **Network features** - Shared animations between desktop buddies
- **Plugin system** - Custom built-in function extensions
- **Visual editor** - GUI for creating animations
- **More examples** - Expanding the example library

---

**Made with ❤️ in Rust** 

*Gizmo: Where mathematics meets pixel art*
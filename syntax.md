# Gizmo Language Syntax Reference

## Core Design Philosophy
- **Procedural pixel art generation** with mathematical expressions
- **Dynamic animation creation** using pattern generators and loops
- **High-performance rendering** supporting 1ms to 10000ms frame rates
- **Extensible scripting** with full mathematical and logical operations
- **Real-time desktop buddy** with draggable, always-on-top behavior

## Language Overview

Gizmo is a complete scripting language for creating procedural pixel art animations. It combines:
- **Mathematical expressions** with proper operator precedence
- **Pattern generation** for algorithmic art creation
- **Control flow** with conditionals and loops
- **Animation control** with precise timing
- **Built-in functions** for mathematical operations

## Data Types

### Frame
A 2D array of pixels (1 = on, 0 = off) representing a single animation frame.
```gizmo
frame simple = [
    [0,1,0],
    [1,1,1],
    [0,1,0]
];
```

### Frames
An array of frame objects for creating animations.
```gizmo
frames sequence = [];
// Frames are added dynamically using add_frame()
```

### Numbers
Floating-point numbers supporting full mathematical operations.
```gizmo
center_x = 32;
radius = 15.5;
angle = 3.14159 / 4;
```

## Pattern Generation

### Dynamic Pattern Creation
The most powerful feature - generate frames using mathematical expressions:
```gizmo
frame circle = pattern(64, 64) {
    center_x = 32;
    center_y = 32;
    
    dx = col - center_x;
    dy = row - center_y;
    distance = sqrt(dx * dx + dy * dy);
    
    return distance <= 20 ? 1 : 0;
};
```

### Pattern Variables
Inside patterns, these variables are automatically available:
- `row` - Current pixel row (0 to height-1)
- `col` - Current pixel column (0 to width-1) 
- `time` - Current iteration in repeat loops

## Operators

### Mathematical Operators (with proper precedence)
```gizmo
result = 2 + 3 * 4;        // 14 (multiplication first)
value = (x + y) / 2;       // Parentheses override precedence
modulo = frame_num % 8;    // Modulo operation
```

### Comparison Operators
```gizmo
is_center = distance < 10;
at_edge = x >= width or y >= height;
exact_match = color == target_color;
in_range = value > min and value <= max;
```

### Logical Operators
```gizmo
visible = is_active and not is_hidden;
should_draw = x >= 0 and x < width and y >= 0 and y < height;
is_corner = (x == 0 or x == width-1) and (y == 0 or y == height-1);
```

### Ternary Operator
```gizmo
pixel_value = distance < radius ? 1 : 0;
brightness = is_lit ? (pulsing ? 1 : 0.7) : 0;
```

## Control Flow

### Conditional Statements
```gizmo
if distance > max_radius then
    pixel_value = 0;
else
    pixel_value = sin(distance * 0.1) > 0 ? 1 : 0;
end;
```

### Repeat Loops
```gizmo
frames animation_sequence = [];

repeat 60 times do
    frame current_frame = pattern(128, 128) {
        rotation = time * 0.1;  // 'time' is the loop iteration
        
        angle = atan2(row - 64, col - 64);
        rotated_angle = angle + rotation;
        
        return sin(rotated_angle * 6) > 0 ? 1 : 0;
    };
    
    add_frame(animation_sequence, current_frame);
end;
```

### Variable Assignment
```gizmo
// Simple assignment
radius = 25;
center_point = width / 2;

// Complex expressions
wave_offset = sin(time * 0.2) * 10;
brightness = abs(sin(angle)) * 0.8 + 0.2;
```

## Built-in Functions

### Mathematical Functions
```gizmo
sin(x);          // Sine function
cos(x);          // Cosine function
sqrt(x);         // Square root
atan2(y, x);     // Arc tangent of y/x
abs(x);          // Absolute value
floor(x);        // Round down to integer
ceil(x);         // Round up to integer
random();        // Random number 0.0 to 1.0
```

### Animation Functions
```gizmo
add_frame(frames_array, frame);        // Add frame to animation
loop_speed(frames_array, milliseconds); // Set playback speed and start
play(frames_array);                    // Play once
loop(frames_array);                    // Loop forever
```

## Complete Examples

### Rotating Spiral
```gizmo
frames spiral_animation = [];

repeat 48 times do
    frame spiral_frame = pattern(64, 64) {
        center_x = 32;
        center_y = 32;
        
        dx = col - center_x;
        dy = row - center_y;
        distance = sqrt(dx * dx + dy * dy);
        angle = atan2(dy, dx);
        
        rotation_offset = time * 0.13;
        
        spiral_value = sin(distance * 0.2 - rotation_offset) +
                      sin(angle * 3 + rotation_offset * 2);
        
        threshold = 0.3 + sin(distance * 0.05 + rotation_offset * 0.8) * 0.2;
        
        return spiral_value > threshold ? 1 : 0;
    };
    
    add_frame(spiral_animation, spiral_frame);
end;

loop_speed(spiral_animation, 80);
```

### Conditional Logic Example
```gizmo
frame complex_pattern = pattern(32, 32) {
    center = 16;
    dx = col - center;
    dy = row - center;
    distance = sqrt(dx * dx + dy * dy);
    
    if distance < 5 then
        return 1;
    else
        if distance < 10 then
            angle = atan2(dy, dx);
            spokes = floor(angle * 4 / (2 * 3.14159));
            return spokes % 2;
        else
            return random() > 0.7 ? 1 : 0;
        end;
    end;
};
```

### Loading Spinner
```gizmo
frames spinner_sequence = [];

repeat 24 times do
    frame spinner_frame = pattern(64, 64) {
        center_x = 32;
        center_y = 32;
        
        dx = col - center_x;
        dy = row - center_y;
        distance = sqrt(dx * dx + dy * dy);
        angle = atan2(dy, dx);
        
        rotation = time * 0.26;
        arm_angle = (angle + rotation) % (2 * 3.14159);
        arm_number = floor(arm_angle / (2 * 3.14159 / 6));
        
        arm_active = (arm_number == 0) or
                    (arm_number == 1 and distance > 11) or
                    (arm_number == 2 and distance > 14) or
                    (arm_number == 3 and distance > 17);
        
        on_arm = (arm_number < 4) and
                 (distance >= 8) and
                 (distance <= 28);
        
        center_dot = distance <= 4;
        
        return center_dot or (on_arm and arm_active) ? 1 : 0;
    };
    
    add_frame(spinner_sequence, spinner_frame);
end;

loop_speed(spinner_sequence, 120);
```

## Language Features

### Operator Precedence (highest to lowest)
1. **Parentheses**: `()`
2. **Function calls**: `sin()`, `sqrt()`
3. **Unary operators**: `-x` (future)
4. **Multiplicative**: `*`, `/`, `%`
5. **Additive**: `+`, `-`
6. **Relational**: `<`, `>`, `<=`, `>=`
7. **Equality**: `==`, `!=`
8. **Logical AND**: `and`
9. **Logical OR**: `or`
10. **Ternary**: `condition ? true_expr : false_expr`

### Variable Scope
- **Global variables**: Defined at top level, accessible everywhere
- **Pattern variables**: `row`, `col`, `time` automatically provided in patterns
- **Loop variables**: `time` represents current iteration in repeat loops

### Performance Considerations
- **Fast animations**: Use `loop_speed()` with values 1ms-19ms for smooth high-speed animations
- **Complex patterns**: Large pattern sizes (128x128+) with complex math may impact performance
- **Frame caching**: Frames are generated once and cached for animation playback

## Usage Recommendations

### Best Practices
1. **Start simple**: Begin with small patterns (32x32 or 64x64) for testing
2. **Use mathematical functions**: Leverage `sin()`, `cos()` for smooth, organic animations  
3. **Optimize timing**: Use appropriate `loop_speed()` values for your animation style
4. **Test incrementally**: Build complex patterns step by step
5. **Leverage conditionals**: Use `if-then-else` for complex logic patterns

### Common Patterns
- **Circular patterns**: Use `distance = sqrt(dx*dx + dy*dy)`
- **Radial patterns**: Use `angle = atan2(dy, dx)`
- **Wave animations**: Combine `sin()` with `time` variable
- **Geometric shapes**: Use modulo `%` and conditionals
- **Organic motion**: Layer multiple sine waves with different frequencies

## CLI Usage

```bash
gizmo start examples/animation.gzmo    # Start animation
gizmo restart                          # Restart current animation  
gizmo stop                            # Stop animation
```

The Gizmo window is draggable and stays always-on-top for the perfect desktop buddy experience!
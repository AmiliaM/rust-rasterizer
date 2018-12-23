# Rusterizer

The Rusterizer is an implementation of raster graphics for my graphics class. It has the following features:
* Drawing points on an integer grid with arbitrary RGB color (`R8G8B8`)
* Rasterizing discrete objects
    * Arbitrary line list
    * Rectangle given (top-left, bottom-right)
    * Ellipse given (width, height)
    * Polygon as an edge loop
    * Characters in a line-list "font" (`A-Z0-9` only)
    * String of characters
    * Groups of other objects
* User interface for selecting, grouping, and transforming discrete objects
* Command interface implemented with character string object, for creating objects


## User Interaction
### Commands
Commands are entered on the keyboard and appear as drawn letters. The string object used for commands may be transformed as any other object. The `ENTER` key is used to submit commands. If a command is invalid, the command string will be replaced by the string `"INVALID"`.
#### `ellipse (w) (h)`
Draws an ellipse with width `w`, height `h`, starting at `(100, 100)`.
#### `rect (x0) (y0) (x1) (y1)`
Draws a rectangle with top-left `(x0, y0)` and bottom-right `(x1, y1)`, starting at `(100, 100)`.
### Controls
`LCTRL` is the left control key.

`LSHIFT` is the left shift key.

Directions such as `DOWN` refer to arrow keys.
#### `TAB`
Change current object selection. Current selection is indicated in yellow.
#### `LCTRL+DOWN`
Translate the scene camera down.
#### `LCTRL+UP`
Translate the scene camera up.
#### `LCTRL+LEFT`
Translate the scene camera left.
#### `LCTRL+RIGHT`
Translate the scene camera right.
#### `LSHIFT+DOWN`
Translate the currently selected object down.
#### `LSHIFT+UP`
Translate the currently selected object up.
#### `LSHIFT+LEFT`
Translate the currently selected object left.
#### `LSHIFT+RIGHT`
Translate the currently selected object right.
#### `LCTRL+DASH`
Scale the scene down.
#### `LCTRL+EQUAL`
Scale the scene up.
#### `DASH`
Scale the currently selected object down.
#### `EQUAL`
Scale the currently selected object up.
#### `LCTRL+[`
Rotate the scene left.
#### `LCTRL+]`
Rotate the scene right.
#### `[`
Rotate the currently selected object left.
#### `]`
Rotate the currently selected object right.
#### `LCTRL+X`
If the currently selected object is a group, disband it.
#### `LSHIFT+[0-9]`
Add the currently selected object to the group indicated by the number key pressed.
#### `DOWN`
Write the current scene state to `saved_drawing.json` in the current directory.
#### `UP`
Load the scene state from `saved_drawing.json` in the current directory.

## Implementation Details
### Libraries Used
* Rust standard library
    * Time, IO, formatting, reference counting
* Serde
    * Object serialization, deserialization
    * `serde_json` for JSON format support
* SDL2
    * Window creation, management
    * Canvas for screen drawing primitives (points)
    * Keyboard event handling
### Base Types
* `Point` - `(i32, i32)`
    * 32-bit integer pair representing an `(x, y)` coordinate pair.
* `PColor` - `(u8, u8, u8)`
    * 24-bit 3-tuple of unsigned byte representing a color as `(r, g, b)`.
* `Line` - `(Point, Point)`
    * 64-bit `Point` pair representing a line as a `(start, end)` vector.

### Object Model
* `Scene` - Holds objects, group->number mappings, intrinsic camera (trans+rot+scale)
    * `Object` - Has an underlying shape; holds basic transformation logic and parameters (trans+rot+scale)
        * `Shape` - Enum of shapes parameterized over draw implementation needs; holds draw logic
            * `Circle` - Represents a circle using `width(i32)` and `height(i32)` parameters
            * `Rect` - Represents a rectangle using two `Point`s
            * `Polygon` - Represents a polygon as an edge loop using list of `Point`s
            * `Letters` - Represents a string of characters to be drawn using font
            * `Lines` - Represents an arbitrary list of lines to be drawn
            * `Group` - Represents a group of sub-objects
* `ObjectList` - Holds a list of objects. Newtype of vector of reference-counted, mutable objects. Needed for simplicity, serialization support.
* `VecExt` trait - Extension trait for vectors of points allowing common transforms to be done easily
    * `scissor` - Implements scissorboxing using binary search
    * `scissor_iter` - Implements scissorboxing iteratively
    * `translate` - Translates points by given `x(i32)` and `y(i32)` deltas
    * `rotate` - Rotates points in-place by given `a(f32)`
    * `scale` - Scales points by given `a(f32)` and `b(f32)` (corresponding `x`, `y` axes)
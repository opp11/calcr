Calcr - a small commandline calculator
======================================
This is a small commanline calculator written in Rust.

The following operators, functions, constants, and delimiters can be used in calcr:

#### Operators
```
+        - plus
-        - minus or negation
*        - muliplication
/        - division
^        - powers
!        - factorial (only works on positive integers)
```

#### Functions
```
sin      - sine
cos      - cosine
tan      - tangent
asin     - arcsine
acos     - arccosine
atan     - arctangent
sqrt / √ - square root
abs      - absolute value
exp      - exponentiation (e to power of)
ln       - natural logarithm (e as base)
log      - base 10 logarithm
```

#### Constants
```
pi / π  - the number pi
e       - Euler's number
phi / ϕ - the golden ratio
```

#### Variables
Calcr also supports defining your own variables as follows:
```
x = 2 + 4 * sin(0.5*pi)
```
However, it should be noted that case is ignored.

#### Exiting
In order to exit calcr, press escape, or type `quit`.

Building
--------
The easiest way to build the project is to use Cargo. Navigate to the project
directory and run:
```
cargo build --release
```
this will place the compiled program in the `./target` directory, from where
you can copy it to whereever.

Calcr - a small commandline calculator
======================================
This is a small commanline calculator written in Rust.

As of now the program supports the following operators and functions:
```
+    - plus
-    - minus or negation
*    - muliplication
/    - division
^    - powers
!    - factorial (only works on positive integers)
sin  - sine
cos  - cosine
tan  - tangent
asin - arcsine
acos - arccosine
atan - arctangent
sqrt - square root
abs  - absolute value
exp  - exponentiation (e to power of)
ln   - natural logarithm (e as base)
log  - base 10 logarithm
```

Furthermore the following constants are predefined
```
pi  - the number pi
e   - Euler's number
phi - the golden ratio
```

Building
--------
The easiest way to build the project is to use Cargo. Navigate to the project
directory and run:
```
cargo build --release
```
this will place the compiled program in the `./target` directory, from where
you can copy it to whereever.

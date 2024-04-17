## Rarc

'Arc-polygons in rust'

### Introduction
Some time ago my brother gave me an algorithmic problem having to do with a
type of 2D shape which we call an 'arc-polygon', which is
like a regular 2D polygon but generalized to include arcs (part of a circle)
besides regular line segments for connecting its vertices.

Given such an arc-polygon $P$ and some $t \geq 0$ consider the set of points

$`Q = \left\{ x \in \mathbb{R}^2 \;|\; d(x, P) = t \right\}
\quad \left( \text{where} \;
d(x, P) = \inf_{y \in P} |x - y| \right)`$

We conjecture that $Q$ can be described as a finite set of new arc-polygons.

#### Hence the problem:

Implement an algorithm to find for each arc-polygon $P$
and offset $t \geq 0$ this set of arc-polygons
$`\{P_1, P_2, \dots, P_n\}`$ so that $`\bigcup_i P_i = Q`$.

### Status

Work-in-progress.

### Setup

Requirements:
- [rustup](https://www.rust-lang.org/tools/install)

Run:
- `cargo run`
- `cargo run --example [name]` (checkout `./examples` folder)

### Preview

![App example](/assets/ss1.png)
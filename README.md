# plotters-offscreen-canvas

Implemenets the [Plotters Backend API](https://docs.rs/plotters-backend/latest/plotters_backend/) crate

[plotters-canvas](https://github.com/plotters-rs/plotters-canvas) cannot be run inside the web worker context.
I write this library to draw directly on the Offscreen Canvas.


# Testing

```
wasm-pack test --chrome
wasm-pack test --chrome --headless
```

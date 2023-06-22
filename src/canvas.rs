use plotters_backend::{DrawingBackend, DrawingErrorKind, BackendStyle, BackendColor};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{OffscreenCanvas, OffscreenCanvasRenderingContext2d};

pub struct OffscreenCanvasBackend {
    canvas: OffscreenCanvas,
    context: OffscreenCanvasRenderingContext2d,
}

pub struct CanvasError(String);

impl std::fmt::Display for CanvasError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       return write!(fmt, "Canvas Error: {}", self.0); 
    }
}

impl std::fmt::Debug for CanvasError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(fmt, "CanvasError({})", self.0);
    }
}

impl std::error::Error for CanvasError {}

impl OffscreenCanvasBackend {
    fn init_backend(canvas: OffscreenCanvas) -> Option<Self> {
        let context: OffscreenCanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        Some(OffscreenCanvasBackend{ canvas, context })
    }

    /// Create a new drawing backend backed with an ofscreen canvas object
    ///  - Return either thte drawing backend, or non in error case
    pub fn new(canvas: OffscreenCanvas) -> Option<Self> {
        Self::init_backend(canvas)
    }

    // pub fn with_offscreen_canvas_object(canvas: OffscreenCanvas) -> Option<Self> {
    //     Self::init_backend(canvas)
    // }

    fn set_line_style(&mut self, style: &impl BackendStyle) {
        self.context
            .set_stroke_style(&make_canvas_color(style.color()));
        self.context.set_line_width(style.stroke_width() as f64);
    }
}

fn make_canvas_color(color: BackendColor) -> JsValue {
    let (r, g, b) = color.rgb;
    let a = color.alpha;
    format!("rgba({},{},{},{}", r, g, b, a).into()
}

impl DrawingBackend for OffscreenCanvasBackend {
    type ErrorType = CanvasError;

    fn ensure_prepared(
        &mut self
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    } 

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn get_size(&self) -> (u32, u32) {
        (self.canvas.width(), self.canvas.height())
    }

    fn draw_pixel(
        &mut self,
        point: plotters_backend::BackendCoord,
        style: plotters_backend::BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }

        self.context.set_fill_style(&make_canvas_color(style.color()));
        self.context.fill_rect(f64::from(point.0), f64::from(point.1), 1.0, 1.0);
        
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
            &mut self,
            from: plotters_backend::BackendCoord,
            to: plotters_backend::BackendCoord,
            style: &S,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            if style.color().alpha == 0.0 {
                return Ok(());
            }

            self.set_line_style(style);
            self.context.begin_path();
            self.context.move_to(f64::from(from.0), f64::from(from.1));
            self.context.line_to(f64::from(to.0), f64::from(to.1));
            self.context.stroke();
            Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use plotters::prelude::*;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use wasm_bindgen_test::*;


    wasm_bindgen_test_configure!(run_in_browser);


    fn create_canvas(width: u32, height: u32) -> OffscreenCanvas {
        let canvas = OffscreenCanvas::new(width, height).unwrap();
        canvas
    }

    #[wasm_bindgen_test]
    fn test_draw_pixel_alphas() {
        let (width, height) = (100_u32, 100_u32);
        let canvas = create_canvas(width, height);
        let backend = OffscreenCanvasBackend::new(canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        for i in -20..20 {
            let alpha = i as f64 * 0.1;
            root.draw_pixel((50 + i, 50 + i), &BLACK.mix(alpha)).unwrap();
        }
    }
}
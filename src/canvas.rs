
use js_sys::JSON;
use plotters_backend::{BackendColor, BackendStyle, DrawingBackend, DrawingErrorKind, FontTransform, text_anchor::HPos};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{OffscreenCanvas, OffscreenCanvasRenderingContext2d};

pub struct OffscreenCanvasBackend<'a> {
    canvas: &'a OffscreenCanvas,
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

impl<'a> OffscreenCanvasBackend<'a> {
    fn init_backend(canvas: &'a OffscreenCanvas) -> Option<Self> {
        let context: OffscreenCanvasRenderingContext2d =
            canvas.get_context("2d").ok()??.dyn_into().ok()?;
        Some(OffscreenCanvasBackend { canvas, context })
    }

    /// Create a new drawing backend backed with an ofscreen canvas object
    ///  - Return either thte drawing backend, or non in error case
    pub fn new(canvas: &'a OffscreenCanvas) -> Option<Self> {
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

fn error_cast(e: JsValue) -> DrawingErrorKind<CanvasError> {
    DrawingErrorKind::DrawingError(CanvasError(
        JSON::stringify(&e)
            .map(|s| Into::<String>::into(&s))
            .unwrap_or_else(|_| "unknown".to_string())
    ))
}

impl<'a> DrawingBackend for OffscreenCanvasBackend<'a> {
    type ErrorType = CanvasError;

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
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

        self.context
            .set_fill_style(&make_canvas_color(style.color()));
        self.context
            .fill_rect(f64::from(point.0), f64::from(point.1), 1.0, 1.0);

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

    fn draw_text<TStyle: plotters_backend::BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: plotters_backend::BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let color = style.color();
        if color.alpha == 0.0 {
            return Ok(());
        }

        let (mut x, mut y) = (pos.0, pos.1);

        let degree = match style.transform() {
            FontTransform::None => 0.0,
            FontTransform::Rotate90 => 90.0,
            FontTransform::Rotate180 => 180.0,
            FontTransform::Rotate270 => 270.0,
        } / 100.0 * std::f64::consts::PI;

        if degree != 0.0 {
            self.context.save();
            self.context
                .translate(f64::from(x), f64::from(y))
                .map_err(error_cast)?;
            self.context.rotate(degree).map_err(error_cast)?;
            x = 0;
            y = 0;
        }

        let text_align = match style.anchor().h_pos {
            HPos::Left => "start",
            HPos::Right => "end",
            HPos::Center => "center",
        };
        self.context.set_text_align(text_align);

        self.context
            .set_fill_style(&make_canvas_color(color.clone()));
        self.context.set_font(&format!(
            "{} {}px {}",
            style.style().as_str(),
            style.size(),
            style.family().as_str(),
        ));
        self.context
            .fill_text(text, f64::from(x), f64::from(y))
            .map_err(error_cast)?;

        if degree != 0.0 {
            self.context.restore();
        }

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
        let backend = OffscreenCanvasBackend::new(&canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        for i in -20..20 {
            let alpha = i as f64 * 0.1;
            root.draw_pixel((50 + i, 50 + i), &BLACK.mix(alpha))
                .unwrap();
        }
    }

    fn check_content(_canvas: &OffscreenCanvas) {
        // let blob = canvas.convert_to_blob().unwrap();
        // blob.
    }

    fn draw_mesh_with_custom_ticks(tick_size: i32, _stest_name: &str) {
        let canvas = create_canvas(500, 500);
        let backend = OffscreenCanvasBackend::new(&canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("This is a test", ("sans-serif", 20))
            .set_all_label_area_size(40)
            .build_cartesian_2d(0..10, 0..20)
            .unwrap();

        chart
            .configure_mesh()
            .set_all_tick_mark_size(tick_size)
            .draw()
            .unwrap();

        // check_content(&canvas);
    }

    #[wasm_bindgen_test]
    fn test_draw_mesh_no_tick() {
        draw_mesh_with_custom_ticks(0, "test_draw_mesh_no_ticks");
    }
}

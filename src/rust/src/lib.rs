use extendr_api::{
    graphics::{ClippingStrategy, DevDesc, DeviceDescriptor, DeviceDriver, R_GE_gcontext},
    prelude::*,
};
use itertools::Itertools;
use std::fs::File;
use std::io::{BufWriter, Write};

#[allow(dead_code)]
struct SVGDevice {
    // Not sure if BufWriter really contributes a better performance in actual.
    svg_file: BufWriter<File>,
}

impl DeviceDriver for SVGDevice {
    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;

    fn close(&mut self, _: DevDesc) {
        if let Err(e) = writeln!(self.svg_file, "</svg>") {
            rprintln!("Cannot write the end tag: {e}");
        }

        if let Err(e) = self.svg_file.flush() {
            rprintln!("Cannot flush: {e}");
        }
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let fill = i32_to_csscolor(gc.fill);
        let (cx, cy) = center;

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<circle cx="{cx:.3}" cy="{cy:.3}" r="{r:.3}" stroke="{stroke}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a circle: {e}");
        }
    }

    fn polyline<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);

        let points = coords
            .into_iter()
            .map(|(x, y)| format!("{x:.3},{y:.3}"))
            .collect::<Vec<String>>()
            .join(" ");

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<polyline points="{points}"  stroke="{stroke}" fill="none" />"##
        ) {
            rprintln!("Cannot write a polyline: {e}");
        }
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let (x1, y1) = from;
        let (x2, y2) = to;

        let stroke = i32_to_csscolor(gc.col);

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<line x1="{x1:.3}" y1="{y1:.3}" x2="{x2:.3}" y2="{y2:.3}" stroke="{stroke}" fill="none" />"##
        ) {
            rprintln!("Cannot write a line: {e}");
        }
    }

    fn text(
        &mut self,
        pos: (f64, f64),
        text: &str,
        angle: f64,
        hadj: f64,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        // R!("browser()").unwrap();

        let fill = i32_to_csscolor(gc.col);

        let (x, y) = pos;
        let rot = -angle;

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<text x="{x:.3}" y="{y:.3}" transform="rotate({rot:.3}, {x:.3}, {y:.3})" fill="{fill}">{text}</text>"##
        ) {
            rprintln!("Cannot write a text: {e}");
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let fill = i32_to_csscolor(gc.fill);

        let (x0, y0) = from;
        let (x1, y1) = to;

        let x = x0.min(x1);
        let y = y0.min(y1);
        let width = (x0 - x1).abs();
        let height = (y0 - y1).abs();

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<rect x="{x:.3}" y="{y:.3}" width="{width:.3}" height="{height:.3}" stroke="{stroke}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a rect: {e}");
        }
    }

    fn polygon<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let fill = i32_to_csscolor(gc.fill);

        let points = coords
            .into_iter()
            .map(|(x, y)| format!("{x:.3},{y:.3}"))
            .collect::<Vec<String>>()
            .join(" ");

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<polygon points="{points}"  stroke="{stroke}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a polygon: {e}");
        }
    }

    fn path<T: IntoIterator<Item = impl IntoIterator<Item = (f64, f64)>>>(
        &mut self,
        coords: T,
        winding: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let stroke_width = gc.lwd / 96.0;
        let fill = i32_to_csscolor(gc.fill);
        let fill_rule = if winding { "nonzero" } else { "evenodd" };

        let points = coords
            .into_iter()
            .map(|subpath| {
                let xy = subpath
                    .into_iter()
                    .map(|(x, y)| format!("L {x:.3} {y:.3}"))
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("M {xy} Z")
            })
            .collect::<Vec<String>>()
            .join("\n");

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<path d="{points}" stroke="{stroke}" stroke-width="{stroke_width}" fill-rule="{fill_rule}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a path: {e}");
        }
    }
}

// R's color representation is in the order of Alpha, Blue, Green, and Red. So,
// we need to flip the order. Besides, it seems SVG spec doesn't accept
// "#RRGGBBAA" format.
//
// https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796
fn i32_to_csscolor(x: i32) -> String {
    let x: u32 = unsafe { std::mem::transmute(x) };

    let r = x & 255;
    let g = (x >> 8) & 255;
    let b = (x >> 16) & 255;
    let a = (x >> 24) & 255;

    format!("rgba({r}, {g}, {b}, {a})")
}

/// A graphic device that does nothing
///
/// @param svg_file A path to output SVG file.
/// @param width  Device width in inch.
/// @param height Device width in inch.
/// @export
#[extendr]
fn extendr_svg(svg_file: &str, width: i32, height: i32) {
    let svg_file = File::create(svg_file).expect("Cannot create the SVG file");
    let mut svg_file = BufWriter::new(svg_file);

    // Typically, 72 points per inch
    let width_pt = width * 72;
    let height_pt = height * 72;

    // write the headers
    writeln!(
        svg_file,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width_pt} {height_pt}" width="{width_pt}" height="{height_pt}">"#
    )
    .expect("Cannot write the start tag");

    let device_driver = SVGDevice { svg_file };

    let device_descriptor =
        // In SVG's coordinate y=0 is at top, so, we need to flip it by setting bottom > top.
        DeviceDescriptor::new().device_size(0.0, width_pt as _, height_pt as _, 0.0);

    device_driver.create_device::<SVGDevice>(device_descriptor, "extendr_svg");
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod extendrSVGdevice;
    fn extendr_svg;
}

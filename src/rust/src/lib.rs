use extendr_api::{
    graphics::{DevDesc, DeviceDescriptor, DeviceDriver, R_GE_gcontext},
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
    fn close(&mut self, _: DevDesc) {
        if let Err(e) = writeln!(self.svg_file, "</svg>") {
            println!("Cannot write the end tag");
        }

        if let Err(e) = self.svg_file.flush() {
            rprintln!("Cannot flush: {e}");
        }
    }

    fn circle(&mut self, x: f64, y: f64, r: f64, gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let fill = i32_to_csscolor(gc.fill);

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<circle cx="{x}" cy="{y}" r="{r}" stroke="{stroke}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a circle: {e}");
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);

        let points = itertools::zip(x, y)
            .map(|(&x, &y)| {
                let x = x;
                let y = y;
                format!("{x},{y}")
            })
            .join(" ");

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<polyline points="{points}"  stroke="{stroke}" fill="none" />"##
        ) {
            rprintln!("Cannot write a polyline: {e}");
        }
    }

    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" fill="none" />"##
        ) {
            rprintln!("Cannot write a line: {e}");
        }
    }

    fn text(
        &mut self,
        x: f64,
        y: f64,
        str: &str,
        rot: f64,
        _hadj: f64,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        // R!("browser()").unwrap();

        let fill = i32_to_csscolor(gc.col);

        let rot = -rot;

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<text x="{x}" y="{y}" transform="rotate({rot}, {x}, {y})" fill="{fill}">{str}</text>"##
        ) {
            rprintln!("Cannot write a text: {e}");
        }
    }

    fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let fill = i32_to_csscolor(gc.fill);

        let x = x0.min(x1);
        let y = y0.min(y1);
        let width = (x0 - x1).abs();
        let height = (y0 - y1).abs();

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<rect x="{x}" y="{y}" width="{width}" height="{height}" stroke="{stroke}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a rect: {e}");
        }
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let fill = i32_to_csscolor(gc.fill);

        let points = itertools::zip(x, y)
            .map(|(&x, &y)| {
                let x = x;
                let y = y;
                format!("{x},{y}")
            })
            .join(" ");

        if let Err(e) = writeln!(
            self.svg_file,
            r##"<polygon points="{points}"  stroke="{stroke}" fill="{fill}" />"##
        ) {
            rprintln!("Cannot write a polygon: {e}");
        }
    }

    fn path(
        &mut self,
        x: &[f64],
        y: &[f64],
        nper: &[i32],
        winding: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        // R!("browser()").unwrap();

        let stroke = i32_to_csscolor(gc.col);
        let stroke_width = gc.lwd / 96.0;
        let fill = i32_to_csscolor(gc.fill);
        let fill_rule = if winding { "nonzero" } else { "evenodd" };

        let mut i = itertools::zip(x, y);
        let mut points = String::new();

        for &n in nper {
            let mut i_part = i.by_ref().take(n as _);
            let (&first_point_x, &first_point_y) = i_part.next().expect("The length doesn't match");
            let first_point = format!("M {first_point_x} {first_point_y}");

            let other_points = i_part
                .map(|(&x, &y)| {
                    let x = x;
                    let y = y;
                    format!("L {x} {y}")
                })
                .join(" ");

            points.push_str(format!("{first_point} {other_points} Z").as_str());
        }

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

# Generated by extendr: Do not edit by hand
#
# This file was created with the following call:
#   .Call("wrap__make_extendrSVGdevice_wrappers", use_symbols = TRUE, package_name = "extendrSVGdevice")

#' @docType package
#' @usage NULL
#' @useDynLib extendrSVGdevice, .registration = TRUE
NULL

#' A graphic device that does nothing
#'
#' @param svg_file A path to output SVG file.
#' @param width  Device width in inch.
#' @param height Device width in inch.
#' @export
extendr_svg <- function(svg_file, width, height) invisible(.Call(wrap__extendr_svg, svg_file, width, height))


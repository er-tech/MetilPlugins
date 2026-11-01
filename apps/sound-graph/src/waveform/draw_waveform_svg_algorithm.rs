use std::fmt::Write;

use super::compute_columns_algorithm::{peak_of, Column};
use super::escape_xml_algorithm::escape_xml;
use super::region_contract::Region;
use super::time_ticks_algorithm::{format_time, nice_step_ms, ticks_ms};

/// How the picture looks.
#[derive(Debug, Clone)]
pub struct SvgOptions {
    pub width: usize,
    pub height: usize,
    pub color: String,
    /// A color, or `none` for a see-through picture.
    pub background: String,
    pub show_time: bool,
    pub title: Option<String>,
    /// Stretch the waves so the loudest point touches the edge; otherwise full scale is the edge
    /// and a quiet sound looks quiet.
    pub normalize: bool,
}

/// The height kept under the waves for the time axis.
const AXIS_HEIGHT: usize = 20;

/// Draws the waves as an SVG picture: one column of the picture for each of `columns`, the waves
/// mirrored around a middle line, the `regions` marked behind them and a time axis under them.
/// Everything that comes from outside is escaped.
// The pieces of the SVG are written a line at a time, some with two lines in one string.
#[allow(clippy::write_with_newline)]
pub fn draw_waveform_svg(
    columns: &[Column],
    duration_ms: u64,
    regions: &[Region],
    options: &SvgOptions,
) -> String {
    let width = options.width;
    let axis = if options.show_time { AXIS_HEIGHT } else { 0 };
    let plot_height = (options.height - axis.min(options.height / 2)) as f32;
    let middle = plot_height / 2.0;
    let half = middle - 1.0;
    let peak = peak_of(columns);
    let scale = if options.normalize && peak > 0.0 {
        half / peak
    } else {
        half
    };
    let clamp = |y: f32| y.clamp(0.0, plot_height);

    let title = options.title.as_deref().unwrap_or("Sound waves");
    let description = format!(
        "The waves of a sound that lasts {}, drawn in {width} columns.",
        format_time(duration_ms, 1_000)
    );
    let mut svg = String::with_capacity(columns.len() * 24 + 1_024);
    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-labelledby=\"title desc\">\n",
        height = options.height
    );
    let _ = write!(
        svg,
        "<title id=\"title\">{}</title>\n<desc id=\"desc\">{}</desc>\n",
        escape_xml(title),
        escape_xml(&description)
    );
    if options.background != "none" {
        let _ = write!(
            svg,
            "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>\n",
            escape_xml(&options.background)
        );
    }

    // The regions go behind the waves.
    for (index, region) in regions.iter().enumerate() {
        if duration_ms == 0 {
            break;
        }
        let x = region.start_ms.min(duration_ms) as f32 / duration_ms as f32 * width as f32;
        let end = region.end_ms.min(duration_ms) as f32 / duration_ms as f32 * width as f32;
        let region_width = (end - x).max(1.0);
        let color = region.color.as_deref().unwrap_or("#f6ad55");
        let _ = write!(
            svg,
            "<rect x=\"{x:.1}\" y=\"0\" width=\"{region_width:.1}\" height=\"{plot_height:.1}\" fill=\"{}\" fill-opacity=\"0.22\"/>\n",
            escape_xml(color)
        );
        if let Some(label) = &region.label {
            let fits = (region_width / 6.0) as usize;
            if fits >= 2 {
                let shown = if label.chars().count() > fits {
                    let cut: String = label.chars().take(fits - 1).collect();
                    format!("{cut}…")
                } else {
                    label.clone()
                };
                // Neighbouring labels take turns at two heights so they do not sit on each other.
                let y = 12 + (index % 2) * 12;
                let _ = write!(
                    svg,
                    "<text x=\"{:.1}\" y=\"{y}\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#2d3748\">{}</text>\n",
                    x + 3.0,
                    escape_xml(&shown)
                );
            }
        }
    }

    let _ = write!(
        svg,
        "<line x1=\"0\" y1=\"{middle:.1}\" x2=\"{width}\" y2=\"{middle:.1}\" stroke=\"#a0aec0\" stroke-width=\"1\"/>\n"
    );

    // The waves: along the top of the columns, then back along the bottom.
    let mut path = String::with_capacity(columns.len() * 20);
    for (index, column) in columns.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        let _ = write!(
            path,
            "{command}{:.1},{:.1} ",
            index as f32 + 0.5,
            clamp(middle - column.max * scale)
        );
    }
    for (index, column) in columns.iter().enumerate().rev() {
        let _ = write!(
            path,
            "L{:.1},{:.1} ",
            index as f32 + 0.5,
            clamp(middle - column.min * scale)
        );
    }
    let _ = write!(
        svg,
        "<path d=\"{path}Z\" fill=\"{color}\" stroke=\"{color}\" stroke-width=\"1\" stroke-linejoin=\"round\"/>\n",
        color = escape_xml(&options.color)
    );

    if options.show_time && duration_ms > 0 {
        let _ = write!(
            svg,
            "<line x1=\"0\" y1=\"{plot_height:.1}\" x2=\"{width}\" y2=\"{plot_height:.1}\" stroke=\"#718096\" stroke-width=\"1\"/>\n"
        );
        let step = nice_step_ms(duration_ms, (width / 100).max(2));
        for time in ticks_ms(duration_ms, (width / 100).max(2)) {
            let x = time as f32 / duration_ms as f32 * width as f32;
            let anchor = if x < 18.0 {
                "start"
            } else if x > width as f32 - 18.0 {
                "end"
            } else {
                "middle"
            };
            let _ = write!(
                svg,
                "<line x1=\"{x:.1}\" y1=\"{plot_height:.1}\" x2=\"{x:.1}\" y2=\"{:.1}\" stroke=\"#718096\" stroke-width=\"1\"/>\n<text x=\"{x:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#4a5568\">{}</text>\n",
                plot_height + 4.0,
                plot_height + 15.0,
                format_time(time, step)
            );
        }
    }
    svg.push_str("</svg>\n");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> SvgOptions {
        SvgOptions {
            width: 100,
            height: 120,
            color: "#2b6cb0".into(),
            background: "#ffffff".into(),
            show_time: true,
            title: None,
            normalize: false,
        }
    }

    /// Loud for the first 20 columns, silent for the rest.
    fn loud_then_silent() -> Vec<Column> {
        (0..100)
            .map(|c| {
                if c < 20 {
                    Column {
                        min: -1.0,
                        max: 1.0,
                    }
                } else {
                    Column { min: 0.0, max: 0.0 }
                }
            })
            .collect()
    }

    fn points(svg: &str) -> Vec<(f32, f32)> {
        let path = svg
            .split("<path d=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        path.trim_end_matches('Z')
            .split_whitespace()
            .map(|point| {
                let (x, y) = point[1..].split_once(',').unwrap();
                (x.parse().unwrap(), y.parse().unwrap())
            })
            .collect()
    }

    #[test]
    fn draws_a_picture_of_the_size_asked_with_a_title_and_the_waves_as_one_path() {
        let svg = draw_waveform_svg(&loud_then_silent(), 10_000, &[], &options());
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"120\" viewBox=\"0 0 100 120\""), "{svg}");
        assert!(svg.contains("<title id=\"title\">Sound waves</title>"));
        assert!(svg.contains("fill=\"#ffffff\"") && svg.contains("fill=\"#2b6cb0\""));
        assert_eq!(svg.matches("<path ").count(), 1);
        assert!(svg.trim_end().ends_with("</svg>"));
        // 100 columns along the top and 100 back along the bottom.
        assert_eq!(points(&svg).len(), 200);
    }

    #[test]
    fn loud_columns_reach_the_edge_and_silent_ones_sit_on_the_middle_line() {
        let svg = draw_waveform_svg(&loud_then_silent(), 10_000, &[], &options());
        let points = points(&svg);
        // The plot is 100 high (120 less the axis), so the middle is 50 and full scale 49 away.
        assert!((points[0].1 - 1.0).abs() < 0.11, "{:?}", points[0]);
        assert!((points[199].1 - 99.0).abs() < 0.11, "{:?}", points[199]);
        assert!((points[50].1 - 50.0).abs() < 0.11, "{:?}", points[50]);
    }

    #[test]
    fn normalising_stretches_a_quiet_sound_to_the_edge_and_otherwise_it_stays_small() {
        let quiet: Vec<Column> = (0..100)
            .map(|_| Column {
                min: -0.1,
                max: 0.1,
            })
            .collect();
        let plain = points(&draw_waveform_svg(&quiet, 5_000, &[], &options()));
        assert!((plain[0].1 - 45.1).abs() < 0.2, "{:?}", plain[0]);
        let stretched = points(&draw_waveform_svg(
            &quiet,
            5_000,
            &[],
            &SvgOptions {
                normalize: true,
                ..options()
            },
        ));
        assert!((stretched[0].1 - 1.0).abs() < 0.2, "{:?}", stretched[0]);
    }

    #[test]
    fn regions_are_marked_behind_the_waves_with_their_labels_cut_to_fit_and_escaped() {
        let regions = [
            Region {
                start_ms: 0,
                end_ms: 5_000,
                label: Some("A <very> long sentence about cats".into()),
                color: None,
            },
            Region {
                start_ms: 6_000,
                end_ms: 6_100,
                label: Some("tiny".into()),
                color: Some("red\" onload=\"x".into()),
            },
        ];
        let svg = draw_waveform_svg(&loud_then_silent(), 10_000, &regions, &options());
        // The first covers half the picture (50 px, room for 8 characters: 7 and an ellipsis).
        assert!(
            svg.contains("<rect x=\"0.0\" y=\"0\" width=\"50.0\""),
            "{svg}"
        );
        assert!(svg.contains("A &lt;very…</text>"), "{svg}");
        // The second is 1 px wide: no room for a label, and its color cannot break out of the attribute.
        assert!(!svg.contains(">tiny<"));
        assert!(svg.contains("fill=\"red&quot; onload=&quot;x\""), "{svg}");
        assert!(!svg.contains("onload=\"x\""));
        // The regions are before the path.
        assert!(svg.find("<rect x=").unwrap() < svg.find("<path ").unwrap());
    }

    #[test]
    fn the_axis_has_round_times_and_can_be_left_out_and_the_background_can_be_see_through() {
        let svg = draw_waveform_svg(&loud_then_silent(), 10_000, &[], &options());
        assert!(
            svg.contains(">0:00</text>") && svg.contains(">0:10</text>"),
            "{svg}"
        );
        let bare = draw_waveform_svg(
            &loud_then_silent(),
            10_000,
            &[],
            &SvgOptions {
                show_time: false,
                background: "none".into(),
                title: Some("Take \"1\"".into()),
                ..options()
            },
        );
        assert!(!bare.contains("<text") && !bare.contains("<rect"), "{bare}");
        assert!(bare.contains("<title id=\"title\">Take &quot;1&quot;</title>"));
    }
}

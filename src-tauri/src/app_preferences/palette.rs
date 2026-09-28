//! The same product tokens consumed by React's `collectPalette` bridge.
//!
//! Keep colors in the shared CSS asset. In particular, the CSS body color
//! differs from the native viewport's historical default.

use std::{collections::BTreeMap, sync::OnceLock};

use super::ResolvedTheme;
use crate::native_viewport::ViewportPalette;

const CSS: &str = include_str!("../../../src/index.css");
static LIGHT: OnceLock<Result<ViewportPalette, String>> = OnceLock::new();
static DARK: OnceLock<Result<ViewportPalette, String>> = OnceLock::new();

/// Parse each embedded theme once; malformed or missing required tokens are
/// surfaced to the caller instead of introducing a second fallback palette.
pub(crate) fn viewport_palette(
    theme: ResolvedTheme,
) -> Result<&'static ViewportPalette, &'static str> {
    let cache = match theme {
        ResolvedTheme::Light => &LIGHT,
        ResolvedTheme::Dark => &DARK,
    };
    cache
        .get_or_init(|| parse_palette(CSS, theme))
        .as_ref()
        .map_err(String::as_str)
}

fn parse_palette(css: &str, theme: ResolvedTheme) -> Result<ViewportPalette, String> {
    let css = without_comments(css)?;
    let tokens = theme_tokens(&css, theme)?;
    let rgb = |token| token_rgb(&tokens, token);
    Ok(ViewportPalette {
        background: rgb("--viewport")?,
        panel: rgb("--panel")?,
        header: rgb("--header")?,
        ui_edge: rgb("--edge")?,
        ink: rgb("--ink")?,
        mute: rgb("--mute")?,
        accent: rgb("--accent")?,
        grid_fine: rgb("--cad-ground-fine")?,
        grid_major: rgb("--cad-ground-major")?,
        body: rgb("--cad-body")?,
        body_selected: rgb("--cad-body-selected")?,
        body_tool: rgb("--cad-body-tool")?,
        body_selected_edge: rgb("--cad-pick-selected")?,
        face_hover: rgb("--cad-face-hover")?,
        face_selected: rgb("--cad-face-selected")?,
        edge: rgb("--cad-edge")?,
        edge_hover: rgb("--cad-pick-hover")?,
        edge_selected: rgb("--cad-pick-selected")?,
        pick_halo: rgb("--cad-pick-halo")?,
        origin_plane_xy: rgb("--cad-origin-plane-xy")?,
        origin_plane_xz: rgb("--cad-origin-plane-xz")?,
        origin_plane_yz: rgb("--cad-origin-plane-yz")?,
        active_sketch: rgb("--cad-pick-normal")?,
        defined_sketch: rgb("--cad-defined")?,
        hover: rgb("--cad-pick-hover")?,
        selection: rgb("--cad-pick-selected")?,
        constraint_related: rgb("--cad-constraint-related")?,
        finished_sketch: rgb("--cad-pick-normal")?,
        finished_sketch_point: rgb("--cad-pick-normal")?,
        finished_sketch_point_outline: rgb("--cad-finished-point-outline")?,
        preview: rgb("--cad-preview")?,
        dimension: rgb("--dimgreen")?,
        projected: rgb("--cad-projected")?,
    })
}

// This is deliberately a parser for the flat product-token blocks, not a CSS
// engine. New color syntaxes must be supported explicitly by both consumers.
fn without_comments(mut css: &str) -> Result<String, String> {
    let mut result = String::with_capacity(css.len());
    while let Some((before, rest)) = css.split_once("/*") {
        result.push_str(before);
        result.push(' ');
        let (_, after) = rest
            .split_once("*/")
            .ok_or("Shared appearance CSS has an unterminated comment")?;
        css = after;
    }
    result.push_str(css);
    Ok(result)
}

fn theme_tokens(css: &str, theme: ResolvedTheme) -> Result<BTreeMap<&str, &str>, String> {
    // :root provides dark defaults; light inherits any token it does not
    // override, exactly as the document element does in the React shell.
    let mut tokens = block_tokens(css, ":root[data-theme='dark']")?;
    if matches!(theme, ResolvedTheme::Light) {
        tokens.extend(block_tokens(css, ":root[data-theme='light']")?);
    }
    Ok(tokens)
}

fn block_tokens<'a>(css: &'a str, selector: &str) -> Result<BTreeMap<&'a str, &'a str>, String> {
    let (_, after) = css
        .split_once(selector)
        .ok_or_else(|| format!("Shared appearance CSS is missing {selector}"))?;
    let after = after
        .trim_start()
        .strip_prefix('{')
        .ok_or_else(|| format!("Shared appearance CSS has an invalid {selector} block"))?;
    let (block, _) = after
        .split_once('}')
        .ok_or_else(|| format!("Shared appearance CSS has an unclosed {selector} block"))?;
    if block.contains('{') {
        return Err(format!(
            "Shared appearance CSS has nested rules in {selector}"
        ));
    }
    let mut tokens = BTreeMap::new();
    for declaration in block.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (name, value) = declaration.split_once(':').ok_or_else(|| {
            format!("Shared appearance CSS has an invalid declaration: {declaration}")
        })?;
        if name.trim().starts_with("--") {
            tokens.insert(name.trim(), value.trim());
        }
    }
    Ok(tokens)
}

fn token_rgb(tokens: &BTreeMap<&str, &str>, name: &str) -> Result<[f32; 3], String> {
    let value = tokens
        .get(name)
        .ok_or_else(|| format!("Shared appearance CSS is missing color token {name}"))?;
    let hex = value
        .strip_prefix('#')
        .filter(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| {
            format!("Shared appearance CSS color {name} must be #RRGGBB, got {value}")
        })?;
    let number = u32::from_str_radix(hex, 16)
        .map_err(|error| format!("Shared appearance CSS color {name}: {error}"))?;
    Ok([
        ((number >> 16) & 255) as f32 / 255.0,
        ((number >> 8) & 255) as f32 / 255.0,
        (number & 255) as f32 / 255.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_colors_and_cache_do_not_use_the_legacy_body_default() {
        let dark = viewport_palette(ResolvedTheme::Dark).unwrap();
        let light = viewport_palette(ResolvedTheme::Light).unwrap();
        assert_eq!(dark.body, [139.0 / 255.0, 155.0 / 255.0, 172.0 / 255.0]);
        assert_eq!(light.body, [159.0 / 255.0, 179.0 / 255.0, 197.0 / 255.0]);
        assert_eq!(dark.accent, [116.0 / 255.0, 99.0 / 255.0, 216.0 / 255.0]);
        assert_eq!(light.ink, [37.0 / 255.0, 43.0 / 255.0, 50.0 / 255.0]);
        assert_ne!(dark.body, ViewportPalette::default().body);
        assert!(std::ptr::eq(
            dark,
            viewport_palette(ResolvedTheme::Dark).unwrap()
        ));
        assert!(std::ptr::eq(
            light,
            viewport_palette(ResolvedTheme::Light).unwrap()
        ));
    }

    #[test]
    fn missing_or_unsupported_tokens_fail_explicitly_and_light_inherits_root() {
        let missing = CSS.replace("--cad-body:", "--unused-body:");
        assert!(parse_palette(&missing, ResolvedTheme::Light)
            .unwrap_err()
            .contains("--cad-body"));
        let malformed = CSS.replace("#8b9bac", "rgb(139, 155, 172)");
        assert!(parse_palette(&malformed, ResolvedTheme::Dark)
            .unwrap_err()
            .contains("#RRGGBB"));
        let inherited = CSS.replace("--cad-body: #9fb3c5;", "");
        assert_eq!(
            parse_palette(&inherited, ResolvedTheme::Light)
                .unwrap()
                .body,
            viewport_palette(ResolvedTheme::Dark).unwrap().body
        );
        assert!(parse_palette("/* unterminated", ResolvedTheme::Dark).is_err());
        assert!(parse_palette(
            ":root[data-theme='dark'] { --panel: #23262b;",
            ResolvedTheme::Dark
        )
        .is_err());
        assert!(
            parse_palette(":root[data-theme='dark'] {}", ResolvedTheme::Light)
                .unwrap_err()
                .contains("light")
        );
    }
}

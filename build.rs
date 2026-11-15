use std::env;
use std::fs;
use std::path::Path;

fn extract_view_box(svg: &str) -> (f32, f32) {
    if let Some(pos) = svg.find("viewBox=\"") {
        let rest = &svg[pos + 9..];
        if let Some(end) = rest.find('"') {
            let nums: Vec<f32> = rest[..end]
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            if nums.len() == 4 {
                return (nums[2], nums[3]);
            }
        }
    }
    (24.0, 24.0)
}

fn extract_body(stem: &str, svg: &str, w: f32, h: f32) -> String {
    let raw = if stem == "realme" {
        if let Some(pos) = svg.find("id=\"path30\"") {
            if let Some(start) = svg[..pos].rfind("<path") {
                if let Some(end) = svg.rfind("</svg>") {
                    svg[start..end].to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else if let Some(start) = svg.find("<g").or_else(|| svg.find("<path")) {
        if let Some(end) = svg.rfind("</svg>") {
            svg[start..end].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let size = w.max(h);
    if (size - h).abs() > 1.0 && !raw.is_empty() {
        let dy = (size - h) / 2.0;
        format!("<g transform=\"translate(0, {dy:.2})\">{raw}</g>")
    } else {
        raw
    }
}

fn main() {
    println!("cargo:rerun-if-changed=assets/brands");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("brands_gen.rs");

    let brands_dir = Path::new("assets/brands");
    let mut out = String::new();
    let mut match_arms = String::new();

    if let Ok(entries) = fs::read_dir(brands_dir) {
        let mut files: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("svg"))
            .collect();
        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let path = entry.path();
            let stem = path.file_stem().unwrap().to_string_lossy().to_lowercase();
            let const_name = format!("ICON_{}", stem.to_uppercase());
            let svg = fs::read_to_string(&path).unwrap_or_default();
            let (w, h) = extract_view_box(&svg);
            let view = w.max(h);
            let body = extract_body(&stem, &svg, w, h);
            let escaped = body.escape_default().to_string();

            out.push_str(&format!(
                "pub static {const_name}: rune::Icon = rune::Icon {{ name: \"{stem}\", view: {view:.2}, body: \"{escaped}\" }};\n"
            ));
            if stem != "android" {
                match_arms.push_str(&format!("        \"{stem}\" => &{const_name},\n"));
            }
        }
    }

    out.push_str("\npub fn icon_for_brand(brand: &str) -> &'static rune::Icon {\n");
    out.push_str("    let b = brand.trim().to_ascii_lowercase();\n");
    out.push_str("    match b.as_str() {\n");
    out.push_str(&match_arms);
    out.push_str("        \"redmi\" => &ICON_XIAOMI,\n");
    out.push_str("        \"pixel\" => &ICON_GOOGLE,\n");
    out.push_str("        _ => &ICON_ANDROID,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    fs::write(dest, out).unwrap();
}

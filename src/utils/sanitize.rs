use std::path::Path;

pub fn sanitize_name(name: String) -> String {
    let mut name = name.to_string();

    if name.starts_with(" ") {
        name = name[1..].to_string();
    }
    if name.ends_with(" ") {
        name = name[..name.len() - 1].to_string();
    }

    name = name
        .replace("/", "-")
        .replace("\\", "-")
        .replace(":", "-")
        .replace("*", "-")
        .replace("?", "-")
        .replace("\"", "-")
        .replace("<", "-")
        .replace(">", "-")
        .replace("|", "-");

    // remove url encodings
    name = html_escape::decode_html_entities(&name).to_string();

    // remove trailing dots
    name = name.strip_suffix(".").unwrap_or(&name).to_string();

    name
}

pub fn remove_extension(filename: &str) -> &str {
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename)
}

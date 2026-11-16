const LUMINANCE_THRESHOLD: f64 = 200.0;
const CONTRAST_THRESHOLD: f64 = 3.5;

/// Remove color declarations that keep an HTML email fixed in light mode.
/// Messages with their own dark-mode rules are left untouched.
pub(super) fn adapt(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut cursor = 0;
    while let Some(relative_start) = html[cursor..].find('<') {
        let start = cursor + relative_start;
        output.push_str(&html[cursor..start]);

        if html[start..].starts_with("<!--") {
            let end = html[start + 4..]
                .find("-->")
                .map(|offset| start + 4 + offset + 3)
                .unwrap_or(html.len());
            output.push_str(&html[start..end]);
            cursor = end;
            continue;
        }

        let Some(end) = tag_end(html, start) else {
            output.push_str(&html[start..]);
            return output;
        };
        let tag = &html[start..=end];
        let name = tag_name(tag);
        if name.is_empty() || tag.starts_with("</") || tag.starts_with("<!") || tag.starts_with("<?") {
            output.push_str(tag);
            cursor = end + 1;
            continue;
        }

        output.push_str(&adapt_tag(tag, &name));
        cursor = end + 1;

        if name == "style" {
            let lowered_tail = html[cursor..].to_ascii_lowercase();
            if let Some(close_offset) = lowered_tail.find("</style") {
                let close_start = cursor + close_offset;
                output.push_str(&adapt_stylesheet(&html[cursor..close_start]));
                if let Some(close_end) = tag_end(html, close_start) {
                    output.push_str(&html[close_start..=close_end]);
                    cursor = close_end + 1;
                } else {
                    output.push_str(&html[close_start..]);
                    return output;
                }
            }
        }
    }
    output.push_str(&html[cursor..]);
    output
}

fn tag_end(html: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, byte) in html.as_bytes().iter().enumerate().skip(start + 1) {
        match (*byte, quote) {
            (b'\'', None) => quote = Some(b'\''),
            (b'"', None) => quote = Some(b'"'),
            (b'\'', Some(b'\'')) | (b'"', Some(b'"')) => quote = None,
            (b'>', None) => return Some(offset),
            _ => {}
        }
    }
    None
}

fn tag_name(tag: &str) -> String {
    let Some(start) = tag.find('<').map(|index| index + 1) else {
        return String::new();
    };
    let start = if tag.as_bytes().get(start) == Some(&b'/') {
        start + 1
    } else {
        start
    };
    let end = tag[start..]
        .find(|character: char| character.is_ascii_whitespace() || matches!(character, '/' | '>'))
        .map(|offset| start + offset)
        .unwrap_or(tag.len());
    tag[start..end].to_ascii_lowercase()
}

fn adapt_tag(tag: &str, name: &str) -> String {
    let mut name_start = 1;
    if tag.as_bytes().get(name_start) == Some(&b'/') {
        name_start += 1;
    }
    while tag
        .as_bytes()
        .get(name_start)
        .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'/' | b'>'))
    {
        name_start += 1;
    }
    let mut output = tag[..name_start].to_owned();
    let mut cursor = name_start;

    while cursor < tag.len() {
        let whitespace_start = cursor;
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if cursor >= tag.len() || matches!(tag.as_bytes()[cursor], b'>' | b'/') {
            output.push_str(&tag[whitespace_start..]);
            break;
        }

        let attribute_start = cursor;
        while tag.as_bytes().get(cursor).is_some_and(|byte| {
            !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/')
        }) {
            cursor += 1;
        }
        if cursor == attribute_start {
            output.push_str(&tag[whitespace_start..]);
            break;
        }
        let attribute_name = tag[attribute_start..cursor].to_ascii_lowercase();
        let name_end = cursor;
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }

        let mut value_range = None;
        if tag.as_bytes().get(cursor) == Some(&b'=') {
            cursor += 1;
            while tag
                .as_bytes()
                .get(cursor)
                .is_some_and(u8::is_ascii_whitespace)
            {
                cursor += 1;
            }
            if matches!(tag.as_bytes().get(cursor), Some(b'\'' | b'"')) {
                let quote = tag.as_bytes()[cursor];
                cursor += 1;
                let value_start = cursor;
                while tag.as_bytes().get(cursor).is_some_and(|byte| *byte != quote) {
                    cursor += 1;
                }
                value_range = Some((value_start, cursor, quote));
                if tag.as_bytes().get(cursor).is_some() {
                    cursor += 1;
                }
            } else {
                let value_start = cursor;
                while tag.as_bytes().get(cursor).is_some_and(|byte| {
                    !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/')
                }) {
                    cursor += 1;
                }
                value_range = Some((value_start, cursor, 0));
            }
        }
        let attribute_end = cursor;

        let remove = if name == "body" {
            matches!(attribute_name.as_str(), "bgcolor" | "text" | "link" | "vlink")
        } else {
            matches!(attribute_name.as_str(), "bgcolor" | "color")
                || (name == "text" && attribute_name == "fill")
        };
        if remove {
            continue;
        }

        if attribute_name == "style"
            && let Some((value_start, value_end, quote)) = value_range {
                let value = &tag[value_start..value_end];
                let adapted = if name == "body" {
                    sanitize_body_style(value)
                } else {
                    sanitize_style(value)
                };
                if adapted.is_empty() {
                    continue;
                }
                output.push_str(&tag[whitespace_start..value_start]);
                output.push_str(&adapted);
                output.push(if quote == 0 { ' ' } else { quote as char });
                if quote == 0 {
                    output.pop();
                    output.push_str(&tag[value_end..attribute_end]);
                }
                cursor = attribute_end;
                continue;
            }

        if name == "text" && attribute_name == "fill"
            && let Some((value_start, value_end, quote)) = value_range
                && luminance(&tag[value_start..value_end]) <= LUMINANCE_THRESHOLD
            {
                output.push_str(&tag[whitespace_start..value_start]);
                output.push_str("currentColor");
                if quote != 0 {
                    output.push(quote as char);
                }
                output.push_str(&tag[value_end..attribute_end]);
                cursor = attribute_end;
                continue;
            }

        let _ = name_end;
        output.push_str(&tag[whitespace_start..attribute_end]);
        cursor = attribute_end;
    }
    output
}

fn sanitize_body_style(style: &str) -> String {
    declarations(style)
        .into_iter()
        .filter(|(name, _)| !matches!(name.as_str(), "background-color" | "color"))
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn sanitize_style(style: &str) -> String {
    let mut properties = declarations(style);

    if property(&properties, "mix-blend-mode").is_some_and(|value| value != "normal") {
        properties.retain(|(name, _)| name != "mix-blend-mode");
    }

    let background_image = property(&properties, "background-image");
    if background_image.is_some_and(|value| value != "none" && !value.contains("gradient")) {
        properties.retain(|(name, _)| name != "background-image");
    }
    if property(&properties, "background").is_some_and(|value| value.contains("gradient")) {
        properties.retain(|(name, _)| name != "background");
        properties.retain(|(name, _)| name != "background-image");
    }

    let color = property(&properties, "color").map(str::to_owned);
    let background = property(&properties, "background").map(str::to_owned);
    let background_color = property(&properties, "background-color")
        .map(str::to_owned)
        .or_else(|| background.as_deref().and_then(background_color).map(str::to_owned));
    let has_background = background.is_some() || background_color.is_some();

    if !has_background || background_color.as_deref().is_some_and(is_transparent) {
        if color.as_deref().is_some_and(|value| luminance(value) <= LUMINANCE_THRESHOLD) {
            properties.retain(|(name, _)| name != "color");
        }
    } else {
        let bg = background_color.as_deref().unwrap_or_default();
        let low_contrast = color.as_deref().is_some_and(|fg| contrast(bg, fg) < CONTRAST_THRESHOLD);
        if luminance(bg) > LUMINANCE_THRESHOLD || low_contrast {
            properties.retain(|(name, _)| !matches!(name.as_str(), "background" | "background-color"));
            if color.as_deref().is_some_and(|value| luminance(value) <= LUMINANCE_THRESHOLD) {
                properties.retain(|(name, _)| name != "color");
            }
        }
    }

    properties
        .into_iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn property<'a>(properties: &'a [(String, String)], name: &str) -> Option<&'a str> {
    properties
        .iter()
        .rev()
        .find(|(property, _)| property == name)
        .map(|(_, value)| value.as_str())
}

fn declarations(style: &str) -> Vec<(String, String)> {
    split_css(style, b';')
        .into_iter()
        .filter_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                None
            } else {
                Some((name, value.to_owned()))
            }
        })
        .collect()
}

fn split_css(input: &str, delimiter: u8) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote = 0u8;
    let mut comment = false;
    let mut index = 0;
    while index < bytes.len() {
        if comment {
            if bytes[index..].starts_with(b"*/") {
                comment = false;
                index += 2;
                continue;
            }
        } else if quote != 0 {
            if bytes[index] == b'\\' {
                index += 1;
            } else if bytes[index] == quote {
                quote = 0;
            }
        } else {
            match bytes[index] {
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    comment = true;
                    index += 2;
                    continue;
                }
                b'\'' | b'"' => quote = bytes[index],
                b'(' => depth += 1,
                b')' => depth = depth.saturating_sub(1),
                byte if byte == delimiter && depth == 0 => {
                    parts.push(&input[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
        }
        index += 1;
    }
    parts.push(&input[start..]);
    parts
}

fn adapt_stylesheet(css: &str) -> String {
    let mut output = String::with_capacity(css.len());
    let mut cursor = 0;
    while let Some(open) = find_css_open(css, cursor) {
        let Some(close) = matching_css_close(css, open) else {
            output.push_str(&css[cursor..]);
            return output;
        };
        output.push_str(&css[cursor..=open]);
        let body = &css[open + 1..close];
        let adapted = if find_css_open(body, 0).is_some() {
            adapt_stylesheet(body)
        } else {
            sanitize_style(body)
        };
        output.push_str(&adapted);
        output.push('}');
        cursor = close + 1;
    }
    output.push_str(&css[cursor..]);
    output
}

fn find_css_open(css: &str, start: usize) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut quote = 0u8;
    let mut comment = false;
    let mut index = start;
    while index < bytes.len() {
        if comment {
            if bytes[index..].starts_with(b"*/") {
                comment = false;
                index += 2;
                continue;
            }
        } else if quote != 0 {
            if bytes[index] == b'\\' {
                index += 1;
            } else if bytes[index] == quote {
                quote = 0;
            }
        } else {
            match bytes[index] {
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    comment = true;
                    index += 2;
                    continue;
                }
                b'\'' | b'"' => quote = bytes[index],
                b'{' => return Some(index),
                _ => {}
            }
        }
        index += 1;
    }
    None
}

fn matching_css_close(css: &str, open: usize) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut depth = 1usize;
    let mut quote = 0u8;
    let mut comment = false;
    let mut index = open + 1;
    while index < bytes.len() {
        if comment {
            if bytes[index..].starts_with(b"*/") {
                comment = false;
                index += 2;
                continue;
            }
        } else if quote != 0 {
            if bytes[index] == b'\\' {
                index += 1;
            } else if bytes[index] == quote {
                quote = 0;
            }
        } else {
            match bytes[index] {
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    comment = true;
                    index += 2;
                    continue;
                }
                b'\'' | b'"' => quote = bytes[index],
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        index += 1;
    }
    None
}

fn background_color(background: &str) -> Option<&str> {
    let background = background.trim();
    if background.starts_with('#') {
        let end = background
            .find(|character: char| character.is_ascii_whitespace() || character == ')')
            .unwrap_or(background.len());
        return Some(&background[..end]);
    }
    let lowered = background.to_ascii_lowercase();
    for function in ["rgb(", "rgba(", "hsl(", "hsla("] {
        if let Some(start) = lowered.find(function)
            && let Some(end) = background[start..].find(')') {
                return Some(&background[start..start + end + 1]);
            }
    }
    background.split_whitespace().find(|word| is_named_color(word))
}

fn is_named_color(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "gray"
            | "grey"
            | "lightgray"
            | "lightgrey"
            | "darkgray"
            | "darkgrey"
            | "silver"
            | "navy"
            | "maroon"
            | "purple"
            | "teal"
            | "aqua"
            | "fuchsia"
            | "olive"
            | "orange"
            | "transparent"
    )
}

fn is_transparent(value: &str) -> bool {
    color(value).is_none_or(|(_, _, _, alpha)| alpha <= 0.2)
}

fn luminance(value: &str) -> f64 {
    color(value).map_or(0.0, |(red, green, blue, _)| {
        0.2125 * red + 0.7154 * green + 0.0721 * blue
    })
}

fn contrast(background: &str, foreground: &str) -> f64 {
    let background = luminance(background);
    let foreground = luminance(foreground);
    (background.max(foreground) + 0.05) / (background.min(foreground) + 0.05)
}

fn color(value: &str) -> Option<(f64, f64, f64, f64)> {
    let value = value.trim().trim_end_matches("!important").trim();
    let lowered = value.to_ascii_lowercase();
    let named = match lowered.as_str() {
        "black" => Some((0.0, 0.0, 0.0, 1.0)),
        "white" => Some((255.0, 255.0, 255.0, 1.0)),
        "red" => Some((255.0, 0.0, 0.0, 1.0)),
        "green" => Some((0.0, 128.0, 0.0, 1.0)),
        "blue" => Some((0.0, 0.0, 255.0, 1.0)),
        "yellow" => Some((255.0, 255.0, 0.0, 1.0)),
        "gray" | "grey" => Some((128.0, 128.0, 128.0, 1.0)),
        "lightgray" | "lightgrey" => Some((211.0, 211.0, 211.0, 1.0)),
        "darkgray" | "darkgrey" => Some((169.0, 169.0, 169.0, 1.0)),
        "silver" => Some((192.0, 192.0, 192.0, 1.0)),
        "navy" => Some((0.0, 0.0, 128.0, 1.0)),
        "maroon" => Some((128.0, 0.0, 0.0, 1.0)),
        "purple" => Some((128.0, 0.0, 128.0, 1.0)),
        "teal" => Some((0.0, 128.0, 128.0, 1.0)),
        "aqua" => Some((0.0, 255.0, 255.0, 1.0)),
        "fuchsia" => Some((255.0, 0.0, 255.0, 1.0)),
        "olive" => Some((128.0, 128.0, 0.0, 1.0)),
        "orange" => Some((255.0, 165.0, 0.0, 1.0)),
        "transparent" => Some((0.0, 0.0, 0.0, 0.0)),
        _ => None,
    };
    if named.is_some() {
        return named;
    }

    if let Some(hex) = lowered.strip_prefix('#') {
        let expanded;
        let digits = match hex.len() {
            3 | 4 => {
                expanded = hex.chars().flat_map(|c| [c, c]).collect::<String>();
                expanded.as_str()
            }
            6 | 8 => hex,
            _ => return None,
        };
        let value = |index| u8::from_str_radix(&digits[index..index + 2], 16).ok();
        return Some((
            f64::from(value(0)?),
            f64::from(value(2)?),
            f64::from(value(4)?),
            if digits.len() == 8 {
                f64::from(value(6)?) / 255.0
            } else {
                1.0
            },
        ));
    }

    if lowered.starts_with("rgb(") || lowered.starts_with("rgba(") {
        let values: Vec<_> = lowered
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')')
            .split([',', '/', ' '])
            .filter(|part| !part.is_empty())
            .collect();
        if values.len() < 3 {
            return None;
        }
        let channel = |value: &str| {
            value
                .strip_suffix('%')
                .and_then(|percent| percent.parse::<f64>().ok().map(|v| v * 2.55))
                .or_else(|| value.parse::<f64>().ok())
        };
        let alpha = values
            .get(3)
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0);
        return Some((
            channel(values[0])?,
            channel(values[1])?,
            channel(values[2])?,
            alpha,
        ));
    }
    None
}

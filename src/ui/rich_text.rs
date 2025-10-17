//! Provides an adapter for `html2text` to `ratatui` rich text.
use anyhow::{Context, Result};
use html_escape::decode_html_entities;
use html2text::render::{RichAnnotation, TaggedLine};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Adapter for `html2text` to `ratatui` rich text.
///
/// This is a simple adapter to convert the rich annotations from `html2text` to
/// `ratatui` rich text.
pub fn html_to_rich_text(html: &str) -> Result<Vec<Line<'_>>> {
    let html = decode_html_entities(html);
    let tagged_lines = html2text::from_read_rich(html.as_bytes(), usize::MAX)
        .context("failed to get html2text RichAnnotations")?;
    Ok(tagged_lines.into_iter().map(tagged_line_to_line).collect())
}

/// Convert a [`TaggedLine`] to a [`Line`].
fn tagged_line_to_line(tagged_line: TaggedLine<Vec<RichAnnotation>>) -> Line<'static> {
    let spans: Vec<Span> = tagged_line
        .tagged_strings()
        .map(|tagged_str| {
            let style = annotations_to_style(&tagged_str.tag);
            Span::styled(tagged_str.s.to_string(), style)
        })
        .collect();
    Line::from(spans)
}

/// Convert and combine a slice of [`RichAnnotation`] to a [`Style`].
fn annotations_to_style(annotations: &[RichAnnotation]) -> Style {
    let mut style = Style::default();
    for ann in annotations {
        style = match ann {
            RichAnnotation::Link(_) => style
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(Color::Cyan)
                .fg(Color::Blue),
            RichAnnotation::Emphasis => style.add_modifier(Modifier::ITALIC),
            RichAnnotation::Strong => style.add_modifier(Modifier::BOLD),
            RichAnnotation::Strikeout => style.add_modifier(Modifier::CROSSED_OUT),
            RichAnnotation::Code => style.underline_color(Color::Yellow).bg(Color::DarkGray),
            RichAnnotation::Image(_) => style.fg(Color::Blue),
            _ => style,
        }
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_strong_html_to_rich_text() {
        let html = "<strong>Hello</strong>";
        let expected = vec![Line::from(vec![Span::styled(
            "Hello",
            Style::default().add_modifier(Modifier::BOLD),
        )])];
        assert_eq!(html_to_rich_text(html).unwrap(), expected);
    }

    #[test]
    fn test_single_italic_html_to_rich_text() {
        let html = "<em>Hello</em>";
        let expected = vec![Line::from(vec![Span::styled(
            "Hello",
            Style::default().add_modifier(Modifier::ITALIC),
        )])];
        assert_eq!(html_to_rich_text(html).unwrap(), expected);
    }

    #[test]
    fn test_double_encoded_html() {
        let double_encoded = r#"&lt;p&gt;This is &lt;strong&gt;bold&lt;/strong&gt; text&lt;/p&gt;"#;
        let lines = html_to_rich_text(double_encoded).unwrap();
        assert!(!lines.is_empty());

        // The text should not contain &lt; or &gt;
        let text = format!("{:?}", lines);
        assert!(!text.contains("&lt;"));
    }
}

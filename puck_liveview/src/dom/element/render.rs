use super::Element;

impl Element {
    /// Render the `Element` as HTML. This speeds up page load times, because we send this first.
    #[allow(unused)]
    pub(crate) fn render(&self) -> String {
        self.write_opening_tag()
            + if let Some(ref text) = self.text {
                text
            } else {
                ""
            }
            + &self
                .children
                .iter()
                .map(|child| child.render())
                .collect::<String>()
            + &self.write_closing_tag()
    }

    /// Renders the opening tag as an HTML string. Note that we intentionally do not write the
    /// event listeners as HTML; we emit instructions to add them that we then stream over the
    /// WebSocket.
    fn write_opening_tag(&self) -> String {
        "<".to_string() + &self.name + " " + &self.write_attributes() + " " + ">"
    }

    /// Returns the closing tag.
    fn write_closing_tag(&self) -> String {
        "</".to_string() + &self.name + ">"
    }

    /// Writes the attributes for this element.
    fn write_attributes(&self) -> String {
        self.attributes
            .iter()
            .map(|(key, value)| "\"".to_string() + key + "\"=" + "\"" + value + "\" ")
            .collect::<String>()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
/// We only run these tests on the native target.
mod test_render {
    use scraper::{Html, Selector};

    use crate::dom::element::Element;

    #[test]
    fn test_render_nested_tags() {
        let tree = Element {
            id: 0,
            name: "div".into(),
            children: vec![
                Element {
                    id: 1,
                    name: "p".into(),
                    key: Some("a".to_string()),
                    text: Some("the cat sat on the mat".into()),
                    ..Default::default()
                },
                Element {
                    id: 2,
                    name: "p".into(),
                    key: Some("b".to_string()),
                    text: Some("the mat sat on the cat".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let html = tree.render();
        let fragment = Html::parse_fragment(&html);

        let div_selector = Selector::parse("div").unwrap();
        let p_selector = Selector::parse("p").unwrap();

        let p_tags = fragment.select(&p_selector).collect::<Vec<_>>();

        assert_eq!(fragment.select(&div_selector).collect::<Vec<_>>().len(), 1);
        assert_eq!(p_tags.len(), 2);

        let first_tag = p_tags[0];
        assert_eq!(
            first_tag
                .first_child()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string(),
            "the cat sat on the mat".to_string()
        );

        let second_tag = p_tags[1];
        assert_eq!(
            second_tag
                .first_child()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string(),
            "the mat sat on the cat".to_string()
        );
    }
}

use serde::{Deserialize, Serialize};

/// Represents a single section in a meeting template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    /// Section title (e.g., "Summary", "Action Items")
    pub title: String,

    /// Instruction for the LLM on what to extract/include
    pub instruction: String,

    /// Format type: "paragraph", "list", or "string"
    pub format: String,

    /// Optional markdown formatting hint for list items (e.g., table structure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_format: Option<String>,

    /// Alternative formatting hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_item_format: Option<String>,
}

/// Represents a complete meeting template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template display name
    pub name: String,

    /// Brief description of the template's purpose
    pub description: String,

    /// List of sections in the template
    pub sections: Vec<TemplateSection>,
}

impl Template {
    /// Validates the template structure
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Template name cannot be empty".to_string());
        }

        if self.description.is_empty() {
            return Err("Template description cannot be empty".to_string());
        }

        if self.sections.is_empty() {
            return Err("Template must have at least one section".to_string());
        }

        for (i, section) in self.sections.iter().enumerate() {
            if section.title.is_empty() {
                return Err(format!("Section {} has empty title", i));
            }

            if section.instruction.is_empty() {
                return Err(format!("Section '{}' has empty instruction", section.title));
            }

            match section.format.as_str() {
                "paragraph" | "list" | "string" => {},
                other => return Err(format!(
                    "Section '{}' has invalid format '{}'. Must be 'paragraph', 'list', or 'string'",
                    section.title, other
                )),
            }
        }

        Ok(())
    }

    /// Generates a clean markdown template scaffold using real ATX headings
    pub fn to_markdown_structure(&self) -> String {
        let mut markdown = String::from("# [AI-Generated Title]\n");

        for section in &self.sections {
            markdown.push_str(&format!("\n## {}\n\n", section.title));

            if section.format == "list" {
                let example = section
                    .item_format
                    .as_ref()
                    .or(section.example_item_format.as_ref())
                    .map(|format| format.as_str())
                    .unwrap_or("- ");
                markdown.push_str(example);
                markdown.push('\n');
            }
        }

        markdown
    }

    /// Generates section-specific instructions for the LLM
    pub fn to_section_instructions(&self) -> String {
        let mut instructions = String::from(
            "- **For the main title (`# [AI-Generated Title]`):** Analyze the entire transcript and create a concise, descriptive title for the meeting.\n"
        );

        for section in &self.sections {
            instructions.push_str(&format!(
                "- **For the '{}' section:** {}.\n",
                section.title, section.instruction
            ));

            // Add item format instructions if present
            let item_format = section.item_format.as_ref()
                .or(section.example_item_format.as_ref());

            if let Some(format) = item_format {
                instructions.push_str(&format!(
                    "  - Items in this section should follow the format: `{}`.\n",
                    format
                ));
            }
        }

        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_template() {
        let template = Template {
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            sections: vec![
                TemplateSection {
                    title: "Summary".to_string(),
                    instruction: "Provide a summary".to_string(),
                    format: "paragraph".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
            ],
        };

        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let template = Template {
            name: "".to_string(),
            description: "A test template".to_string(),
            sections: vec![],
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_format() {
        let template = Template {
            name: "Test".to_string(),
            description: "Test".to_string(),
            sections: vec![
                TemplateSection {
                    title: "Test".to_string(),
                    instruction: "Test".to_string(),
                    format: "invalid".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
            ],
        };

        assert!(template.validate().is_err());
    }

    fn section(title: &str, format: &str) -> TemplateSection {
        TemplateSection {
            title: title.to_string(),
            instruction: "Fill this section".to_string(),
            format: format.to_string(),
            item_format: None,
            example_item_format: None,
        }
    }

    fn section_with_formats(
        title: &str,
        format: &str,
        item_format: Option<&str>,
        example_item_format: Option<&str>,
    ) -> TemplateSection {
        TemplateSection {
            title: title.to_string(),
            instruction: "Fill this section".to_string(),
            format: format.to_string(),
            item_format: item_format.map(|s| s.to_string()),
            example_item_format: example_item_format.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_scaffold_first_line_is_ai_generated_title_h1() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Summary", "paragraph")],
        };

        let output = template.to_markdown_structure();
        assert_eq!(
            output.lines().next(),
            Some("# [AI-Generated Title]"),
            "first line must be the literal H1 title placeholder, got: {output:?}"
        );
    }

    #[test]
    fn test_scaffold_uses_atx_h2_section_headings() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Action Items", "list"), section("Summary", "paragraph")],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("\n## Action Items\n") || output.contains("## Action Items\n"),
            "list section must use an ATX level-2 heading, got: {output:?}"
        );
        assert!(
            output.contains("## Summary\n"),
            "paragraph section must use an ATX level-2 heading, got: {output:?}"
        );
        assert!(
            !output.contains("**"),
            "no bold-paragraph section markers may remain, got: {output:?}"
        );
    }

    #[test]
    fn test_list_section_uses_item_format_over_example_item_format() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section_with_formats(
                "Decisions",
                "list",
                Some("- Decision: {}"),
                Some("* Example item format"),
            )],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("- Decision: {}"),
            "item_format must win over example_item_format, got: {output:?}"
        );
        assert!(
            !output.contains("* Example item format"),
            "example_item_format must not be emitted when item_format is set, got: {output:?}"
        );
    }

    #[test]
    fn test_list_section_falls_back_to_example_item_format() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section_with_formats(
                "Decisions",
                "list",
                None,
                Some("* Example item format"),
            )],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("* Example item format"),
            "example_item_format must be used when item_format is None, got: {output:?}"
        );
    }

    #[test]
    fn test_list_section_both_formats_none_bare_bullet_fallback() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Action Items", "list")],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("\n- \n"),
            "both format fields None must fall back to a bare '- ' line, got: {output:?}"
        );
    }

    #[test]
    fn test_table_item_format_emitted_verbatim_without_synthesized_rows() {
        let table = "| Decision | Owner |\n|----------|-------|";
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section_with_formats("Decisions", "list", Some(table), None)],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("## Decisions\n\n| Decision | Owner |\n|----------|-------|\n"),
            "stored table string must be emitted verbatim after the heading, got: {output:?}"
        );
        assert!(
            !output.contains("----------|-------|\n|"),
            "no example data rows may be synthesized below the separator, got: {output:?}"
        );
    }

    #[test]
    fn test_paragraph_section_has_no_example_line() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Summary", "paragraph")],
        };

        let output = template.to_markdown_structure();
        assert!(output.contains("## Summary\n"), "got: {output:?}");
        assert!(
            !output.contains("- "),
            "paragraph sections must not get an example line, got: {output:?}"
        );
    }

    #[test]
    fn test_string_section_has_no_example_line() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Meeting Notes", "string")],
        };

        let output = template.to_markdown_structure();
        assert!(output.contains("## Meeting Notes\n"), "got: {output:?}");
        assert!(
            !output.contains("- "),
            "string sections must not get an example line, got: {output:?}"
        );
    }

    #[test]
    fn test_exact_scaffold_structure() {
        let table = "| Decision | Owner |\n|----------|-------|";
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![
                section_with_formats("Action Items", "list", Some(table), None),
                section("Summary", "paragraph"),
            ],
        };

        let expected = "# [AI-Generated Title]\n\n## Action Items\n\n| Decision | Owner |\n|----------|-------|\n\n## Summary\n\n";
        assert_eq!(
            template.to_markdown_structure(),
            expected,
            "scaffold structure must match the canonical markdown exactly"
        );
    }

    #[test]
    fn test_bullet_only_item_format_emitted_verbatim() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section_with_formats("Items", "list", Some("- "), None)],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("## Items\n\n- \n"),
            "bullet-only item_format must be emitted as a single bare bullet, got: {output:?}"
        );
        assert!(
            !output.contains("- - "),
            "bullet item_format must not be doubled, got: {output:?}"
        );
    }

    #[test]
    fn test_item_format_without_separator_row_emitted_verbatim() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section_with_formats(
                "Action Items",
                "list",
                Some("- **Owner**: action description"),
                None,
            )],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("## Action Items\n\n- **Owner**: action description\n"),
            "non-table item_format must be emitted verbatim after the heading, got: {output:?}"
        );
    }

    #[test]
    fn test_paragraph_only_template_structure() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Summary", "paragraph"), section("Discussion", "paragraph")],
        };

        let output = template.to_markdown_structure();
        assert!(output.contains("## Summary\n"), "got: {output:?}");
        assert!(output.contains("## Discussion\n"), "got: {output:?}");
        assert!(
            !output.contains("- ") && !output.contains("**"),
            "paragraph-only template must contain headings only, got: {output:?}"
        );
    }

    #[test]
    fn test_single_section_template() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Action Items", "list")],
        };

        let output = template.to_markdown_structure();
        assert_eq!(
            output.matches("## Action Items").count(),
            1,
            "single section must produce exactly one heading, got: {output:?}"
        );
        assert_eq!(
            output.matches("# [AI-Generated Title]").count(),
            1,
            "single section must produce exactly one H1, got: {output:?}"
        );
    }

    #[test]
    fn test_section_title_with_hash_and_pipe_verbatim() {
        let template = Template {
            name: "T".to_string(),
            description: "D".to_string(),
            sections: vec![section("Risks #1 | Blockers", "paragraph")],
        };

        let output = template.to_markdown_structure();
        assert!(
            output.contains("\n## Risks #1 | Blockers\n"),
            "section title with '#' and '|' must be emitted verbatim in the heading, got: {output:?}"
        );
    }
}

use std::path::{Component, Path};

/// Truncate a directory path to the last 3 components with `...` prefix
/// when the path has more than 3 non-root components.
pub fn format_directory(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let p = Path::new(path);
    let non_root: Vec<&str> = p
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();

    if non_root.len() > 3 {
        let last_three = &non_root[non_root.len() - 3..];
        format!(".../{}", last_three.join("/"))
    } else if non_root.is_empty() {
        // Root path or equivalent
        "/".to_string()
    } else {
        format!("/{}", non_root.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_five_component_path_to_last_three_with_ellipsis_prefix() {
        assert_eq!(
            format_directory("/Users/jamescarr/Git/jamesacarr/claude-statusline"),
            ".../Git/jamesacarr/claude-statusline"
        );
    }

    #[test]
    fn truncates_four_component_path_to_last_three_with_ellipsis_prefix() {
        assert_eq!(
            format_directory("/Users/jamescarr/Git/jamesacarr"),
            ".../jamescarr/Git/jamesacarr"
        );
    }

    #[test]
    fn returns_three_component_path_unchanged() {
        assert_eq!(
            format_directory("/Users/jamescarr/project"),
            "/Users/jamescarr/project"
        );
    }

    #[test]
    fn returns_two_component_path_unchanged() {
        assert_eq!(format_directory("/Users/jamescarr"), "/Users/jamescarr");
    }

    #[test]
    fn returns_single_component_path_unchanged() {
        assert_eq!(format_directory("/tmp"), "/tmp");
    }

    #[test]
    fn returns_root_path_unchanged() {
        assert_eq!(format_directory("/"), "/");
    }

    #[test]
    fn handles_trailing_slash_same_as_without() {
        assert_eq!(
            format_directory("/Users/jamescarr/project/"),
            "/Users/jamescarr/project"
        );
    }

    #[test]
    fn handles_path_with_spaces_in_components() {
        assert_eq!(
            format_directory("/Users/james carr/My Project/foo/bar"),
            ".../My Project/foo/bar"
        );
    }

    #[test]
    fn returns_empty_string_for_empty_input() {
        assert_eq!(format_directory(""), "");
    }
}

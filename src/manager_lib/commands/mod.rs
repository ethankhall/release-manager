pub mod versions;

use regex::Regex;

/// Validate that a path can point to a repo. A repo path consists of two parts, the project
/// and the repo.
pub(crate) fn is_path_repo(path: String) -> Result<(), String> {
    let re = Regex::new(r"^([0-9A-Za-z_\-\.]{1,})/([0-9A-Za-z_\-\.]{1,})$").unwrap();
    return if re.is_match(&path) {
        Ok(())
    } else {
        Err(s!("Path must be in format <project>/<repo>. An example is ethankhall/version-management."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_values() {
        assert_eq!(is_path_repo(s!("ethankhall/version-management")), Ok(()));
        assert!(is_path_repo(s!("ethankhall/ version-management")).is_err());
    }
}
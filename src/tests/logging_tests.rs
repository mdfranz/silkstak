#[cfg(test)]
mod tests {
    use crate::resolve_log_file_path;

    #[test]
    fn test_log_file_path_no_env_var() {
        let result = resolve_log_file_path(None, false);
        assert_eq!(result, Some("silkstak.log".to_string()));
    }

    #[test]
    fn test_log_file_path_disabled() {
        assert_eq!(resolve_log_file_path(Some("0".to_string()), false), None);
        assert_eq!(
            resolve_log_file_path(Some("false".to_string()), false),
            None
        );
        assert_eq!(resolve_log_file_path(Some("off".to_string()), false), None);
    }

    #[test]
    fn test_log_file_path_explicit_path() {
        let result = resolve_log_file_path(Some("/tmp/custom.log".to_string()), false);
        assert_eq!(result, Some("/tmp/custom.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_silkstak_log_when_set_to_1() {
        let result = resolve_log_file_path(Some("1".to_string()), false);
        assert_eq!(result, Some("silkstak.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_silkstak_log_when_set_to_true() {
        let result = resolve_log_file_path(Some("true".to_string()), false);
        assert_eq!(result, Some("silkstak.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_silkstak_log_when_empty() {
        let result = resolve_log_file_path(Some("".to_string()), false);
        assert_eq!(result, Some("silkstak.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_silkstak_log_when_rust_log_is_set() {
        let result = resolve_log_file_path(Some("debug".to_string()), true);
        assert_eq!(result, Some("silkstak.log".to_string()));
    }

    #[test]
    fn test_log_file_path_respects_custom_path_when_no_rust_log() {
        let result = resolve_log_file_path(Some("/var/log/app.log".to_string()), false);
        assert_eq!(result, Some("/var/log/app.log".to_string()));
    }
}

#[cfg(test)]
mod logging_tests {
    use crate::resolve_log_file_path;

    #[test]
    fn test_log_file_path_no_env_var() {
        let result = resolve_log_file_path(None, false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_log_file_path_explicit_path() {
        let result = resolve_log_file_path(Some("/tmp/custom.log".to_string()), false);
        assert_eq!(result, Some("/tmp/custom.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_zerostack_log_when_set_to_1() {
        let result = resolve_log_file_path(Some("1".to_string()), false);
        assert_eq!(result, Some("zerostack.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_zerostack_log_when_set_to_true() {
        let result = resolve_log_file_path(Some("true".to_string()), false);
        assert_eq!(result, Some("zerostack.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_zerostack_log_when_empty() {
        let result = resolve_log_file_path(Some("".to_string()), false);
        assert_eq!(result, Some("zerostack.log".to_string()));
    }

    #[test]
    fn test_log_file_path_defaults_to_zerostack_log_when_rust_log_is_set() {
        let result = resolve_log_file_path(Some("debug".to_string()), true);
        assert_eq!(result, Some("zerostack.log".to_string()));
    }

    #[test]
    fn test_log_file_path_respects_custom_path_when_no_rust_log() {
        let result = resolve_log_file_path(Some("/var/log/app.log".to_string()), false);
        assert_eq!(result, Some("/var/log/app.log".to_string()));
    }
}

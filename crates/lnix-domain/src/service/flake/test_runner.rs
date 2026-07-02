//! Renders the bash test-runner block embedded in the shell hook.
//!
//! The block runs each declared test command, tallies pass/fail counts,
//! and exits non-zero if any command failed. It is gated behind
//! `LAZYNIX_TEST_MODE` by the caller.

/// Renders the test-runner script for the given commands.
///
/// Returns an empty string when no tests are declared, so the caller
/// can skip emitting the `LAZYNIX_TEST_MODE` guard entirely.
pub(super) fn render_test_execution(tests: &[String]) -> String {
    if tests.is_empty() {
        return String::new();
    }

    let test_commands = tests
        .iter()
        .map(|cmd| format!("        \"{}\"", cmd))
        .collect::<Vec<_>>()
        .join(" \\\n");

    format!(
        r#"TESTS_FAILED=0
TESTS_PASSED=0

echo "Running tests..."
echo "================="

for test_cmd in \
{}
do
    echo ""
    echo "Running: $test_cmd"
    echo "---"
    if bash -c "$test_cmd"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        echo "[PASS] $test_cmd"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        echo "[FAIL] $test_cmd"
    fi
done

echo ""
echo "================="
echo "Test Results: $TESTS_PASSED passed, $TESTS_FAILED failed"

if [ $TESTS_FAILED -gt 0 ]; then
    exit 1
fi"#,
        test_commands
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_empty_when_no_tests() {
        // Arrange
        let tests: Vec<String> = vec![];

        // Act
        let script = render_test_execution(&tests);

        // Assert
        assert_eq!(script, "");
    }

    #[test]
    fn renders_loop_over_each_command() {
        // Arrange
        let tests = vec!["pytest".to_string(), "cargo test".to_string()];

        // Act
        let script = render_test_execution(&tests);

        // Assert
        assert!(script.contains("pytest"));
        assert!(script.contains("cargo test"));
        assert!(script.contains("for test_cmd in"));
        assert!(script.contains("Test Results:"));
        assert!(script.contains("exit 1"));
    }
}

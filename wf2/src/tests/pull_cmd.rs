#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::{_commands, _file_ops};
    use std::env::current_dir;
    use std::path::PathBuf;
    use wf2_core::task::Task;
    use wf2_core::util::path_buf_to_string;

    #[test]
    fn test_pull_single_top_level() {
        let args = vec!["prog", "pull", "vendor"];
        let cwd = "/users/acme";
        let expected_commands = vec![
            "docker exec wf2__acme__php test -e /var/www/vendor",
            "docker cp wf2__acme__php:/var/www/vendor /users/acme",
        ];
        test_pull(args, cwd, expected_commands, vec![]);
    }

    #[test]
    fn test_pull_single_nested() {
        let args = vec!["prog", "pull", "vendor/wearejh"];
        let cwd = "/users/acme";
        let expected_commands = vec![
            "docker exec wf2__acme__php test -e /var/www/vendor/wearejh",
            "docker cp wf2__acme__php:/var/www/vendor/wearejh /users/acme/vendor",
        ];
        let expected_file_ops = vec![Task::dir_create("/users/acme/vendor", "Directory creation")];
        test_pull(args, cwd, expected_commands, expected_file_ops);
    }

    #[test]
    fn test_pull_multi() {
        let args = vec!["prog", "pull", "file1", "var/log"];
        let cwd = "/users/acme";
        test_pull(
            args,
            cwd,
            vec![
                "docker exec wf2__acme__php test -e /var/www/file1",
                "docker exec wf2__acme__php test -e /var/www/var/log",
                "docker cp wf2__acme__php:/var/www/file1 /users/acme",
                "docker cp wf2__acme__php:/var/www/var/log /users/acme/var",
            ],
            vec![Task::dir_create("/users/acme/var", "Directory creation")],
        )
    }

    #[test]
    fn test_pull_file() {
        let args = vec!["prog", "pull", "fixtures/wf2_overrides/site.conf"];
        let cwd = current_dir().expect("works");
        let parent = cwd.join("fixtures/wf2_overrides");
        let cp_cmd = format!(
            "docker cp wf2__wf2__php:/var/www/fixtures/wf2_overrides/site.conf {}",
            path_buf_to_string(&parent)
        );
        let expected_commands = vec![
            "docker exec wf2__wf2__php test -e /var/www/fixtures/wf2_overrides/site.conf",
            &cp_cmd,
        ];
        test_pull(
            args,
            cwd,
            expected_commands,
            vec![Task::dir_create(parent.clone(), "Directory creation")],
        );
    }

    #[test]
    fn test_pull_folder_with_delete() {
        let args = vec!["prog", "pull", "fixtures/wf2_overrides"];
        let cwd = current_dir().expect("works").clone();
        let wf2_dir = cwd.parent().expect("root");
        let full_path = wf2_dir.join("fixtures/wf2_overrides");
        let fixtures = wf2_dir.join("fixtures");
        let cp_cmd = format!(
            "docker cp wf2__wf2__php:/var/www/fixtures/wf2_overrides {}",
            path_buf_to_string(&fixtures)
        );
        let expected_commands = vec![
            "docker exec wf2__wf2__php test -e /var/www/fixtures/wf2_overrides",
            &cp_cmd,
        ];
        test_pull(
            args,
            wf2_dir,
            expected_commands,
            vec![
                Task::dir_remove(full_path.clone(), "Directory Removal"),
                Task::dir_create(full_path.clone(), "Directory creation"),
            ],
        );
    }

    fn test_pull(
        args: Vec<&str>,
        cwd: impl Into<PathBuf>,
        expected_commands: Vec<&str>,
        expected_file_ops: Vec<Task>,
    ) {
        let cwd = cwd.into();
        let get_tasks = move || {
            let input = CLIInput::_from_args(args.clone())._with_cwd(cwd.clone());
            let cli_output = CLIOutput::from_input(input);
            cli_output.expect("test").tasks.unwrap()
        };
        assert_eq!(_commands(get_tasks()), expected_commands);
        assert_eq!(_file_ops(get_tasks()), _file_ops(expected_file_ops));
    }
}

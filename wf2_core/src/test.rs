use crate::task::{FileOpPaths, Task};
use std::path::PathBuf;

use crate::cli::cli_input::CLIInput;
use crate::cli::cli_output::CLIOutput;
use crate::file_op::FileOp;

#[derive(Debug)]
pub struct Test {
    pub recipe: Option<String>,
    pub file: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub args: Vec<String>,
    pub cli_input: Option<CLIInput>,
}

impl Test {
    pub fn new(args: impl Into<Vec<String>>) -> Test {
        Test {
            args: args.into(),
            recipe: None,
            file: None,
            cwd: None,
            cli_input: None,
        }
    }
    pub fn from_cmd(cmd: impl Into<String>) -> Test {
        let str = cmd.into();
        let args: Vec<String> = str.trim().split(' ').map(String::from).collect();
        Test::new(args)
    }
    pub fn from_skipped(cmd: impl Into<String>, count: usize) -> Test {
        let input: String = cmd.into();
        let split = input.trim().split(' ');
        let mut before = split
            .clone()
            .take(count)
            .map(String::from)
            .collect::<Vec<String>>();
        let after = split
            .clone()
            .skip(count)
            .map(String::from)
            .collect::<Vec<String>>()
            .join(" ");
        let trimmed = &after[1..after.len() - 1];
        before.push(trimmed.to_string());
        Test::new(before)
    }
    pub fn with_file(&mut self, p: impl Into<PathBuf>) -> &mut Self {
        self.file = Some(p.into());
        self
    }
    pub fn with_cwd(&mut self, cwd: impl Into<PathBuf>) -> &mut Self {
        self.cwd = Some(cwd.into());
        self
    }
    pub fn with_recipe(&mut self, recipe: impl Into<String>) -> &mut Self {
        self.recipe = Some(recipe.into());
        self
    }
    pub fn with_cli_input(&mut self, cli_input: CLIInput) -> &mut Self {
        self.cli_input = Some(cli_input);
        self
    }
    pub fn tasks(&mut self) -> Vec<Task> {
        let mut args = vec![String::from("wf2")];

        // the args minus the cli name
        let cmd_args: Vec<String> = self.args.clone().into_iter().skip(1).collect();

        // if a file was given, load it as `--config`
        if let Some(pb) = self.file.as_ref() {
            let path = pb.clone().to_string_lossy().to_string();
            let config = vec![String::from("--config"), path];
            args.extend(config);
        }

        // if a file was given, load it as `--cwd`
        if let Some(cwd) = self.cwd.as_ref() {
            let path = cwd.clone().to_string_lossy().to_string();
            let cwd_config = vec![String::from("--cwd"), path];
            args.extend(cwd_config);
        }

        // if a recipe was given, load it as `--recipe`
        if let Some(recipe) = self.recipe.as_ref() {
            let recipe = recipe.to_string();
            let recipe_config = vec![String::from("--recipe"), recipe];
            args.extend(recipe_config);
        }

        // now add the args given in the test
        args.extend(cmd_args);

        let cli_input_to_merge = self.cli_input.clone().unwrap_or_default();

        CLIOutput::from_input(CLIInput {
            args,
            ..cli_input_to_merge
        })
        .expect("Test")
        .tasks
        .expect("tasks test")
    }
    pub fn file_ops_commands(&mut self) -> (Vec<String>, Vec<FileOp>) {
        let tasks = self.tasks();
        (Test::_commands(&tasks), Test::_file_ops(&tasks))
    }
    pub fn file_ops_paths_commands(&mut self) -> (Vec<String>, FileOpPaths) {
        let tasks = self.tasks();
        (Test::_commands(&tasks), Task::file_op_paths(&tasks))
    }
    pub fn commands(&mut self) -> Vec<String> {
        let tasks = self.tasks();
        Test::_commands(&tasks)
    }
    pub fn _commands(tasks: &[Task]) -> Vec<String> {
        tasks.iter().fold(vec![], |mut acc, t| match t {
            Task::SimpleCommand { command, .. } | Task::Command { command, .. } => {
                acc.push(command.clone());
                acc
            }
            Task::Seq(tasks) => {
                let other = Test::_commands(tasks);
                acc.extend(other);
                acc
            }
            _ => acc,
        })
    }
    pub fn file_ops(&mut self) -> Vec<FileOp> {
        let tasks = self.tasks();
        Test::_file_ops(&tasks)
    }
    pub fn _file_ops(tasks: &[Task]) -> Vec<FileOp> {
        tasks.iter().fold(vec![], |mut acc, t| match t {
            Task::File { op, .. } => {
                acc.push(op.clone());
                acc
            }
            Task::Seq(tasks) => {
                let other = Test::_file_ops(tasks);
                acc.extend(other);
                acc
            }
            _ => acc,
        })
    }
}

pub fn test_cmd(path: impl Into<PathBuf>, cmd: impl Into<String>) -> Vec<Task> {
    let t = Test::from_cmd(cmd).with_file(path).tasks();
    dbg!(&t);
    t

    //    let ctx = Context::new_from_file(path).expect("res").expect("opt");
    //    let cmd = (EnvCmd).subcommands(&ctx);
    //    let app = App::new("wf2").subcommands(cmd);
    //    let matches = app.get_matches_from_safe(&args).expect("matches in test");
    //    (EnvCmd)
    //        .exec(matches.subcommand_matches("env"), &ctx)
    //        .unwrap()
}

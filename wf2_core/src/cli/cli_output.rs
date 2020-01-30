use crate::cli::{append_subcommands, CLI};

use clap::{App, ArgMatches};

use crate::cli::cli_input::CLIInput;
use crate::cli::CLIHelp;
use crate::commands::{internal_commands, CliCommand};
use crate::recipes::available_recipes;
use crate::recipes::recipe_kinds::RecipeKinds;
use crate::scripts::script::Script;
use crate::util::two_col;
use crate::{
    context::{Context, ContextOverrides, RunMode},
    task::Task,
};
use std::path::PathBuf;

pub struct CLIOutput {
    pub ctx: Context,
    pub tasks: Option<Vec<Task>>,
}

impl CLIOutput {
    ///
    /// Create CLIOutput from CLIInput
    ///
    pub fn from_input(input: CLIInput) -> Result<CLIOutput, failure::Error> {
        let input_args: Vec<String> = input.args.clone().into_iter().map(|s| s).collect();
        //        let base_len = 6;
        let cli = CLI::new();
        let ctx = cli.get_ctx(input.args.clone())?;
        let mut help_text = vec![];

        // Add recipe pass thru & scripts if present
        if let Some(recipe) = ctx.recipe {
            // recipe pass-thru commands first
            let recipe = RecipeKinds::select(recipe);
            let recipe_pass_thru = recipe.pass_thru_commands();
            if !recipe_pass_thru.is_empty() {
                help_text.push(format!(
                    "PASS THRU COMMANDS:\n{}",
                    two_col(recipe_pass_thru)
                ));
            }
            // custom scripts come next
            let script_pairs: Vec<(String, String)> =
                ctx.scripts.clone().map_or(vec![], |s| s.pairs());
            if !script_pairs.is_empty() {
                let script_help_lines = format!("PROJECT COMMANDS\n{}", two_col(script_pairs));
                help_text.push(script_help_lines);
            }
        }

        let both = help_text.join("\n\n");
        let with_link = format!("{}{}", both, CLIHelp::DOC_LINK);

        // append recipe subcommands
        let app = cli.app.after_help(&with_link[..]);

        // flatten the subcommands
        let combined_subcommands = collect_apps(&ctx);

        // append internal subcommands
        let app = append_subcommands(app, combined_subcommands, 20);

        CLIOutput::from_ctx(&app.clone().get_matches_from(input_args), &ctx, input)
    }
    pub fn from_ctx(
        matches: &ArgMatches,
        ctx: &Context,
        input: CLIInput,
    ) -> Result<CLIOutput, failure::Error> {
        let mut ctx = ctx.clone();

        // Overrides because of CLI flags
        let overrides = CLIOutput::matches_to_context_overrides(&matches, input);

        // Now merge the base context (file or default) with any CLI overrides
        {
            ctx.merge(overrides);
        };

        let recipe_subcommands = ctx
            .recipe
            .map(RecipeKinds::select)
            .map(|r| r.subcommands())
            .unwrap_or_else(|| vec![]);

        let internal_cmd = internal_commands()
            .iter()
            .chain(collect_recipe_global_commands().iter())
            .chain(recipe_subcommands.iter())
            .find(|cmd| matches.is_present(cmd.name()))
            .and_then(|cmd| cmd.exec(matches.subcommand_matches(cmd.name()), &ctx));

        let output_tasks = internal_cmd.or_else(|| {
            CLIOutput::get_recipe_pass_thru(&matches, &ctx).or_else(|| match ctx.recipe {
                Some(..) => CLIOutput::get_project_tasks(&matches, &ctx),
                None => None,
            })
        });

        Ok(CLIOutput {
            ctx,
            tasks: output_tasks,
        })
    }

    pub fn matches_to_context_overrides(
        matches: &clap::ArgMatches,
        input: CLIInput,
    ) -> ContextOverrides {
        // cli-provided CWD overrides file-context
        let cwd = match matches.value_of("cwd").map(PathBuf::from) {
            Some(p) => p,
            _ => input.cwd.clone(),
        };

        // run-mode is always Exec unless 'dryrun' is given on CLI
        let run_mode = if !matches.is_present("dryrun") {
            RunMode::Exec
        } else {
            RunMode::DryRun
        };

        let name = Context::get_context_name(&cwd);

        let debug = matches.is_present("debug");

        ContextOverrides {
            cwd,
            run_mode,
            name,
            term: input.term,
            pv: input.pv,
            debug,
            uid: input.uid,
            gid: input.gid,
        }
    }

    ///
    /// Try to access a matching project command
    ///
    pub fn get_project_tasks(matches: &ArgMatches, ctx: &Context) -> Option<Vec<Task>> {
        let name = matches.subcommand_name()?;
        let matching_script = ctx.scripts.as_ref()?.0.get(name)?;
        let desc = matching_script.description.clone();
        let steps = Script::flatten(
            &matching_script.steps,
            name,
            ctx.scripts.as_ref()?,
            &[name.to_string()],
        );

        if steps.is_err() {
            match steps {
                Err(e) => return Some(vec![Task::notify_error(e)]),
                _ => unreachable!(),
            }
        }

        let recipe = RecipeKinds::select(ctx.recipe?);

        let flattened = Script {
            description: desc,
            steps: steps.unwrap(),
        };

        recipe.resolve_script(&ctx, &flattened)
    }

    pub fn get_recipe_pass_thru(matches: &ArgMatches, ctx: &Context) -> Option<Vec<Task>> {
        let recipe = RecipeKinds::select(ctx.recipe?);
        //
        // Get the task list by checking which sub-command was used
        //
        let cmd = match matches.subcommand() {
            (cmd, Some(sub_matches)) => recipe.select_command((cmd, Some(sub_matches))),
            _ => None,
        };

        cmd.and_then(|cmd| recipe.resolve_cmd(&ctx, cmd))
    }
}

fn collect_apps(ctx: &Context) -> Vec<App> {
    // All current recipe sub-commands
    // like `up`, `stop` etc
    let recipe_subcommands = ctx
        .recipe
        .map(RecipeKinds::select)
        .map(|r| r.subcommands())
        .unwrap_or_else(|| vec![]);

    // this is all of the individual recipe 'global' commands
    // global meaning that the recipe can register a command
    // that doesn't need to run in an actual project
    let recipe_global = collect_recipe_global_commands();

    // flatten the subcommands
    internal_commands()
        .iter()
        .chain(recipe_global.iter())
        .chain(recipe_subcommands.iter())
        .fold(vec![], |mut acc, item| {
            acc.extend(item.subcommands(&ctx));
            acc
        })
}

fn collect_recipe_global_commands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    // all available recipes
    let recipes = available_recipes();

    // this is all of the individual recipe 'global' commands
    recipes.iter().fold(vec![], |mut acc, r| {
        acc.extend(r.global_subcommands());
        acc
    })
}

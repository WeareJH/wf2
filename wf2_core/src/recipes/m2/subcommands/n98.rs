
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::task::Task;

pub struct N98PassThru;

impl N98PassThru {
    pub const ABOUT: &'static str = "[m2] Execute n98-magerun2 commands inside the PHP container";
}

pub fn n98(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
    PhpService::select(&ctx)
        .map(|service| {
            let full_command = format!(
                r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} n98-magerun2 {trailing_args}"#,
                width = ctx.term.width,
                height = ctx.term.height,
                container_name = service.container_name,
                trailing_args = trailing
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            vec![Task::simple_command(full_command)]
        })
        .unwrap_or_else(Task::task_err_vec)
}

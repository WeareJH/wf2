use crate::{
    context::Context,
    dc_tasks::DcTasks,
    file::File,
    recipes::m2::{
        m2_runtime_env_file::M2RuntimeEnvFile,
        m2_vars::{M2Vars, NGINX_OUTPUT_FILE, TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE},
        M2Templates,
    },
    task::Task,
};

///
/// Write all files & replace all variables so it's ready to use
///
pub fn exec(
    ctx: &Context,
    runtime_env: &M2RuntimeEnvFile,
    _vars: &M2Vars,
    templates: M2Templates,
    dc: DcTasks,
) -> Vec<Task> {
    vec![
        Task::file_write(
            runtime_env.file_path(),
            "Writes the .env file to disk",
            runtime_env.bytes(),
        ),
        Task::file_write(
            ctx.cwd.join(&ctx.file_prefix).join(UNISON_OUTPUT_FILE),
            "Writes the unison file",
            templates.unison.bytes,
        ),
        Task::file_write(
            ctx.cwd.join(&ctx.file_prefix).join(TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            templates.traefik.bytes,
        ),
        Task::file_write(
            ctx.cwd.join(&ctx.file_prefix).join(NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            templates.nginx.bytes,
        ),
        dc.eject(),
    ]
}

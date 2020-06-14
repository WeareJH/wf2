use crate::context::Context;

use crate::file::File;
use crate::recipes::m2::m2_vars::M2Vars;

use crate::dc_tasks::DcTasksTrait;
use crate::recipes::m2::output_files::m2_runtime_env_file::M2RuntimeEnvFile;
use crate::recipes::m2::M2Recipe;
use crate::scripts::script::{ResolveScript, Script};
use crate::task::Task;
use crate::util::path_buf_to_string;

impl ResolveScript for M2Recipe {
    fn resolve_script(&self, ctx: &Context, script: &Script) -> Option<Vec<Task>> {
        if Script::has_dc_tasks(&script.steps) {
            let _vars = M2Vars::from_ctx(&ctx).ok()?;
            let (dc, dc_tasks) = (M2Recipe).dc_and_tasks(&ctx).ok()?;

            let recipes_services = dc.service_names();
            let script_refs = Script::service_names(&script.steps);

            if let (Some(allowed), Some(script_refs)) = (recipes_services, script_refs) {
                let missing: Vec<String> = script_refs
                    .iter()
                    .filter(|item| !allowed.contains(item))
                    .map(String::from)
                    .collect();

                if !missing.is_empty() {
                    use ansi_term::Colour::{Cyan, Red};
                    let error = format!(
                        "You tried to use the following service(s) in \
                        your wf2 file - \nbut they don't exist in this recipe\n\n    {}
                        ",
                        Red.paint(missing.join(", "))
                    );
                    let advise = format!(
                        "The following names are all valid though\n\n    {}",
                        Cyan.paint(allowed.join("\n    "))
                    );
                    return Some(vec![Task::notify_error(vec![error, advise].join("\n"))]);
                }
            }

            let script = script.set_dc_file(path_buf_to_string(&dc_tasks.file));
            let script_tasks: Vec<Task> = script.into();
            let additional_dc_tasks = vec![
                dc_tasks.write_task(),
                M2RuntimeEnvFile::from_ctx(&ctx).ok()?.write_task(),
            ]
            .into_iter();
            Some(
                additional_dc_tasks
                    .chain(script_tasks.into_iter())
                    .collect(),
            )
        } else {
            let ts: Vec<Task> = script.clone().into();
            Some(ts)
        }
    }
}

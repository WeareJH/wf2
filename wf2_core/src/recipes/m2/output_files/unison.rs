use crate::context::Context;
use crate::file::File;
use crate::recipes::m2::services::M2RecipeOptions;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UnisonFile {
    file_path: PathBuf,
    ignore_not: Option<Vec<PathBuf>>,
}

impl File<UnisonFile> for UnisonFile {
    const DESCRIPTION: &'static str = "Writes the unison file";
    const HOST_OUTPUT_PATH: &'static str = "unison/conf/sync.prf";

    fn from_ctx(ctx: &Context) -> Result<UnisonFile, failure::Error> {
        let opts: Result<M2RecipeOptions, _> = ctx.parse_options();

        Ok(UnisonFile {
            file_path: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
            ignore_not: opts.ok().and_then(|opts| {
                opts.services
                    .and_then(|s| s.unison.and_then(|u| u.ignore_not))
            }),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        let mut lines: Vec<String> = include_str!("./sync.prf")
            .lines()
            .map(String::from)
            .collect();
        if let Some(ignore_not) = self.ignore_not.as_ref() {
            let extra_lines: Vec<String> = ignore_not
                .iter()
                .map(|pb| format!("ignorenot = Path {}", pb.display()))
                .collect();
            lines.extend(extra_lines);
        }
        let output: String = lines.join("\n");
        output.into_bytes()
    }
}

use clap::ArgMatches;

#[derive(Debug, Clone)]
pub enum Cmd {
    PassThrough { cmd: String, trailing: Vec<String> },
}

impl Cmd {
    pub fn select_pass_thru(input: (&str, Option<&ArgMatches>)) -> Option<Cmd> {
        match input {
            // Fall-through case. `cmd` will be the first param here,
            // so we just need to concat that + any other trailing
            //
            // eg -> `wf2 logs unison -vv`
            //      \
            //       \
            //      `docker-composer logs unison -vv`
            //
            (cmd, Some(sub_matches)) => {
                let mut args = vec![cmd];
                let ext_args: Vec<&str> = match sub_matches.values_of("") {
                    Some(trailing) => trailing.collect(),
                    None => vec![],
                };
                args.extend(ext_args);
                Some(Cmd::PassThrough {
                    cmd: cmd.to_string(),
                    trailing: args.into_iter().map(|x| x.to_string()).collect(),
                })
            }
            _ => None,
        }
    }
}

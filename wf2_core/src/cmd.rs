use clap::ArgMatches;

#[derive(Debug, Clone)]
pub struct PassThruCmd {
    pub cmd: String,
    pub trailing: Vec<String>,
}

///
/// Fall-through case. `cmd` will be the first param here,
/// so we just need to concat that + any other trailing
///
/// eg -> `wf2 logs unison -vv`
///      \
///       \
///      `docker-composer logs unison -vv`
///
impl PassThruCmd {
    pub fn select_pass_thru(input: (&str, &ArgMatches)) -> PassThruCmd {
        let (cmd, sub_matches) = input;
        let mut args = vec![cmd];
        let ext_args: Vec<&str> = match sub_matches.values_of("") {
            Some(trailing) => trailing.collect(),
            None => vec![],
        };
        args.extend(ext_args);
        PassThruCmd {
            cmd: cmd.to_string(),
            trailing: args.into_iter().map(|x| x.to_string()).collect(),
        }
    }
}

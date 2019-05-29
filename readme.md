# wf2 [![Build Status](https://travis-ci.org/WeareJH/wf2.svg?branch=master)](https://travis-ci.org/WeareJH/wf2)


## Install
`wf2` is distributed as a single binary with everything packaged inside - 
this means you *do not* need PHP or Composer installed on your machine. 

1. Download the latest version from the [releases page](https://github.com/WeareJH/wf2/releases)
2. Make the file executable: (assuming you keep it in the `Downloads` folder)

    `chmod +x ~/Downloads/wf2`
3. Now either add an alias to your `~/.zshrc` (or bash profile)

    `echo 'alias wf2="~/Downloads/wf2"' >> ~/.zshrc`
    
   or, move the program to somewhere in your existing path - but if you
   know what that even means, you don't need the instructions for it :)


## Features
```
USAGE:
    wf2 [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --dryrun     Output descriptions of the sequence of tasks, without actually executing them
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Sets the level of verbosity

OPTIONS:
        --cwd <cwd>    Sets the CWD for all docker commands
        --php <php>    choose 7.1 or 7.2 [possible values: 7.1, 7.2]

SUBCOMMANDS:
    db-dump      Dump the current database to dump.sql
    db-import    Import a DB file
    down         Take down containers & delete everything
    eject        Dump all files into the local directory for manual running
    exec         Execute commands in the PHP container
    help         Prints this message or the help of the given subcommand(s)
    m            Execute commands in the PHP container
    pull         Pull files or folders from the PHP container to the host
    stop         Take down containers & retain data
    up           Bring up containers
```

# wf2 [![Build Status](https://travis-ci.org/WeareJH/wf2.svg?branch=master)](https://travis-ci.org/WeareJH/wf2)


## Install
`wf2` is distributed as a single binary with everything packaged inside - 
this means you *do not* need PHP or Composer installed on your machine. 

1. Download the latest version from the [releases page](https://github.com/WeareJH/wf2/releases)
2. Make the file executable: (assuming you keep it in the `Downloads` folder)

    `chmod +x ~/Downloads/wf2`
3. Now either add an alias to your `~/.zshrc` (or bash profile)

    `echo 'alias wf2="~/Downloads/wf2"' >> ~/.zshrc`
    
   or, move the program to somewhere in your existing path - see instructions below if you wish to do this:

<details><summary>Instructions for adding to your path</summary>

1. Move the executable from your Downloads folder to /opt

    `sudo mv ~/Downloads/wf2 /opt`

2. **Replace** the alias you made previously in your *zshrc* or *bash_profile* with:

    `export PATH="$PATH:/opt"`

3. Use the following command to refresh any already open terminals

    `source ~/.zshrc`

4. Or for bash users

    `source ~/.bash_profile`

5. Type the following command to check all is installed OK:

    `wf2`

6. You should see the same output as below (in features):

</details>


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
        --config <config>    path to a wf2.yml config file
        --cwd <cwd>          Sets the CWD for all docker commands
        --php <php>          choose 7.1 or 7.2 [possible values: 7.1, 7.2]

SUBCOMMANDS:
    db-dump      Dump the current database to dump.sql
    db-import    Import a DB file
    doctor       Try to fix common issues with a recipe
    down         Take down containers & delete everything
    eject        Dump all files into the local directory for manual running
    exec         Execute commands in the PHP container
    help         Prints this message or the help of the given subcommand(s)
    m            Execute commands in the PHP container
    pull         Pull files or folders from the PHP container to the host
    stop         Take down containers & retain data
    up           Bring up containers
    composer     Passes thru, eg: `wf2 composer install -vvv`      
    npm          Passes thru, eg: `wf2 npm i` 
```

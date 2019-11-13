# wf2 [![Build Status](https://travis-ci.org/WeareJH/wf2.svg?branch=master)](https://travis-ci.org/WeareJH/wf2)

## Express Install

Simply run the following in your terminal

```
zsh <(curl -L https://raw.githubusercontent.com/WeareJH/wf2/master/express-install.sh) && source ~/.zshrc
```

## Install
`wf2` is distributed as a single binary with everything packaged inside -
this means you *do not* need PHP or Composer installed on your machine.

1. Download the latest version from the [releases page](https://github.com/WeareJH/wf2/releases)
2. Make the file executable: (assuming it downloaded to the `Downloads` folder)

    `chmod +x ~/Downloads/wf2`
    
3. Move the executable from your Downloads folder to /opt

    `sudo mv ~/Downloads/wf2 /opt`
    
    - If "opt" does not exist run the command below

        `sudo mkdir /opt`
    
    - Then make sure the permissions are correct on the folder
    
        `sudo chown -R $(whoami) /opt`

5. Add this to the bottom of your *zshrc* or *bash_profile*:

    `export PATH="$PATH:/opt"`

6. Use the following command to refresh any already open terminals

    `source ~/.zshrc`

7. Or for bash users

    `source ~/.bash_profile`

8. Type the following command to check all is installed OK:

    `wf2`

9. You should see the same output as below (in features):



## Features (assuming you are using `M2` recipe)

```
wf2 0.14.0

USAGE:
    wf2 [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --debug      Route all PHP requests to the container with XDEBUG
        --dryrun     Output descriptions of the sequence of tasks, without actually executing them
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Sets the level of verbosity

OPTIONS:
        --config <config>    path to a wf2.yml config file
        --cwd <cwd>          Sets the CWD for all docker commands
        --php <php>          path to a wf2.yml config file [possible values: 7.1, 7.2, 7.3]

SUBCOMMANDS:
    up               Bring up containers
    stop             Take down containers & retain data
    down             Take down containers & delete everything
    pull             Pull files or folders from the main container to the host
    push             Push files or folders into the main container
    doctor           Try to fix common issues with a recipe
    eject            Dump all files into the local directory for manual running
    update-images    Update images used in the current recipe
    db-dump          Dump the current database to dump.sql
    db-import        Import a DB file
    exec             Execute commands in the main container
    help             Prints this message or the help of the given subcommand(s)

PASS THRU COMMANDS:
    composer    [M2] Run composer commands with the correct user
    npm         [M2] Run npm commands
    dc          [M2] Run docker-compose commands
    node        [M2] Run commands in the node container
    m           [M2] Execute ./bin/magento commands inside the PHP container
```

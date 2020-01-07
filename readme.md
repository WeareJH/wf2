# wf2 ![](https://github.com/WeareJH/wf2/workflows/.github/workflows/test.yml/badge.svg)

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

## Help

For help on the commands available, run the following: 

```shell script
wf2 --help
```

`--help` is recipe specific. So if you're in a M2 project, you'll only see
commands that are relevant to M2 sites.

If you just want to explore what the the wf2 tool can do in each recipe, just use
the `--recipe` command

```shell script
# See M2 help
wf2 --recipe M2 --help

# See Wp help
wf2 --recipe Wp --help
```

## Contributing.

Before pushing any code, run the following to ensure you catch
problems before they get to CI

```shell script
bash pre-push.sh
```

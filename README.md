uribo
=====

[![uribo](https://img.shields.io/crates/v/uribo.svg)](https://crates.io/crates/uribo)
[![Documentation](https://docs.rs/uribo/badge.svg)](https://docs.rs/uribo)
[![Actions Status](https://github.com/sile/uribo/workflows/CI/badge.svg)](https://github.com/sile/uribo/actions)
![License](https://img.shields.io/crates/l/uribo)

A command line tool that executes a shell command defined in a `$PWD/(..)*/.uribo` file.

```console
// Define and run the "hello" command.
$ uribo run hello
"hello" command is not defined

$ uribo put hello -- echo "Hello World!"
$ cat .uribo
{
  "hello": {
    "command": "echo",
    "args": [
      "Hello World!"
    ]
  }
}

$ uribo run hello
Hello World!

// If the specified command is not found in the $PWD/.uribo file,
// the parent directories will be searched.
$ echo '{"ver": {"command": "uribo", "args": ["--version"]}}' > ../.uribo
$ uribo run ver
uribo 0.2.0
```

Recommended Fish shell configuration
------------------------------------

```fish
set -x URIBO_DEFAULT_CONFIG_PATH "$HOME/.uribo"

function fish_command_not_found
    if uribo find $argv[1] > /dev/null 2> /dev/null
        uribo run $argv
    else
        __fish_default_command_not_found_handler $argv[1]
    end
end
```

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
  "hello": "echo Hello World!"
}

$ uribo run hello
Hello World!

// If the specified command is not found in the $PWD/.uribo file,
// the parent directories will be searched.
$ echo '{"ver": "uribo --version"}' > ../.uribo
$ uribo run ver
uribo 0.1.0
```

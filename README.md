TOKECOUNT
=====

Simple cli wrapper around [Tokenizers](https://github.com/huggingface/tokenizers) library
to count tokens in a text.

Installation
----------------
TODO

Usage
-----
It is possible to use it in two ways:

1. Provide text files to count tokens in it as arguments:

```bash
tokecount <file1> <file2> ...
```

2. Using pipe to pass text to it:

```bash
echo "Hello world" | tokecount
```

Options
----------------

Tokecount supports the following options:

* `--identifier` or `-i`: Specify the tokenizer model to use. Default is `bert-base-uncased`.
* `--json-confi` or `-j`: Path to a JSON file with tokenizer configuration.
* `--revision`: Specify the revision of the tokenizer model to use. Default is `main`.
* `--token` or `-t`: Hugging Face token for authentication.
* `--recursive` or `-r`: Recursively count tokens in files in the provided directories excluding symbolic links.
* `--dereference-recursive` or `-R`: Recursively count tokens in files in the provided directories including symbolic
  links.
* `--include`: Specify a glob pattern to include files.
* `--exclude`: Specify a glob pattern to exclude files.
* `--exclude-dir`: Specify a glob pattern to exclude directories.
* `--verbose` or `-v`: Print token counts for each file.

Examples:

```bash
# Count tokens in a file
tokecount myfile.txt
# Count tokens in multiple files
tokecount file1.txt file2.txt
# Use a specific tokenizer model
tokecount -i gpt2 myfile.txt
# Use a JSON configuration file
tokecount -j config.json myfile.txt
# Recursively count tokens in a directory
tokecount -r mydirectory
# Recursively count tokens in a directory including symbolic links
tokecount -R mydirectory
# Show token counts for each file
tokecount -v -r mydirectory
```

Environment Variables
----------------
Tokecount supports the following environment variables:
* `TOKEN_COUNT_MODEL` - Default tokenizer model to use.
* `TOKEN_COUNT_JSON_CONFIG` - Default path to a JSON file with tokenizer configuration.

TOKENCOUNT
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
tokencount <file1> <file2> ...
```

2. Using pipe to pass text to it:

```bash
echo "Hello world" | tokencount
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
tokencount myfile.txt
# Count tokens in multiple files
tokencount file1.txt file2.txt
# Use a specific tokenizer model
tokencount -i gpt2 myfile.txt
# Use a JSON configuration file
tokencount -j config.json myfile.txt
# Recursively count tokens in a directory
tokencount -r mydirectory
# Recursively count tokens in a directory including symbolic links
tokencount -R mydirectory
# Show token counts for each file
tokencount -v -r mydirectory
# Count tokens in a file using a pipe
echo "This is a test" | tokencount -i gpt2
# Count tokens for files in a directory with specific patterns
tokencount -r mydirectory --include "*.txt" --exclude "*.log"
```

Environment Variables
----------------
Tokecount supports the following environment variables:
* `TOKEN_COUNT_MODEL` - Default tokenizer model to use.
* `TOKEN_COUNT_JSON_CONFIG` - Default path to a JSON file with tokenizer configuration.

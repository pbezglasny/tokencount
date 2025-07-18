pub mod files;

use clap::Parser;
use files::{FileContent, FileMatchConfig, get_matched_files};
use std::collections::HashMap;
use std::env;
use std::io::{IsTerminal, Read};
use tokenizers::{FromPretrainedParameters, Tokenizer};

const DEFAULT_TOKENIZER: &str = "bert-base-uncased";
const TOKEN_COUNT_MODEL_VAR: &str = "TOKEN_COUNT_MODEL";
const TOKEN_COUNT_FILE_VAR: &str = "TOKEN_COUNT_JSON_CONFIG";
const FILE_CHUNK_SIZE: usize = 20;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

/// Token count utility
/// Counts tokens in files using provided tokenizer model.
/// If no model is provided, it uses `bert-base-uncased` by default.
#[derive(Parser, Debug)]
struct Arguments {
    /// Name of tokenizer model to use, only one of identifier or file can be used
    #[arg(short, long, default_value = None)]
    identifier: Option<String>,
    /// Path to json config, only one of identifier or file can be used
    #[arg(short, long, default_value = None)]
    json_config: Option<String>,
    /// Revision of model tokenizer
    #[arg(long, default_value = "main")]
    revision: String,
    /// Huggingface token in case download tokenizer requires authentification
    #[arg(short, long, default_value = None)]
    token: Option<String>,
    /// Read all files under each directory recursively, exclude symbolic links
    #[arg(short, long, default_value_t = false)]
    recursive: bool,
    /// Read all files under each directory recursively, include symbolic links
    #[arg(short = 'R', long, default_value_t = false)]
    dereference_recursive: bool,
    /// Glob. Show count only for files that names only matched to glob pattern.
    /// If include and exclude patterns are passed, include wins.
    #[arg(long, default_value = None)]
    include: Vec<String>,
    /// Glob pattern of files to exclude
    #[arg(long, default_value = None)]
    exclude: Vec<String>,
    /// Glob pattern of directories to exclude from counting
    #[arg(long, default_value = None)]
    exclude_dir: Vec<String>,
    /// Print token count per file
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
    /// Files to count tokens
    #[arg()]
    files: Vec<String>,
}

fn read_pipe() -> String {
    let mut buffer = String::new();
    let mut stdin = std::io::stdin();
    stdin
        .read_to_string(&mut buffer)
        .expect("Error while read data from pipe");
    buffer.trim().to_string()
}

fn get_tokenizer(args: &Arguments) -> Result<Tokenizer> {
    if args.identifier.is_some() && args.json_config.is_some() {
        panic!("Both identifier and file options are presented. Only one can be used")
    }
    if let Some(model_name) = &args.identifier {
        let params = FromPretrainedParameters {
            revision: args.revision.clone(),
            user_agent: HashMap::new(),
            token: args.token.clone(),
        };
        Tokenizer::from_pretrained(model_name, Some(params))
    } else if let Some(json_config) = &args.json_config {
        Tokenizer::from_file(json_config)
    } else if let Ok(file_path) = env::var(TOKEN_COUNT_FILE_VAR) {
        Tokenizer::from_file(file_path)
    } else {
        let tokenizer_model =
            env::var(TOKEN_COUNT_MODEL_VAR).unwrap_or(DEFAULT_TOKENIZER.to_string());
        let params = FromPretrainedParameters {
            revision: args.revision.clone(),
            user_agent: HashMap::new(),
            token: args.token.clone(),
        };
        Tokenizer::from_pretrained(tokenizer_model, Some(params))
    }
}

fn main() {
    let args = Arguments::parse();
    let tokenizer = get_tokenizer(&args).expect("Failed to initialize tokenizer");
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        // Standard use
        let config = FileMatchConfig::new(
            args.recursive || args.dereference_recursive,
            args.dereference_recursive,
            args.include,
            args.exclude,
            args.exclude_dir,
        );
        let mut token_count: u64 = 0;
        for file_chunk in get_matched_files(args.files, config).chunks(FILE_CHUNK_SIZE) {
            let file_contents: Vec<FileContent> = file_chunk
                .iter()
                .map(|file| FileContent::new(file.clone()))
                .filter(|file| file.is_text_file())
                .collect();
            let files_names: Vec<String> = file_contents
                .iter()
                .map(|file| file.get_path_string())
                .collect();
            let data: Vec<String> = file_contents
                .iter()
                .map(|file| file.read_content())
                .collect();
            let lengths: Vec<usize> = tokenizer
                .encode_batch(data, false)
                .map(|vec| vec.iter().map(|enc| enc.len()).collect())
                .expect("Error while encoding text");
            if args.verbose {
                for (file_name, length) in files_names.iter().zip(lengths.iter()) {
                    println!("{file_name} {length}");
                }
            } else {
                for length in lengths {
                    token_count += length as u64;
                }
            }
        }
        if !args.verbose {
            println!("{token_count}");
        }
    } else {
        // Pipe
        let data = read_pipe();
        let token_count = tokenizer
            .encode(data, false)
            .map(|enc| enc.len())
            .expect("Error while encoding text");
        let result = if args.verbose {
            format!(". {token_count}")
        } else {
            format!("{token_count}")
        };
        println!("{result}");
    }
}

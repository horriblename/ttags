use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli<'a>() -> App<'a, 'a> {
    App::new("ttags")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(files_arg())
        .arg(tag_file_arg())
        .arg(relative_arg())
        .arg(append_arg())
        .arg(parser_arg())
        .arg(queries_arg())
        .arg(extension_arg())
        .subcommand(lsp_subcommand())
}

fn parser_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("parser")
        .short("p")
        .long("parser")
        .value_name("FILE")
        .takes_value(true)
        .required(false)
        .help("Path to tree-sitter parser shared library (.so/.dll)")
}

fn queries_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("queries")
        .short("q")
        .long("queries")
        .value_name("FILE")
        .takes_value(true)
        .required(false)
        .help("Path to custom tags query file (.scm)")
}

fn extension_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("extension")
        .short("e")
        .long("extension")
        .value_name("EXT=FILETYPE")
        .takes_value(true)
        .required(false)
        .help("File extension mapping (e.g., -e py=python uses .py files and looks for tree_sitter_python)")
}

fn files_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("files")
        .multiple(true)
        .help("Specify files to parse for tags")
        .required(true)
}

fn tag_file_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("tag_file")
        .short("f")
        .long("tag-file")
        .value_name("FILE|-")
        .takes_value(true)
        .required(false)
        .help("File to write tags to. Use '-' to output to stdout")
        .default_value("./tags")
}

fn relative_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("relative")
        .long("relative")
        .short("r")
        .takes_value(false)
        .help("Should paths be relative to cwd? By default, it's relative to tag-file")
}

fn append_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("append")
        .short("a")
        .long("append")
        .takes_value(false)
        .help("Append tags to existing file")
}

fn lsp_subcommand<'a>() -> App<'a, 'a> {
    SubCommand::with_name("lsp")
        .about("Creates lsp server")
        .setting(AppSettings::DisableVersion)
}

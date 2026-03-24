use clap::ArgMatches;
use pathdiff::diff_paths;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use sugar_path::SugarPath;
use walkdir::WalkDir;

use crate::cli;
use crate::tag::Tag;

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Config {
    pub runtime_paths: Vec<PathBuf>,
    pub filetype_mapping: HashMap<String, String>,
    pub files: Vec<String>,
    pub tag_path: String,
    pub relative_path: String,
    pub append: bool,
    pub lsp: bool,
    pub custom_parser: Option<String>,
    pub custom_queries: Option<String>,
    pub custom_extension: Option<String>,
    pub custom_filetype: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        let matches = cli::build_cli().get_matches();

        let lsp = matches.subcommand_name() == Some("lsp");
        let files = Self::fetch_files(&matches, lsp);
        let tag_path = Self::path_to_string(Self::fetch_tag_file(&matches));
        let relative_path = Self::path_to_string(Self::fetch_relative_path(&matches));
        let append = matches.is_present("append") || lsp;
        let custom_parser = matches.value_of("parser").map(String::from);
        let custom_queries = matches.value_of("queries").map(String::from);

        let runtime_paths: Vec<PathBuf> = env::var("TT_PATH")
            .map(|p| {
                p.split(':')
                    .map(|s| Path::new(s).normalize().resolve())
                    .collect()
            })
            .unwrap_or_default();

        let mut filetype_mapping: HashMap<String, String> = HashMap::new();
        for base_path in &runtime_paths {
            let filepath = base_path.join("filetypes.txt");
            if let Ok(contents) = std::fs::read_to_string(&filepath) {
                for line in contents.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((ext, ft)) = line.split_once('=') {
                        filetype_mapping
                            .entry(ext.to_string())
                            .or_insert(ft.to_lowercase());
                    } else {
                        // TODO: color
                        eprintln!("Warning: malformed line in {:?}:{}", filepath, line);
                    }
                }
            }
        }

        let (custom_extension, custom_filetype) = match matches.value_of("extension") {
            Some(ext) => {
                let parts: Vec<&str> = ext.splitn(2, '=').collect();
                if parts.len() == 2 {
                    (Some(parts[0].to_string()), Some(parts[1].to_string()))
                } else {
                    todo!("error");
                }
            }
            None => todo!("error"),
        };

        Self {
            files,
            tag_path,
            relative_path,
            append,
            lsp,
            custom_parser,
            custom_queries,
            custom_extension,
            custom_filetype,
            runtime_paths,
            filetype_mapping,
        }
    }

    pub fn path_relative_to_file(&self, filename: &str) -> String {
        Self::path_to_string(diff_paths(filename, &self.relative_path).unwrap())
    }

    pub fn output(&self) -> BufWriter<Box<dyn Write>> {
        BufWriter::new(if self.going_to_stdout() {
            Box::new(io::stdout())
        } else {
            let mut options = OpenOptions::new();
            let options = options.write(true).read(true).create(true);

            Box::new(options.open(&self.tag_path).unwrap())
        })
    }

    pub fn appending(&self) -> bool {
        self.append && !self.going_to_stdout()
    }

    pub fn clear_tag_file(&self) {
        if !self.going_to_stdout() {
            File::create(&self.tag_path).expect("Failed clearing file");
        }
    }

    pub fn current_tag_contents(&self) -> Vec<Tag> {
        let file = File::open(&self.tag_path).expect("Failed reading tags file");
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents).unwrap();

        contents
            .trim()
            .split('\n')
            .filter(|line| !line.starts_with("!_") && !line.is_empty())
            .map(Tag::parse)
            .collect()
    }

    pub fn going_to_stdout(&self) -> bool {
        Path::new(&self.tag_path).file_name() == Some(OsStr::new("-"))
    }

    fn fetch_files(matches: &ArgMatches<'_>, lsp: bool) -> Vec<String> {
        if lsp {
            vec![]
        } else {
            matches
                .values_of("files")
                .unwrap()
                .flat_map(|f| {
                    let path = Path::new(&f).resolve();

                    if path.is_dir() {
                        WalkDir::new(path)
                            .into_iter()
                            .filter(|entry| entry.as_ref().unwrap().path().is_file())
                            .map(|entry| Self::path_to_string(entry.unwrap().path().to_path_buf()))
                            .collect()
                    } else {
                        vec![Self::path_to_string(path)]
                    }
                })
                .collect()
        }
    }

    fn fetch_tag_file(matches: &ArgMatches<'_>) -> PathBuf {
        Path::new(matches.value_of("tag_file").unwrap()).resolve()
    }

    fn fetch_relative_path(matches: &ArgMatches<'_>) -> PathBuf {
        let tag_file = Self::fetch_tag_file(matches);

        if !matches.is_present("relative")
            && tag_file.as_path().file_name() != Some(OsStr::new("-"))
        {
            tag_file.parent().unwrap().to_path_buf()
        } else {
            env::current_dir().unwrap()
        }
    }

    fn path_to_string(path: PathBuf) -> String {
        path.into_os_string().into_string().unwrap()
    }
}

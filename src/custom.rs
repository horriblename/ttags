use libloading::{Library, Symbol};
use log::debug;
use npezza93_tree_sitter_tags::{TagsConfiguration, TagsContext};
use std::path::{Path, PathBuf};
use std::str;
use std::{fs, io};

use crate::tag::Tag;

type LanguageFn = extern "C" fn() -> *mut tree_sitter::ffi::TSLanguage;

pub struct CustomConfig {
    pub tags_config: TagsConfiguration,
    pub extension: String,
    _library: Library, // prevent library being dropped in config()
}

#[cfg(target_os = "windows")]
const PARSER_EXT: &str = ".dll";
#[cfg(target_os = "macos")]
const PARSER_EXT: &str = ".dylib";
#[cfg(target_os = "linux")]
const PARSER_EXT: &str = ".so";

#[derive(Debug)]
pub enum Error {
    MissingParser,
    MissingQuery,
    FailLoadParser(PathBuf, libloading::Error),
    FailReadQuery(PathBuf, io::Error),
    FailTag(npezza93_tree_sitter_tags::Error),
}

fn find_runtime_file(runtime_paths: &[PathBuf], rel_path: &Path) -> Option<PathBuf> {
    runtime_paths.iter().find_map(|base_path| {
        let path = base_path.join(rel_path);
        if path.exists() { Some(path) } else { None }
    })
}

pub fn find_query_and_parser(
    filetype: &str,
    extension: &str,
    runtime_paths: &[PathBuf],
) -> Result<(CustomConfig, String), Error> {
    let Some(query_path) = find_runtime_file(
        runtime_paths,
        &Path::new("queries").join(filetype).join("tags.scm"),
    ) else {
        return Err(Error::MissingQuery);
    };
    debug!(
        "custom filetype {} found query file path: {:?}",
        filetype, query_path
    );

    let Some(parser_path) = find_runtime_file(
        runtime_paths,
        &Path::new("parser").join(format!("{}{}", filetype, PARSER_EXT)),
    ) else {
        return Err(Error::MissingParser);
    };
    debug!(
        "custom filetype {} found parser: {:?}",
        filetype, parser_path
    );

    let config = config(parser_path, query_path, &extension, filetype)?;

    Ok((config, extension.to_string()))
}

pub fn config(
    parser_path: PathBuf,
    queries_path: PathBuf,
    extension: &str,
    filetype: &str,
) -> Result<CustomConfig, Error> {
    let library = unsafe {
        match Library::new(&parser_path) {
            Ok(library) => library,
            Err(err) => return Err(Error::FailLoadParser(parser_path, err)),
        }
    };

    let func_name = format!("tree_sitter_{}", filetype);
    let language_fn: Symbol<LanguageFn> = unsafe {
        match library.get(func_name.as_bytes()) {
            Ok(f) => f,
            Err(err) => return Err(Error::FailLoadParser(parser_path, err)),
        }
    };

    let language_ptr = language_fn();
    let language = unsafe { tree_sitter::Language::from_raw(language_ptr) };

    let queries = match fs::read_to_string(&queries_path) {
        Ok(q) => q,
        Err(err) => return Err(Error::FailReadQuery(queries_path, err)),
    };

    let tags_config =
        TagsConfiguration::new(language, queries.as_str(), "").map_err(Error::FailTag)?;

    Ok(CustomConfig {
        tags_config,
        extension: extension.to_string(),
        _library: library,
    })
}

pub fn generate_tags_custom<'a>(
    context: &'a mut TagsContext,
    config: &'a TagsConfiguration,
    filename: &'a str,
    contents: &'a [u8],
) -> Vec<Tag> {
    let tags = context.generate_tags(config, contents, None).unwrap().0;

    tags.flat_map(|tag| {
        let tag = tag.unwrap();
        let node_name = config.syntax_type_name(tag.syntax_type_id);
        let tag_name = &contents[tag.name_range.start..tag.name_range.end];
        let original_name = str::from_utf8(<&[u8]>::clone(&tag_name)).unwrap_or("");
        let row = tag.span.start.row;

        vec![Tag::new(original_name, filename, row + 1, node_name)]
    })
    .collect::<Vec<Tag>>()
}

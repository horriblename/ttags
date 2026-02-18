use libloading::{Library, Symbol};
use npezza93_tree_sitter_tags::{TagsConfiguration, TagsContext};
use std::fs;
use std::str;

use crate::tag::Tag;

type LanguageFn = extern "C" fn() -> *mut tree_sitter::ffi::TSLanguage;

pub struct CustomConfig {
    pub tags_config: TagsConfiguration,
    pub extension: String,
    _library: Library, // prevent library being dropped in config()
}

pub fn config(parser_path: &str, queries_path: &str, extension: &str, filetype: &str) -> CustomConfig {
    let library = unsafe { Library::new(parser_path).expect("Failed to load parser library") };

    let func_name = format!("tree_sitter_{}", filetype);
    let language_fn: Symbol<LanguageFn> = unsafe {
        library.get(func_name.as_bytes()).expect(&format!("TODO: Failed to get {} function", func_name))
    };

    let language_ptr = language_fn();
    let language = unsafe { tree_sitter::Language::from_raw(language_ptr) };

    let queries = fs::read_to_string(queries_path).expect("TODO: Failed to read queries file");

    let tags_config = TagsConfiguration::new(
        language,
        queries.as_str(),
        "",
    ).expect("TODO: Failed to create tags configuration");

    CustomConfig {
        tags_config,
        extension: extension.to_string(),
        _library: library,
    }
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

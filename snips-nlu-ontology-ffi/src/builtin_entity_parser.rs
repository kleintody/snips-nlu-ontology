use std::ffi::CStr;
use std::slice;
use std::sync::Arc;
use std::str::FromStr;

use libc;

use ffi_utils::CStringArray;
use errors::*;
use snips_nlu_ontology::*;
use builtin_entity::*;
use ffi_utils::*;

#[repr(C)]
pub struct CBuiltinEntityParser {
    pub parser: *const libc::c_void,
}

macro_rules! get_parser {
    ($opaque:ident) => {{
        let container: &CBuiltinEntityParser = unsafe { &*$opaque };
        let x = container.parser as *const ::BuiltinEntityParser;
        unsafe { &*x }
    }};
}

macro_rules! get_parser_mut {
    ($opaque:ident) => {{
        let container: &CBuiltinEntityParser = unsafe { &*$opaque };
        let x = container.parser as *mut ::BuiltinEntityParser;
        unsafe { &mut *x }
    }};
}


#[no_mangle]
pub extern "C" fn nlu_ontology_create_builtin_entity_parser(
    ptr: *mut *const CBuiltinEntityParser,
    lang: *const libc::c_char,
) -> CResult {
    wrap!(create_builtin_entity_parser(ptr, lang))
}

#[no_mangle]
pub extern "C" fn nlu_ontology_extract_entities(
    ptr: *const CBuiltinEntityParser,
    sentence: *const libc::c_char,
    filter_entity_kinds: *const CStringArray,
    results: *mut *const CBuiltinEntityArray,
) -> CResult {
    wrap!(extract_entity(ptr, sentence, filter_entity_kinds, results))
}

#[no_mangle]
pub extern "C" fn nlu_ontology_destroy_builtin_entity_parser(
    ptr: *mut CBuiltinEntityParser,
) -> CResult {
    let parser = get_parser_mut!(ptr);
    unsafe {
        let _ = Arc::from_raw(parser);
    }
    CResult::RESULT_OK
}

fn create_builtin_entity_parser(
    ptr: *mut *const CBuiltinEntityParser,
    lang: *const libc::c_char,
) -> OntologyResult<()> {
    let lang = unsafe { CStr::from_ptr(lang) }.to_str()?;
    let lang = ::Language::from_str(lang)?;
    let parser = BuiltinEntityParser::get(lang.into());

    unsafe {
        let container = CBuiltinEntityParser {
            parser: Arc::into_raw(parser) as *const libc::c_void,
        };
        *ptr = Box::into_raw(Box::new(container))
    }
    Ok(())
}

fn extract_entity(
    ptr: *const CBuiltinEntityParser,
    sentence: *const libc::c_char,
    filter_entity_kinds: *const CStringArray,
    results: *mut *const CBuiltinEntityArray,
) -> OntologyResult<()> {
    let parser = get_parser!(ptr);
    let sentence = unsafe { CStr::from_ptr(sentence) }.to_str()?;

    let opt_filters: Option<Vec<_>> = if !filter_entity_kinds.is_null() {
        let filters = unsafe {
            let array = &*filter_entity_kinds;
            slice::from_raw_parts(array.data, array.size as usize)
        }
            .into_iter()
            .map(|&ptr| Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?)
                .and_then(|s| ::BuiltinEntityKind::from_identifier(s).chain_err(|| format!("`{}` isn't a known builtin entity kind", s))))
            .collect::<OntologyResult<Vec<_>>>()?;
        Some(filters)
    } else {
        None
    };
    let opt_filters = opt_filters.as_ref().map(|vec| vec.as_slice());

    let c_entities = parser.extract_entities(sentence, opt_filters)
        .into_iter()
        .map(CBuiltinEntity::from)
        .collect::<Vec<CBuiltinEntity>>();
    let c_entities = Box::new(CBuiltinEntityArray::from(c_entities));

    unsafe {
        *results = Box::into_raw(c_entities);
    }

    Ok(())
}


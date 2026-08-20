use libvctrl::{Decoder, EntryKind, Hash, ObjectStore, VctrlError};
use std::fmt::Write;
use std::io::{BufRead, Write as IoWrite};

#[derive(Clone, Copy)]
pub enum CatFileMode {
    PrettyPrint,

    ObjectType,

    ObjectSize,

    Exists,

    Raw(ObjectType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Blob,

    Tree,

    Commit,

    Tag,
}

pub fn cat_file<D: Decoder>(
    store: &dyn ObjectStore,
    decoder: &D,
    object_name: &str,
    mode: CatFileMode,
    writer: &mut impl IoWrite,
) -> Result<(), VctrlError> {
    let hash = parse_hash(object_name)?;

    let mut encoded = Vec::new();
    store
        .get(&hash)?
        .read_to_end(&mut encoded)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;

    match mode {
        CatFileMode::Exists => Ok(()),
        CatFileMode::ObjectType => {
            let obj_type = decode_type(decoder, &encoded)?;
            let type_str = obj_type_to_str(obj_type);
            writeln!(writer, "{type_str}")
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
        CatFileMode::ObjectSize => {
            let _obj_type = decode_type(decoder, &encoded)?;
            let size = encoded.len();
            writeln!(writer, "{size}").map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
        CatFileMode::PrettyPrint => {
            let content = pretty_print(decoder, &encoded)?;
            writer
                .write_all(content.as_bytes())
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
        CatFileMode::Raw(expected_type) => {
            let actual_type = decode_type(decoder, &encoded)?;
            if actual_type != expected_type {
                let actual_type_str = obj_type_to_str(actual_type);
                let expected_type_str = obj_type_to_str(expected_type);
                return Err(VctrlError::Other(format!(
                    "object {object_name} is a {actual_type_str}, not a {expected_type_str}"
                )));
            }
            writer
                .write_all(&encoded)
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct BatchOptions {
    pub format: Option<String>,

    pub nul_terminated: bool,

    pub follow_symlinks: bool,

    pub buffer: bool,

    pub print_contents: bool,
}

pub fn cat_file_batch<D: Decoder>(
    store: &dyn ObjectStore,
    decoder: &D,
    input: &mut impl BufRead,
    output: &mut impl IoWrite,
    options: &BatchOptions,
) -> Result<(), VctrlError> {
    let delimiter: u8 = if options.nul_terminated { 0 } else { b'\n' };

    let mut line = String::new();
    let mut out_buf: Vec<u8> = Vec::new();

    loop {
        line.clear();
        if input
            .read_line(&mut line)
            .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?
            == 0
        {
            break;
        }
        let trimmed = if options.nul_terminated {
            line.trim_end_matches('\0')
        } else {
            line.trim_end()
        };

        let object_name = trimmed;

        if let Ok((info, content)) = handle_one_object(store, decoder, object_name, options) {
            out_buf.extend_from_slice(info.as_bytes());
            out_buf.push(delimiter);
            if let Some(c) = content {
                out_buf.extend_from_slice(&c);
                if options.nul_terminated {
                    out_buf.push(0);
                } else {
                    out_buf.push(b'\n');
                }
            }
            if !options.buffer {
                output
                    .write_all(&out_buf)
                    .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
                out_buf.clear();
            }
        } else {
            let error_line = format!("{object_name} missing");
            out_buf.extend_from_slice(error_line.as_bytes());
            out_buf.push(delimiter);
            if !options.buffer {
                output
                    .write_all(&out_buf)
                    .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
                out_buf.clear();
            }
        }
    }

    if !out_buf.is_empty() {
        output
            .write_all(&out_buf)
            .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
    }
    Ok(())
}

fn handle_one_object<D: Decoder>(
    store: &dyn ObjectStore,
    decoder: &D,
    object_name: &str,
    options: &BatchOptions,
) -> Result<(String, Option<Vec<u8>>), VctrlError> {
    let hash = parse_hash(object_name)?;

    let mut encoded = Vec::new();
    store
        .get(&hash)?
        .read_to_end(&mut encoded)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;

    let obj_type = decode_type(decoder, &encoded)?;
    let obj_size = encoded.len() as u64;

    let info = options.format.as_ref().map_or_else(
        || {
            let type_str = obj_type_to_str(obj_type);
            format!("{hash} {type_str} {obj_size}")
        },
        |fmt| format_batch_info(fmt, &hash, obj_type, obj_size, None),
    );

    let content = if options.print_contents {
        Some(pretty_print(decoder, &encoded)?.into_bytes())
    } else {
        None
    };

    Ok((info, content))
}

fn parse_hash(s: &str) -> Result<Hash, VctrlError> {
    if s.len() != 128 {
        let actual_len = s.len();
        return Err(VctrlError::Other(format!(
            "invalid hash length: {actual_len} (expected 128)"
        )));
    }
    let mut bytes = [0u8; 64];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let hex_byte = &s[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(hex_byte, 16)
            .map_err(|e| VctrlError::Other(format!("invalid hex character in hash: {e}")))?;
    }
    Hash::from_bytes(&bytes)
}

fn decode_type<D: Decoder>(decoder: &D, encoded: &[u8]) -> Result<ObjectType, VctrlError> {
    if decoder.decode_blob(encoded).is_ok() {
        return Ok(ObjectType::Blob);
    }
    if decoder.decode_tree(encoded).is_ok() {
        return Ok(ObjectType::Tree);
    }
    if decoder.decode_commit(encoded).is_ok() {
        return Ok(ObjectType::Commit);
    }
    if decoder.decode_tag(encoded).is_ok() {
        return Ok(ObjectType::Tag);
    }
    Err(VctrlError::CorruptedData("unknown object type".into()))
}

fn pretty_print<D: Decoder>(decoder: &D, encoded: &[u8]) -> Result<String, VctrlError> {
    if let Ok(blob) = decoder.decode_blob(encoded) {
        return Ok(String::from_utf8_lossy(blob.data()).to_string());
    }
    if let Ok(tree) = decoder.decode_tree(encoded) {
        let mut out = String::new();
        for entry in tree.entries() {
            let _ = writeln!(
                &mut out,
                "{:06o} {:?} {} {}",
                entry_mode(entry.kind()),
                entry.kind(),
                entry.hash(),
                entry.name()
            );
        }
        return Ok(out);
    }
    if let Ok(commit) = decoder.decode_commit(encoded) {
        let mut out = String::new();
        let _ = writeln!(&mut out, "tree {}", commit.tree());
        for parent in commit.parents() {
            let _ = writeln!(&mut out, "parent {parent}");
        }
        let _ = writeln!(
            &mut out,
            "author {} <{}>",
            commit.author().name(),
            commit.author().email()
        );
        let _ = writeln!(
            &mut out,
            "committer {} <{}>",
            commit.committer().name(),
            commit.committer().email()
        );
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "{}", commit.message());
        return Ok(out);
    }
    if let Ok(tag) = decoder.decode_tag(encoded) {
        let mut out = String::new();
        let _ = writeln!(&mut out, "object {}", tag.target());
        let _ = writeln!(&mut out, "type commit");
        let _ = writeln!(&mut out, "tag {}", tag.name());
        if let Some(tagger) = tag.tagger() {
            let _ = writeln!(&mut out, "tagger {} <{}>", tagger.name(), tagger.email());
        }
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "{}", tag.message());
        return Ok(out);
    }
    Err(VctrlError::CorruptedData("unknown object type".into()))
}

const fn obj_type_to_str(t: ObjectType) -> &'static str {
    match t {
        ObjectType::Blob => "blob",
        ObjectType::Tree => "tree",
        ObjectType::Commit => "commit",
        ObjectType::Tag => "tag",
    }
}

const fn entry_mode(kind: EntryKind) -> u32 {
    match kind {
        EntryKind::Blob => 0o100_644,
        EntryKind::Executable => 0o100_755,
        EntryKind::Symlink => 0o120_000,
        EntryKind::Tree => 0o040_000,
        EntryKind::Submodule => 0o160_000,
        _ => 0,
    }
}

fn format_batch_info(
    format: &str,
    hash: &Hash,
    obj_type: ObjectType,
    size: u64,
    _mode: Option<u32>,
) -> String {
    let mut output = format.to_owned();
    output = output.replace("%(objectname)", &hash.to_string());
    output = output.replace("%(objecttype)", obj_type_to_str(obj_type));
    output = output.replace("%(objectsize)", &size.to_string());
    output
}

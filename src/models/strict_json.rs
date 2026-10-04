//! Primitive and document-boundary checks around the pinned Facet JSON parser.
//!
//! Facet supports CLI-style scalar coercion and stream-oriented root parsing.
//! This event-only pass follows the model's existing Facet shape. It retains no
//! generic JSON tree or archive values, and limits recursive schema traversal.

use crate::archive::ArchiveEntryErrorCode;
use facet::Def;
use facet::Facet;
use facet::Shape;
use facet::Type;
use facet::UserType;
use facet_format::FormatParser;
use facet_format::ParseEvent;
use facet_format::ParseEventKind;
use facet_format::ScalarValue;

const MAX_SCHEMA_DEPTH: usize = 128;

pub(super) fn check<T: Facet<'static>>(bytes: &[u8]) -> Result<(), ArchiveEntryErrorCode> {
    u32::try_from(bytes.len()).map_err(|_error| ArchiveEntryErrorCode::EntryTooLarge)?;
    let mut parser = facet_json::JsonParser::<false>::new(bytes);
    let first = next_event(&mut parser)?;
    let end = check_value(&mut parser, &first, T::SHAPE, 0)?;
    // The pinned parser ends after its first root value. Its closing span lets
    // us reject any suffix beyond the four whitespace bytes JSON permits.
    require_whitespace(bytes.get(end..))?;
    Ok(())
}

fn next_event<'a>(
    parser: &mut facet_json::JsonParser<'a, false>,
) -> Result<ParseEvent<'a>, ArchiveEntryErrorCode> {
    parser
        .next_event()
        .map_err(|_error| ArchiveEntryErrorCode::InvalidJson)?
        .ok_or(ArchiveEntryErrorCode::InvalidJson)
}

fn check_value(
    parser: &mut facet_json::JsonParser<'_, false>,
    event: &ParseEvent<'_>,
    shape: &'static Shape,
    depth: usize,
) -> Result<usize, ArchiveEntryErrorCode> {
    if depth > MAX_SCHEMA_DEPTH {
        return Err(ArchiveEntryErrorCode::SchemaMismatch);
    }
    if let Def::Option(option) = shape.def {
        if matches!(event.kind, ParseEventKind::Scalar(ScalarValue::Null)) {
            return span_end(event);
        }
        return check_value(parser, event, option.t, depth + 1);
    }
    let valid_scalar = if shape.is_type::<String>() {
        matches!(event.kind, ParseEventKind::Scalar(ScalarValue::Str(_)))
    } else if shape.is_type::<bool>() {
        matches!(event.kind, ParseEventKind::Scalar(ScalarValue::Bool(_)))
    } else if shape.is_type::<u64>() {
        matches!(event.kind, ParseEventKind::Scalar(ScalarValue::U64(_)))
            || matches!(event.kind, ParseEventKind::Scalar(ScalarValue::I64(value)) if value >= 0)
    } else {
        false
    };
    if valid_scalar {
        if shape.is_type::<u64>() {
            check_integer_lexeme(parser, event)?;
        }
        return span_end(event);
    }
    match (shape.def, shape.ty, &event.kind) {
        (Def::List(list), _, ParseEventKind::SequenceStart(_)) => {
            let mut previous_end = span_end(event)?;
            loop {
                let next = next_event(parser)?;
                if matches!(next.kind, ParseEventKind::SequenceEnd) {
                    check_closing_gap(parser, previous_end, &next)?;
                    return span_end(&next);
                }
                previous_end = check_value(parser, &next, list.t, depth + 1)?;
            }
        }
        (_, Type::User(UserType::Struct(structure)), ParseEventKind::StructStart(_)) => {
            let mut previous_end = span_end(event)?;
            loop {
                let next = next_event(parser)?;
                match &next.kind {
                    ParseEventKind::StructEnd => {
                        check_closing_gap(parser, previous_end, &next)?;
                        return span_end(&next);
                    }
                    ParseEventKind::FieldKey(key) => {
                        let name = key.name().ok_or(ArchiveEntryErrorCode::SchemaMismatch)?;
                        let field = structure
                            .fields
                            .iter()
                            .find(|field| field.name == name.as_ref())
                            .ok_or(ArchiveEntryErrorCode::SchemaMismatch)?;
                        let value = next_event(parser)?;
                        previous_end = check_value(parser, &value, field.shape.get(), depth + 1)?;
                    }
                    _ => return Err(ArchiveEntryErrorCode::SchemaMismatch),
                }
            }
        }
        _ => Err(ArchiveEntryErrorCode::SchemaMismatch),
    }
}

fn span_end(event: &ParseEvent<'_>) -> Result<usize, ArchiveEntryErrorCode> {
    let end = event
        .span
        .offset
        .checked_add(event.span.len)
        .ok_or(ArchiveEntryErrorCode::InvalidJson)?;
    end.try_into()
        .map_err(|_error| ArchiveEntryErrorCode::InvalidJson)
}

fn check_closing_gap(
    parser: &facet_json::JsonParser<'_, false>,
    previous_end: usize,
    closing: &ParseEvent<'_>,
) -> Result<(), ArchiveEntryErrorCode> {
    let closing_start = usize::try_from(closing.span.offset)
        .map_err(|_error| ArchiveEntryErrorCode::InvalidJson)?;
    // A separator after the final member is a trailing comma.
    require_whitespace(
        parser
            .input()
            .and_then(|bytes| bytes.get(previous_end..closing_start)),
    )
}

fn require_whitespace(bytes: Option<&[u8]>) -> Result<(), ArchiveEntryErrorCode> {
    if bytes.is_some_and(|bytes| {
        bytes
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
    }) {
        Ok(())
    } else {
        Err(ArchiveEntryErrorCode::InvalidJson)
    }
}

fn check_integer_lexeme(
    parser: &facet_json::JsonParser<'_, false>,
    event: &ParseEvent<'_>,
) -> Result<(), ArchiveEntryErrorCode> {
    let start =
        usize::try_from(event.span.offset).map_err(|_error| ArchiveEntryErrorCode::InvalidJson)?;
    let end = span_end(event)?;
    let number = parser
        .input()
        .and_then(|bytes| bytes.get(start..end))
        .ok_or(ArchiveEntryErrorCode::InvalidJson)?;
    // Nonnegative integer values may include JSON's valid negative zero.
    let digits = number.strip_prefix(b"-").unwrap_or(number);
    if digits.is_empty()
        || !digits.iter().all(u8::is_ascii_digit)
        || (digits.len() > 1 && digits[0] == b'0')
    {
        return Err(ArchiveEntryErrorCode::InvalidJson);
    }
    Ok(())
}

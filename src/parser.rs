use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace1, space0},
    combinator::{map, verify},
    error::ErrorKind,
    multi::many1,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

#[inline]
fn key_name(input: &[u8]) -> IResult<&[u8], &[u8]> {
    verify(take_until(":"), |input: &[u8]| {
        if !input.is_empty() {
            input[0] != b'\n'
        } else {
            false
        }
    })(input)
}

#[inline]
fn separator(input: &[u8]) -> IResult<&[u8], ()> {
    map(tuple((char(':'), space0)), |_| ())(input)
}

#[inline]
fn single_line(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("\n")(input)
}

#[inline]
fn key_value(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    separated_pair(key_name, separator, single_line)(input)
}

#[inline]
fn single_package(input: &[u8]) -> IResult<&[u8], Vec<(&[u8], &[u8])>> {
    many1(terminated(key_value, tag("\n")))(input)
}

#[inline]
fn extract_name(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let info = single_package(input)?;
    for i in info.1 {
        if i.0 == &b"Package"[..] {
            return Ok((info.0, i.1));
        }
    }

    Err(nom::Err::Error(nom::error::make_error(
        info.0,
        ErrorKind::Verify,
    )))
}

#[inline]
pub fn extract_all_names(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    many1(terminated(extract_name, tag("\n")))(input)
}

// parse topic description

fn desc_title(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("Topic Description")(input)
}

fn desc_separator(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("-")(input)
}

fn desc_start(input: &[u8]) -> IResult<&[u8], ()> {
    map(
        tuple((desc_title, desc_separator, single_line, multispace1)),
        |_| (),
    )(input)
}

fn desc_end(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("\nPackage(s) Affected")(input)
}

pub fn extract_topic_description(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(desc_start, desc_end)(input)
}

// tests
#[test]
fn test_desc_title() {
    let test = &b"<!-- test title -->\nTopic Description"[..];
    assert_eq!(
        desc_title(&test),
        Ok((&b"Topic Description"[..], &b"<!-- test title -->\n"[..]))
    );
}

#[test]
fn test_desc_separator() {
    let test = &b"Topic Description\n-------\n"[..];
    assert_eq!(
        desc_separator(&test),
        Ok((&b"-------\n"[..], &b"Topic Description\n"[..]))
    );
}

#[test]
fn test_desc_start() {
    let test = &b"Topic Description\n-------\ncontent"[..];
    assert_eq!(desc_start(&test), Ok((&b"content"[..], ())));
}

#[test]
fn test_extract_topic_description() {
    let test = &b"<!-- test title -->\nTopic Description\n-------\n\ncontent\ncontent\n\nPackage(s) Affected\n"[..];
    assert_eq!(
        extract_topic_description(&test),
        Ok((&b"\nPackage(s) Affected\n"[..], &b"content\ncontent\n"[..]))
    );
}

#[test]
fn test_key_name() {
    let test = &b"name: value"[..];
    assert_eq!(key_name(&test), Ok((&b": value"[..], &b"name"[..])));
}

#[test]
fn test_seperator() {
    let test = &b": value"[..];
    let test_2 = &b": \tvalue"[..];
    assert_eq!(separator(&test), Ok((&b"value"[..], ())));
    assert_eq!(separator(&test_2), Ok((&b"value"[..], ())));
}

#[test]
fn test_single_line() {
    let test = &b"value\n"[..];
    let test_2 = &b"value\t\r\n"[..];
    let test_3 = &b"value \x23\xff\n"[..];
    assert_eq!(single_line(&test), Ok((&b"\n"[..], &b"value"[..])));
    assert_eq!(single_line(&test_2), Ok((&b"\n"[..], &b"value\t\r"[..])));
    assert_eq!(
        single_line(&test_3),
        Ok((&b"\n"[..], &b"value \x23\xff"[..]))
    );
}

#[test]
fn test_key_value() {
    let test = &b"name1: value\n"[..];
    let test_2 = &b"name2: value\t\r\n"[..];
    let test_3 = &b"name3: value \x23\xff\n"[..];
    assert_eq!(
        key_value(&test),
        Ok((&b"\n"[..], (&b"name1"[..], &b"value"[..])))
    );
    assert_eq!(
        key_value(&test_2),
        Ok((&b"\n"[..], (&b"name2"[..], &b"value\t\r"[..])))
    );
    assert_eq!(
        key_value(&test_3),
        Ok((&b"\n"[..], (&b"name3"[..], &b"value \x23\xff"[..])))
    );
}

#[test]
fn test_package() {
    let test = &b"Package: zsync\nVersion: 0.6.2-1\nSection: net\nArchitecture: amd64\nInstalled-Size: 256\n\n"[..];
    assert_eq!(
        single_package(&test),
        Ok((
            &b"\n"[..],
            vec![
                (&b"Package"[..], &b"zsync"[..]),
                (&b"Version"[..], &b"0.6.2-1"[..]),
                (&b"Section"[..], &b"net"[..]),
                (&b"Architecture"[..], &b"amd64"[..]),
                (&b"Installed-Size"[..], &b"256"[..])
            ]
        ))
    );
    assert_eq!(extract_name(&test), Ok((&b"\n"[..], (&b"zsync"[..]))));
}

#[test]
fn test_multi_package() {
    let test = &b"Package: zsync\na: b\n\nPackage: rsync\na: c\n\n"[..];
    assert_eq!(
        extract_all_names(test),
        Ok((&b""[..], vec![&b"zsync"[..], &b"rsync"[..]]))
    );
}

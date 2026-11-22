use std::io::Cursor;

use tim2::*;

fn decode_encode(data: &[u8]) -> (Info, Picture) {
    let mut decoder = Decoder::from_reader(Cursor::new(data)).unwrap();
    assert!(decoder.picture_count() == 1);
    let picture = decoder.next_picture().unwrap().unwrap();
    let image_reader = picture.image(0, None).unwrap().unwrap();
    image_reader.read_to_rgba().unwrap();

    let info = decoder.info();
    let mut new_data = Vec::with_capacity(data.len());
    {
        let mut encoder = Encoder::from_writer(&mut new_data, info).unwrap();
        encoder.write_picture(&picture).unwrap();
    }
    assert_eq!(data, new_data);

    (*info, picture)
}

#[test]
fn i16() {
    let (_, picture) = decode_encode(include_bytes!("samples/i16.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::Rgb16);
    assert_eq!(picture.metadata().clut_type, ClutType::NONE);
}

#[test]
fn i24() {
    let (_, picture) = decode_encode(include_bytes!("samples/i24.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::Rgb24);
    assert_eq!(picture.metadata().clut_type, ClutType::NONE);
}

#[test]
fn i32() {
    let (_, picture) = decode_encode(include_bytes!("samples/i32.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::Rgb32);
    assert_eq!(picture.metadata().clut_type, ClutType::NONE);
}

#[test]
fn i4c16() {
    let (_, picture) = decode_encode(include_bytes!("samples/i4c16.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex4);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB16_CSM1);
}

#[test]
fn i4c24() {
    let (_, picture) = decode_encode(include_bytes!("samples/i4c24.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex4);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB24_CSM1);
}

#[test]
fn i4c32() {
    let (_, picture) = decode_encode(include_bytes!("samples/i4c32.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex4);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB32_CSM1);
}

#[test]
fn i8c16() {
    let (_, picture) = decode_encode(include_bytes!("samples/i8c16.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex8);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB16_CSM1);
}

#[test]
fn i8c24() {
    let (_, picture) = decode_encode(include_bytes!("samples/i8c24.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex8);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB24_CSM1);
}

#[test]
fn i8c32() {
    let (_, picture) = decode_encode(include_bytes!("samples/i8c32.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex8);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB32_CSM1);
}

#[test]
fn i8c32al() {
    let (info, picture) = decode_encode(include_bytes!("samples/i8c32al.tm2"));
    assert_eq!(info.format_id, FormatId::Align128);
    assert_eq!(picture.metadata().image_type, ImageType::IdTex8);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB32_CSM1);
}

#[test]
fn i8c32cm2() {
    let (_, picture) = decode_encode(include_bytes!("samples/i8c32cm2.tm2"));
    assert_eq!(picture.metadata().image_type, ImageType::IdTex8);
    assert_eq!(picture.metadata().clut_type, ClutType::RGB32_CSM2);
}
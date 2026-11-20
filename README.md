# tim2-rs
TIM2 texture decoding and encoding library written in Rust.

## Features
- Based on the official format specifications.
- Full support for the TIM2 and CLUT2 format:
    - RGB16/24/32 and 4/8-bit indexed images.
    - RGB16/24/32 CLUTs (palettes), with or without swizzling.
    - Mipmaps*
    - 16/128 bytes alignment.
- Serde support for serializing metadata with the `serde` feature.

(*) Mipmap support relies on some assumptions made about the mipmap sizes and has not been properly tested, due to a lack of samples. Please let me know if you've had any success with mipmapped TIM2 files when using this library.

## Example
- Decoding
```rs
use tim2::Decoder;

let file = File::open(path)?;
let mut decoder = Decoder::from_reader(BufReader::new(file))?;

while let Some(picture) = decoder.next_picture()? {
    // do something with the picture...
}
```

- Encoding
```rs
use tim2::{Encoder, Picture, Info, FormatId};

let pictures: Vec<Picture>;
let info = Info {
    is_clut2: false,
    format_version: 4,
    format_id: FormatId::Align128,
    picture_count: pictures.len() as u16
};

let file = File::create(path)?;
let mut encoder = Encoder::from_writer(BufWriter::new(file), &info)?;

for picture in pictures.iter() {
    encoder.write_picture(picture)?;
}
```

## Demo
You can test the library's decoding capabilities using [tim2view](tim2view), a simple TIM2 texture viewer built using tim2-rs. Prebuilt binaries are available on the [Releases page](https://github.com/LeadRDRK/tim2-rs/releases), or you could install it using cargo.

Also check out [tim2conv](https://github.com/LeadRDRK/tim2conv), a command line tool for converting/creating TIM2 files.

## License
[MIT](LICENSE)
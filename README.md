# tim2-rs
TIM2 texture decoding and encoding library written in Rust.

# Features
- Nearly full support for the TIM2 format (on par with [Rainbow](https://github.com/marco-calautti/Rainbow)) and CLT2 (CLUT only).
- Serde support for serializing metadata with the `serde` feature.

# TODO
- Support for mipmapped images.

Help needed! If you have a TIM2 file with mipmaps, please create an issue on GitHub with the file attached so I can take a look at it.

# Example
- Decoding
```rs
use tim2::Decoder;

let file = File::open(path)?;
let mut decoder = Decoder::from_reader(BufReader::new(file))?;

while let Some(picture_decoder) = decoder.next_picture()? {
    let picture = picture_decoder.decode()?;
    // do something with the picture...
}
```

- Encoding
```rs
use tim2::{Encoder, Picture, Info};

let pictures: Vec<Picture>;
let info = Info {
    version: 4,
    format: 1, // 128-byte alignment
    picture_count: pictures.len() as u16
};

let file = File::create(path)?;
let mut encoder = Encoder::from_writer(BufWriter::new(file), &info)?;

for picture in pictures.iter() {
    encoder.write_picture(picture)?;
}
```

# Demo
You can test the library's decoding capability using [tim2view](tim2view), a simple TIM2 texture viewer built using tim2-rs. Prebuilt binaries are available on the [Releases page](https://github.com/LeadRDRK/tim2-rs/releases), or you could install it using cargo.

Also check out [tim2conv](https://github.com/LeadRDRK/tim2conv), a command line tool for converting TIM2 files.

# References
tim2-rs was written based on the following resources:

- ["TM2 TIM2 Image" on Reverse Engineering Wiki](https://rewiki.miraheze.org/wiki/TM2_TIM2_Image)
- [TIM2 binary template by Bartłomiej Duda](https://github.com/bartlomiejduda/Tools/blob/master/Binary%20Templates/TIM2_TM2_PS2_Image_binary_template.bt)
- [Rainbow by Marco Calautti](https://github.com/marco-calautti/Rainbow)

# License
[MIT](LICENSE)
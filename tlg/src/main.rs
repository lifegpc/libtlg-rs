mod arg;
#[cfg(feature = "encode")]
use std::io::BufRead;
use std::io::{Seek, Write};

fn convert_bgr_to_rgb(data: &mut libtlg_rs::Tlg) {
    match data.color {
        libtlg_rs::TlgColorType::Bgra32 => {
            for i in (0..data.data.len()).step_by(4) {
                let b = data.data[i];
                data.data[i] = data.data[i + 2];
                data.data[i + 2] = b; // Swap red and blue
            }
        }
        libtlg_rs::TlgColorType::Bgr24 => {
            for i in (0..data.data.len()).step_by(3) {
                let b = data.data[i];
                data.data[i] = data.data[i + 2];
                data.data[i + 2] = b; // Swap red and blue
            }
        }
        _ => {}
    }
}

fn get_relative_path(input: &str, ext: &str) -> String {
    let mut pb = std::path::PathBuf::from(input);
    pb.set_extension(ext);
    pb.to_string_lossy().to_string()
}

fn main() {
    let args = arg::Arg::parse();
    let file = std::fs::File::open(&args.input).expect("Failed to open input file");
    let mut file = std::io::BufReader::new(file);
    if libtlg_rs::check_tlg(&mut file).expect("Failed to check TLG format") {
        let mut tlg = libtlg_rs::load_tlg(&mut file).expect("Failed to load TLG file");
        let output = match &args.output {
            Some(output) => output.clone(),
            None => get_relative_path(&args.input, "png"),
        };
        convert_bgr_to_rgb(&mut tlg);
        let mut output_file = std::fs::File::create(&output).expect("Failed to create output file");
        let mut encoder = png::Encoder::new(&mut output_file, tlg.width, tlg.height);
        encoder.set_color(match tlg.color {
            libtlg_rs::TlgColorType::Bgra32 => png::ColorType::Rgba,
            libtlg_rs::TlgColorType::Bgr24 => png::ColorType::Rgb,
            libtlg_rs::TlgColorType::Grayscale8 => png::ColorType::Grayscale,
        });
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("Failed to write PNG header");
        writer
            .write_image_data(&tlg.data)
            .expect("Failed to write PNG image data");
        if !tlg.tags.is_empty() {
            let mut tags_file = std::fs::File::create(get_relative_path(&output, "tags"))
                .expect("Failed to create tags file");
            for (key, value) in &tlg.tags {
                tags_file.write_all(&key).expect("Failed to write tag key");
                tags_file
                    .write_all(b"=")
                    .expect("Failed to write tag separator");
                tags_file
                    .write_all(&value)
                    .expect("Failed to write tag value");
                tags_file.write_all(b"\n").expect("Failed to write newline");
            }
        }
    } else {
        file.rewind().expect("Failed to rewind file");
        #[cfg(feature = "encode")]
        {
            let decoder = png::Decoder::new(file);
            let mut reader = decoder.read_info().expect("Failed to read PNG info");
            let width = reader.info().width;
            let height = reader.info().height;
            if reader.info().bit_depth != png::BitDepth::Eight {
                panic!("Unsupported bit depth: {:?}", reader.info().bit_depth);
            }
            let color_type = match reader.info().color_type {
                png::ColorType::Rgba => libtlg_rs::TlgColorType::Bgra32,
                png::ColorType::Rgb => libtlg_rs::TlgColorType::Bgr24,
                png::ColorType::Grayscale => libtlg_rs::TlgColorType::Grayscale8,
                _ => panic!("Unsupported color type: {:?}", reader.info().color_type),
            };
            let imgsize = width as usize * height as usize * reader.info().color_type.samples();
            let mut data = vec![0u8; imgsize];
            reader
                .next_frame(&mut data)
                .expect("Failed to read PNG frame");
            let mut tags = std::collections::HashMap::new();
            let tags_path = get_relative_path(&args.input, "tags");
            if std::path::Path::new(&tags_path).exists() {
                let tags_file = std::fs::File::open(&tags_path).expect("Failed to open tags file");
                let mut tags_reader = std::io::BufReader::new(tags_file);
                let mut line = String::new();
                while tags_reader
                    .read_line(&mut line)
                    .expect("Failed to read line")
                    > 0
                {
                    if let Some(eq_pos) = line.find('=') {
                        let key = line[..eq_pos].trim().as_bytes().to_vec();
                        let value = line[eq_pos + 1..].trim().as_bytes().to_vec();
                        tags.insert(key, value);
                    }
                    line.clear();
                }
            }
            let mut tlg = libtlg_rs::Tlg {
                tags,
                version: 5,
                width,
                height,
                color: color_type,
                data,
            };
            convert_bgr_to_rgb(&mut tlg);
            let output = match &args.output {
                Some(output) => output.clone(),
                None => get_relative_path(&args.input, "tlg"),
            };
            let mut output_file =
                std::fs::File::create(&output).expect("Failed to create output file");
            libtlg_rs::save_tlg(&tlg, &mut output_file).expect("Failed to save TLG file");
        }
    }
}

mod arg;
use std::io::Seek;

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

fn main() {
    let args = arg::Arg::parse();
    let file = std::fs::File::open(&args.input).expect("Failed to open input file");
    let mut file = std::io::BufReader::new(file);
    if libtlg_rs::check_tlg(&mut file).expect("Failed to check TLG format") {
        let mut tlg = libtlg_rs::load_tlg(&mut file).expect("Failed to load TLG file");
        let output = match &args.output {
            Some(output) => output.clone(),
            None => {
                let mut pb = std::path::PathBuf::from(&args.input);
                pb.set_extension("png");
                pb.to_string_lossy().to_string()
            }
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
    } else {
        file.rewind().expect("Failed to rewind file");
    }
}

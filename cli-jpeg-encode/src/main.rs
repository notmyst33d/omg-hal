use std::fs;

use clap::Parser;
use image::ImageReader;
use mtk_jpeg::{EncodeConfig, EncodeError, MtkJpeg};
use mtk_m4u::mt6768::Port;
use yuv::{
    YuvBiPlanarImageMut, YuvChromaSubsampling, YuvConversionMode, YuvRange, YuvStandardMatrix,
    rgb_to_yuv_nv12,
};

#[derive(Parser)]
struct Args {
    input: String,
    output: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let image = ImageReader::open(args.input)?
        .decode()?
        .to_rgb8();

    let mut yuv_image = YuvBiPlanarImageMut::<u8>::alloc(
        image.width(),
        image.height(),
        YuvChromaSubsampling::Yuv420,
    );
    rgb_to_yuv_nv12(
        &mut yuv_image,
        &image,
        image.width() * 3,
        YuvRange::Limited,
        YuvStandardMatrix::Bt709,
        YuvConversionMode::Balanced,
    )?;

    let mut output = vec![0u8; 65535];
    let jpeg = MtkJpeg::open()?;
    let config = EncodeConfig {
        width: yuv_image.width,
        height: yuv_image.height,
        y_plane: yuv_image.y_plane.borrow(),
        uv_plane: yuv_image.uv_plane.borrow(),
        output: &mut output,
        read_port: Port::JpgencRdma,
        write_port: Port::JpgencBsdma,
    };

    loop {
        let Err(e) = jpeg.encode(&config) else {
            break;
        };
        if let EncodeError::OutputBufferTooSmall = e {
            todo!();
        } else {
            return Err(e.into());
        }
    }

    fs::write(args.output, output)?;

    println!("Encoded JPEG successfully");

    Ok(())
}

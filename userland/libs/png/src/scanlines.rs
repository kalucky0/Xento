use alloc::vec;

use crate::{
    enums::{FilterType, InterlaceMethod, PixelType},
    AncillaryChunks, PngHeader, ScanlineIterator,
};

pub fn process(
    ancillary_chunks: &AncillaryChunks,
    output_rgba: &mut [u8],
    pixel_type: PixelType,
    png_header: &PngHeader,
    scanline_data: &mut [u8],
) -> Option<()> {
    let bytes_per_pixel =
        (png_header.bit_depth as usize * png_header.color_type.sample_multiplier() as usize + 7)
            / 8;

    match png_header.interlace_method {
        InterlaceMethod::None => process_non_interlaced(
            ancillary_chunks,
            output_rgba,
            pixel_type,
            png_header,
            scanline_data,
            bytes_per_pixel,
        )?,
        InterlaceMethod::Adam7 => process_interlaced(
            ancillary_chunks,
            output_rgba,
            pixel_type,
            png_header,
            scanline_data,
            bytes_per_pixel,
        )?,
    }

    Some(())
}

fn process_non_interlaced(
    ancillary_chunks: &AncillaryChunks,
    output_rgba: &mut [u8],
    pixel_type: PixelType,
    png_header: &PngHeader,
    scanline_data: &mut [u8],
    bytes_per_pixel: usize,
) -> Option<()> {
    let mut cursor = 0;
    let bytes_per_scanline = (png_header.width as u64
        * png_header.bit_depth as u64
        * png_header.color_type.sample_multiplier() as u64
        + 7)
        / 8;
    let bytes_per_scanline = TryInto::<usize>::try_into(bytes_per_scanline).ok()?;
    let mut previous_scanline = vec![0; bytes_per_scanline];

    for y in 0..png_header.height {
        let filter_type = FilterType::new(scanline_data[cursor])?;
        cursor += 1;

        let scanline = &mut scanline_data[cursor..cursor + bytes_per_scanline];

        for x in 0..bytes_per_scanline {
            scanline[x] = defilter(&filter_type, scanline, &previous_scanline, x, bytes_per_pixel)?;
        }

        let iterator =
            ScanlineIterator::new(ancillary_chunks, png_header.width, pixel_type, scanline);

        for (x, pixel) in iterator.enumerate() {
            let index = ((y * png_header.width) as usize + x) * 4;
            output_rgba[index..index + 4].copy_from_slice(&[pixel.0, pixel.1, pixel.2, pixel.3]);
        }

        previous_scanline.copy_from_slice(scanline);
        cursor += bytes_per_scanline;
    }

    Some(())
}

fn process_interlaced(
    ancillary_chunks: &AncillaryChunks,
    output_rgba: &mut [u8],
    pixel_type: PixelType,
    png_header: &PngHeader,
    scanline_data: &mut [u8],
    bytes_per_pixel: usize,
) -> Option<()> {
    let mut cursor = 0;
    let bytes_per_scanline = bytes_per_pixel * png_header.width as usize;
    let mut previous_scanline = vec![0; bytes_per_scanline];

    for i in 1..8 {
        let (width, height) = get_pass_dimensions(png_header.width, png_header.height, i);
        if width == 0 || height == 0 {
            continue;
        }

        let bytes_per_scanline = (width as u64
            * png_header.bit_depth as u64
            * png_header.color_type.sample_multiplier() as u64
            + 7)
            / 8;
        let bytes_per_scanline = TryInto::<usize>::try_into(bytes_per_scanline).ok()?;

        let previous_scanline = &mut previous_scanline[..bytes_per_scanline];
        for b in previous_scanline.iter_mut() {
            *b = 0;
        }

        for y in 0..height {
            let filter_type = FilterType::new(scanline_data[cursor])?;
            cursor += 1;

            let scanline = &mut scanline_data[cursor..cursor + bytes_per_scanline];

            for x in 0..bytes_per_scanline {
                scanline[x] = defilter(&filter_type, scanline, previous_scanline, x, bytes_per_pixel)?;
            }

            let iterator = ScanlineIterator::new(ancillary_chunks, width, pixel_type, scanline);

            for (x, pixel) in iterator.enumerate() {
                let (x, y) = get_pass_coordinates(x as u32, y, i);
                let index = ((y * png_header.width) as usize + x as usize) * 4;
                output_rgba[index..index + 4]
                    .copy_from_slice(&[pixel.0, pixel.1, pixel.2, pixel.3]);
            }

            previous_scanline.copy_from_slice(scanline);
            cursor += bytes_per_scanline;
        }
    }

    Some(())
}

fn get_pass_dimensions(width: u32, height: u32, pass: u8) -> (u32, u32) {
    match pass {
        1 => ((width + 7) / 8, (height + 7) / 8),
        2 => ((width / 8) + ((width % 8) / 5), (height + 7) / 8),
        3 => (
            ((width / 8) * 2) + (width % 8 + 3) / 4,
            (height / 8) + ((height % 8) / 5),
        ),
        4 => (((width / 8) * 2) + (width % 8 + 1) / 4, (height + 3) / 4),
        5 => (
            (width / 2) + (width % 2),
            ((height / 8) * 2) + (height % 8 + 1) / 4,
        ),
        6 => (width / 2, (height / 2) + (height % 2)),
        7 => (width, height / 2),
        _ => (0, 0),
    }
}

fn get_pass_coordinates(x: u32, y: u32, pass: u8) -> (u32, u32) {
    match pass {
        1 => (x * 8, y * 8),
        2 => (x * 8 + 4, y * 8),
        3 => (x * 4, y * 8 + 4),
        4 => (x * 4 + 2, y * 4),
        5 => (x * 2, y * 4 + 2),
        6 => (x * 2 + 1, y * 2),
        7 => (x, y * 2 + 1),
        _ => (0, 0),
    }
}

fn defilter(
    filter_type: &FilterType,
    scanline: &mut [u8],
    previous_scanline: &[u8],
    x: usize,
    bytes_per_pixel: usize,
) -> Option<u8> {
    match filter_type {
        FilterType::None => Some(scanline[x]),
        FilterType::Sub => {
            let a = if x < bytes_per_pixel {
                0
            } else {
                scanline[x - bytes_per_pixel]
            };
            scanline[x] = scanline[x].wrapping_add(a);
            Some(scanline[x])
        }
        FilterType::Up => {
            let b = previous_scanline[x];
            scanline[x] = scanline[x].wrapping_add(b);
            Some(scanline[x])
        }
        FilterType::Average => {
            let a = if x < bytes_per_pixel {
                0
            } else {
                scanline[x - bytes_per_pixel]
            };
            let b = previous_scanline[x];
            scanline[x] = scanline[x].wrapping_add((a + b) / 2);
            Some(scanline[x])
        }
        FilterType::Paeth => {
            let a = if x < bytes_per_pixel {
                0
            } else {
                scanline[x - bytes_per_pixel]
            };
            let b = previous_scanline[x];
            let c = if x < bytes_per_pixel {
                0
            } else {
                previous_scanline[x - bytes_per_pixel]
            };
            let p = (a + b - c) as i16;
            let pa = abs(p - a as i16);
            let pb = abs(p - b as i16);
            let pc = abs(p - c as i16);
            let pr = if pa <= pb && pa <= pc {
                a
            } else if pb <= pc {
                b
            } else {
                c
            };
            scanline[x] = scanline[x].wrapping_add(pr);
            Some(scanline[x])
        }
    }
}

fn abs(a: i16) -> u8 {
    if a < 0 {
        -a as u8
    } else {
        a as u8
    }
}
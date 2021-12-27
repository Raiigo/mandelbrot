use num::Complex;
use std::env;
use std::str::FromStr;
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

fn main() {
    
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage : {} mandelbrot.png 1920x1080 -1,1 1,-1", args[0]);
        std::process::exit(1);
    }
    
    let bounds = parse_pair::<usize>(&args[2], 'x').expect("Error while parsing bounds");
    let upper_left = parse_complex(&args[3]).expect("Error while parsing first complex number");
    let lower_right = parse_complex(&args[4]).expect("Error while parsing second complex number");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    {

        let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();

        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);

                spawner.spawn(move |_| {
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });

            }
        }).unwrap();

    }

    write_image(&args[1], &pixels, bounds).expect("Error while writing image");

}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(x), Ok(y)) => Some((x, y)),
                _ => None
            }
        }
    }

}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        None => None,
        Some((re, im)) => Some(Complex { re, im })
    }
}

fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>) 
   -> Complex<f64> 
{

    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);

    Complex::<f64> { re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
                     im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64 }

}

fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: Complex<f64>,
          lower_right: Complex<f64>)
{

    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {

            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);

            pixels[row * bounds.0 + column] = 
                match escape_time(point, 255) {
                    None => 0,
                    Some(time) => 255 - time as u8
                };

        }
    }

}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error> {
    
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())

}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 100),
                              (50, 50),
                              Complex::<f64> { re: -1.0, im: 1.0},
                              Complex::<f64> { re: 1.0, im: -1.0}),
               Complex::<f64> { re: 0.0, im: 0.0});
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("-10x10",    'x'), Some((-10, 10)));
    assert_eq!(parse_pair::<i32>("800,600f",  ','), None);
    assert_eq!(parse_pair::<i32>("1920/1080", '/'), Some((1920, 1080)));
    assert_eq!(parse_pair::<i32>("10.6*-34",  '*'), None);
    assert_eq!(parse_pair::<String>("abc*jhu",  '*'), Some(("abc".to_string(), "jhu".to_string())))
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("3.14,1"), Some(Complex { re: 3.14, im: 1.0 }));
    assert_eq!(parse_complex("-12/4"), None);
}
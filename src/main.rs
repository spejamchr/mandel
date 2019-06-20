extern crate image;
extern crate threadpool;
extern crate rand;

use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use rand::Rng;

fn main() {
    // Standard view
    mandelbrot(-0.5, 0.0, 1.0, 255, "test.png");

    // A fun take on the standard view
    // mandelbrot(-0.5, 0.0, 1.0, 25, "test.png");

    // Half of the set
    // mandelbrot(-0.65, 0.7, 1.4, 255, "test.png");

    // Nice wallpaper
    // mandelbrot(-1.025, 0.234375, 3.0931094044, 255, "test.png");

    // The center of a spiral zoomed to the limits of f64 accuracy
    // mandelbrot(-1.344662931374433, 0.048458507821225, 46.0, 30000, "test.png");

    // A mini-mandelbrot
    // mandelbrot(-1.3688005, 0.0558586, 24.0, 2500, "test.png");
}

fn calc_scaled_mandel(x0c: f64, y0c: f64, max_iters: u16) -> f64 {
    let mut x = 0.;
    let mut y = 0.;
    let mut i: f64 = 0.;

    while x*x + y*y <= 4. && i < max_iters as f64 {
        let xt = x*x - y*y + x0c;
        y = 2.*x*y + y0c;
        x = xt;
        i += 1.;
    }

    if i < max_iters as f64 {
        let log_zn = log(x*x + y*y) / 2.;
        let nu = log(log_zn / log(2.)) / log(2.);
        i = i + 1. - nu;
    }

    return i / (max_iters as f64);
}

fn calc_rgb(scaled_mandel: f64) -> [u8; 3] {
    let scaled = scaled_mandel * 255.;
    let s = scaled as u8;
    let mut rng = rand::thread_rng();

    if s == 255 {
        return [0, 0, 0];
    } else if s > 40 {
        return [s, s, s];
    } else {
        let fraction = scaled - s as f64;
        let offest: u8 = if rng.gen::<f64>() < fraction { 0 } else { 1 };
        let offset2: u8 = if rng.gen::<f64>() < 0.5 { 0 } else { 1 };
        let mut r = s - offest - offset2;
        if r > s { r = 0 }
        return [r, r, r];
    }
}

fn log(n: f64) -> f64 {
    return n.log(2.7182818285);
}

fn mandelbrot(real: f64, imaginary: f64, zoom: f64, max_iters: u16, file: &str) {
    let ratio = 1.6;
    let px_width = 2880/2;

    let two: f64 = 2.0;

    let height = two.powf(-1.0 * zoom + 2.0);
    let width = height * ratio;
    let pixel_size = width / (px_width as f64 - 1.0);
    let px_height = 1 + (height / pixel_size).round() as u32;
    let bottom = imaginary - (height / 2.0);
    let left = real - (width / 2.0);

    let n_workers = 12;
    let n_jobs = px_width * px_height;
    let pool = ThreadPool::new(n_workers);

    let (tx, rx) = channel();

    for x0 in 0..px_width {
        for y0 in 0..px_height {
            let tx = tx.clone();
            pool.execute(move|| {
                let x0c = (x0 as f64) / (px_width as f64 - 1.0) * width + left;
                let y0c = (1. - (y0 as f64) / (px_height as f64 - 1.0)) * height + bottom;

                let scaled_mandel = calc_scaled_mandel(x0c, y0c, max_iters);
                let rgb = calc_rgb(scaled_mandel);

                tx.send((x0, y0, rgb)).unwrap();
            });
        }
    }

    let  mut img = image::ImageBuffer::new(px_width, px_height);

    rx.iter().take(n_jobs as usize).for_each(|(x, y, rgb)| {
        img.put_pixel(x, y, image::Rgb(rgb));
    });

    img.save(file).unwrap();
}

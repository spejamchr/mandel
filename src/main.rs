extern crate image;
extern crate threadpool;
extern crate rand;

use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use rand::Rng;

fn main() {
    let xs = [-1.4, -0.65];
    let ys = [0.0, 0.46875];
    mandelbrot(xs, ys, 2880, "2880x1800.png");
}

fn calc_rgb(x0c: f64, y0c: f64, max_iters: u16) -> [u8; 3] {
    let mut x = 0.;
    let mut y = 0.;
    let mut i: f64 = 0.;

    while x*x + y*y <= 1000. && i < max_iters as f64 {
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
    let mut rng = rand::thread_rng();

    let scaled = i / (max_iters as f64) * 255.;
    let s = scaled as u8;

    if s == 255 {
        return [0, 0, 0];
    } else if s > 40 {
        return [s, s, s];
    } else {
        let fraction = scaled - s as f64;
        let offest: u8 = if rng.gen::<f64>() < fraction { 0 } else { 1 };
        let offset2: u8 = if rng.gen::<f64>() < 0.5 { 0 } else { 1 };
        let r = s - offest - offset2;
        return [r, r, r];
    }
}

fn log(n: f64) -> f64 {
    return n.log(2.7182818285);
}

fn mandelbrot(xs: [f64; 2], ys0: [f64; 2], px_width: u32, file: &str) {
    let max_iters: u16 = 255;
    let width = xs[1] - xs[0];
    let pixel_size: f64 = (width) / (px_width as f64 - 1.0);
    let px_height: u32 = 1 + ((ys0[1] - ys0[0]) / pixel_size).round() as u32;
    let height: f64 = (px_height as f64 - 1.0) * pixel_size;
    let ys: [f64; 2] = [ys0[0], ys0[0] + height];

    let n_workers = 12;
    let n_jobs = px_width * px_height;
    let pool = ThreadPool::new(n_workers);

    let (tx, rx) = channel();

    for x0 in 0..px_width {
        for y0 in 0..px_height {
            let tx = tx.clone();
            pool.execute(move|| {
                let x0c = (x0 as f64) / (px_width as f64 - 1.0) * width + xs[0];
                let y0c = (1. - (y0 as f64) / (px_height as f64 - 1.0)) * height + ys[0];

                let rgb = calc_rgb(x0c, y0c, max_iters);

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

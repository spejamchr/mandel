extern crate image;
extern crate threadpool;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

fn main() {
    let xs = [-1.4, -0.65];
    let ys = [0.0, 0.46875];
    mandelbrot(xs, ys, 880, "test.png");
}

fn calc_i(x0c: f64, y0c: f64, max_iters: u16) -> u16 {
    let mut x = 0.;
    let mut y = 0.;

    let mut i: u16 = 0;
    while x*x + y*y <= 4. && i < max_iters {
        let xt = x*x - y*y + x0c;
        y = 2.*x*y + y0c;
        x = xt;
        i += 1;
    }

    return i;
}

fn mandelbrot(xs: [f64; 2], ys0: [f64; 2], px_width: u32, file: &str) {
    let max_iters: u16 = 2550;
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

                let i = calc_i(x0c, y0c, max_iters) as u8;

                tx.send((x0, y0, i)).unwrap();
            });
        }
    }

    let  mut img = image::ImageBuffer::new(px_width, px_height);

    rx.iter().take(n_jobs as usize).for_each(|(x, y, i)| {
        img.put_pixel(x, y, image::Rgb([i, i, i]));
    });

    img.save(file).unwrap();
}

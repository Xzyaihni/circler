use std::{
    env,
    process
};

use image::{
    Pixel,
    GenericImage,
    imageops::{self, FilterType}
};


fn complain(message: &str) -> !
{
    eprintln!("{message}");
    process::exit(1)
}

fn combine<P, I>(mut main_image: I, back_image: I, circle_size: f64, edge_fuzz: f64) -> I
where
    f64: From<P::Subpixel>,
    P::Subpixel: num::NumCast,
    P: Pixel + 'static,
    I: GenericImage<Pixel=P>
{
    let (width, height) = (main_image.width(), main_image.height());

    let filter_type = FilterType::CatmullRom;
    let back_image = imageops::resize(&back_image, width, height, filter_type);

    let aspect_ratio = width as f64 / height as f64;

    for y in 0..height
    {
        for x in 0..width
        {
            let x_local = x as f64 / width as f64 * 2.0 - 1.0;
            let x_local = x_local * aspect_ratio;

            let y_local = y as f64 / height as f64 * 2.0 - 1.0;

            let blend = x_local.hypot(y_local) - circle_size;

            let blend = if edge_fuzz != 0.0
            {
                blend / edge_fuzz
            } else
            {
                if blend <= 0.0 {0.0} else {1.0}
            };

            let this_pixel = main_image.get_pixel(x, y);
            let background_pixel = back_image.get_pixel(x, y);

            let mixed_pixel = this_pixel.map2(background_pixel, |a, b|
            {
                let i = blend.max(0.0).min(1.0);

                let mixed = f64::from(a) * (1.0 - i) + f64::from(b) * i;

                <P::Subpixel as num::NumCast>::from(mixed).unwrap()
            });

            main_image.put_pixel(x, y, mixed_pixel);
        }
    }

    main_image
}

fn main()
{
    let parse_image = |path: &str|
    {
        image::open(path).unwrap_or_else(|err|
        {
            complain(&format!("failed to load image at \"{path}\": {err:?}"))
        })
    };

    let mut args = env::args().skip(1);

    let main_image = args.next()
        .unwrap_or_else(|| complain("please provide a path to the main image"));
    let main_image = parse_image(&main_image);

    let back_image = args.next()
        .unwrap_or_else(|| complain("please provide a path to the background image"));
    let back_image = parse_image(&back_image);

    let circle_size = args.next().map(|arg|
    {
        arg.parse().unwrap_or_else(|err|
        {
            complain(&format!("couldnt parse {arg} as a float: {err:?}"))
        })
    }).unwrap_or(0.5);

    let edge_fuzz = args.next().map(|arg|
    {
        arg.parse().unwrap_or_else(|err|
        {
            complain(&format!("couldnt parse {arg} as a float: {err:?}"))
        })
    }).unwrap_or(0.01);

    let combined = combine(main_image, back_image, circle_size, edge_fuzz);

    combined.save("output.png")
        .unwrap_or_else(|err| complain(&format!("error saving the image: {err:?}")));
}
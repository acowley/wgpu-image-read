use wgpu::util::DeviceExt;

struct ImageDims {
    nrows: u32,
    ncols: u32,
}

impl ImageDims {
    fn num_pixels(&self) -> u32 {
        self.nrows * self.ncols
    }
}

fn load_texture_good(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
) -> Result<(ImageDims, wgpu::Texture, Vec<u8>), anyhow::Error> {
    let img = image::io::Reader::open(path)?.decode()?;
    let img_buffer = img
        .as_rgb8()
        .ok_or_else(|| anyhow::format_err!("Image can't be interpreted as rgb8"))?;
    let rgba = img_buffer
        .pixels()
        .flat_map(|&image::Rgb([r, g, b])| [r, g, b, 255])
        .collect::<Vec<u8>>();
    let nrows = img_buffer.height();
    let ncols = img_buffer.width();
    log::info!("Loaded a {}x{} RGB8 image", ncols, nrows);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: ncols,
            height: nrows,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * ncols),
            rows_per_image: std::num::NonZeroU32::new(nrows),
        },
        wgpu::Extent3d {
            width: ncols,
            height: nrows,
            depth_or_array_layers: 1,
        },
    );
    let dims = ImageDims { nrows, ncols };
    Ok((dims, texture, rgba))
}

fn load_texture_bad(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
) -> Result<(ImageDims, wgpu::Texture, Vec<u8>), anyhow::Error> {
    let img = image::io::Reader::open(path)?.decode()?;
    let img_buffer = img
        .as_rgb8()
        .ok_or_else(|| anyhow::format_err!("Image can't be interpreted as rgb8"))?;
    let rgba = img_buffer
        .pixels()
        .flat_map(|&image::Rgb([r, g, b])| [r, g, b, 255])
        .collect::<Vec<u8>>();
    let nrows = img_buffer.height();
    let ncols = img_buffer.width();
    log::info!("Loaded a {}x{} RGB8 image", ncols, nrows);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: ncols,
            height: nrows,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
    });
    let staging = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &rgba,
        usage: wgpu::BufferUsages::COPY_SRC,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_texture(
        wgpu::ImageCopyBuffer {
            buffer: &staging,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * ncols),
                rows_per_image: std::num::NonZeroU32::new(nrows),
            },
        },
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: ncols,
            height: nrows,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    let dims = ImageDims { nrows, ncols };
    Ok((dims, texture, rgba))
}

fn init_gpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .unwrap();
    log::info!("Acquired adapter: {:?}", adapter.get_info());
    pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        Some(std::path::Path::new("trace")),
    ))
    .unwrap()
}

fn save_rgb_texture<P>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    dims: &ImageDims,
    texture: &wgpu::Texture,
    path: P,
) -> Result<Vec<u8>, anyhow::Error>
where
    P: AsRef<str>,
{
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (dims.num_pixels() * 4) as _,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &staging,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dims.ncols),
                rows_per_image: std::num::NonZeroU32::new(dims.nrows),
            },
        },
        wgpu::Extent3d {
            width: dims.ncols,
            height: dims.nrows,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let fut = slice.map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);
    pollster::block_on(fut)?;
    let rgba = slice.get_mapped_range().to_vec();
    image::save_buffer(
        path.as_ref(),
        &rgba,
        dims.ncols,
        dims.nrows,
        image::ColorType::Rgba8,
    )?;
    staging.unmap();

    Ok(rgba)
}

fn ensure_directory<P>(path: P) -> Result<(), std::io::Error>
where
    P: AsRef<std::path::Path>,
{
    if !path.as_ref().exists() {
        log::info!("Creating directory: {}", path.as_ref().to_string_lossy());
        std::fs::create_dir(path)
    } else {
        Ok(())
    }
}

fn ensure_directories() -> Result<(), anyhow::Error> {
    ensure_directory("trace")?;
    ensure_directory("outputs")?;
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    log::info!("Starting up");
    ensure_directories()?;
    let (device, queue) = init_gpu();
    let input_image = match std::env::args().nth(1) {
        None => anyhow::bail!("Usage: image_read input-image-file"),
        Some(p) => p,
    };
    let (dims1, bad, input1) = load_texture_bad(&device, &queue, &input_image)?;
    let output1 = save_rgb_texture(&device, &queue, &dims1, &bad, "outputs/bad.png")?;
    if input1 == output1 {
        log::info!("The 'bad' method read back its own input");
    } else {
        log::info!("The 'bad' method failed to read back its own input");
    }
    let (dims2, good, input2) = load_texture_good(&device, &queue, &input_image)?;
    let output2 = save_rgb_texture(&device, &queue, &dims2, &good, "outputs/good.png")?;
    if input2 == output2 {
        log::info!("The 'good' method read back its own input");
    } else {
        log::info!("The 'good' method failed to read back its own input");
    }
    Ok(())
}

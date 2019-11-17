# blackmagic-raw-rs

This crate wraps the Blackmagic RAW SDK.

## Example: Extracting a Frame

An implementation of the "ExtractFrame" example that comes with the SDK would like something like this in Rust:

```rust
use std::error::Error;

use image::ConvertBuffer;

use blackmagic_raw as braw;

struct Callback {
    output_path: String,
}

impl braw::Callback for Callback {
    fn read_complete(&mut self, _job: braw::Job, result: Result<braw::Frame, braw::Error>) {
        if let Err(err) = self.on_read_complete(result) {
            println!("read error: {}", err);
        }
    }

    fn process_complete(&mut self, _job: braw::Job, result: Result<braw::ProcessedImage, braw::Error>) {
        if let Err(err) = self.on_process_complete(result) {
            println!("process error: {}", err);
        }
    }
}

impl Callback {
    fn on_read_complete(&mut self, result: Result<braw::Frame, braw::Error>) -> Result<(), Box<dyn Error>> {
        let mut frame = result?;
        frame.set_resource_format(braw::ResourceFormat::FORMAT_RGBAU8)?;
        frame.create_job_decode_and_process_frame(None, None)?.submit()?;
        Ok(())
    }

    fn on_process_complete(&mut self, result: Result<braw::ProcessedImage, braw::Error>) -> Result<(), Box<dyn Error>> {
        let mut img = result?;
        let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(img.get_width()?, img.get_height()?, img.get_resource()?).unwrap();
        let img: image::ImageBuffer<image::Rgb<u8>, _> = img.convert();
        img.save(&self.output_path)?;
        Ok(())
    }
}

pub fn extract_frame(input_path: String, output_path: String) -> Result<(), Box<dyn Error>> {
    let mut factory = braw::Factory::new_from_path("/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Libraries")?;
    let mut codec = factory.create_codec()?;
    let mut clip = codec.open_clip(&input_path)?;
    codec.with_callback(Callback{
        output_path: output_path,
    }, |codec| -> Result<(), Box<dyn Error>> {
        clip.create_job_read_frame(0)?.submit()?;
        codec.flush_jobs()?;
        Ok(())
    })??;
    Ok(())
}
```

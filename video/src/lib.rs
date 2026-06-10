use crossbeam_channel::{bounded, Receiver, TrySendError};
use ffmpeg_next::ffi::*;
use std::path::Path;
use std::thread;

pub fn init() {
    let _ = ffmpeg_next::init();
}

#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("Decoder error: {0}")]
    Decoder(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct AVFrameWrap {
    ptr: *mut AVFrame,
}

unsafe impl Send for AVFrameWrap {}
unsafe impl Sync for AVFrameWrap {}

impl AVFrameWrap {
    pub fn new(frame: &ffmpeg_next::util::frame::Video) -> Self {
        let raw_ptr = unsafe { frame.as_ptr() };
        let cloned_ptr = unsafe { av_frame_clone(raw_ptr) };
        Self { ptr: cloned_ptr }
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.ptr).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.ptr).height }
    }

    pub fn format(&self) -> i32 {
        unsafe { (*self.ptr).format }
    }

    pub fn to_image(&self) -> Result<image::DynamicImage, VideoError> {
        unsafe {
            if self.ptr.is_null() {
                return Err(VideoError::Decoder("AVFrame pointer is null".to_string()));
            }
            let width = (*self.ptr).width;
            let height = (*self.ptr).height;
            let src_format = (*self.ptr).format;
            let dst_format = AVPixelFormat::AV_PIX_FMT_RGB24;

            let mut dst_frame = av_frame_alloc();
            if dst_frame.is_null() {
                return Err(VideoError::Decoder(
                    "Failed to allocate destination frame".to_string(),
                ));
            }

            (*dst_frame).width = width;
            (*dst_frame).height = height;
            (*dst_frame).format = dst_format as i32;

            let ret = av_image_alloc(
                (*dst_frame).data.as_mut_ptr(),
                (*dst_frame).linesize.as_mut_ptr(),
                width,
                height,
                dst_format,
                1,
            );
            if ret < 0 {
                av_frame_free(&mut dst_frame);
                return Err(VideoError::Decoder(
                    "Failed to allocate destination image buffer".to_string(),
                ));
            }

            const SWS_BILINEAR: i32 = 2;
            let sws_ctx = sws_getContext(
                width,
                height,
                std::mem::transmute::<i32, AVPixelFormat>(src_format),
                width,
                height,
                dst_format,
                SWS_BILINEAR,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if sws_ctx.is_null() {
                av_freep(&mut (*dst_frame).data[0] as *mut *mut u8 as *mut std::ffi::c_void);
                av_frame_free(&mut dst_frame);
                return Err(VideoError::Decoder(
                    "Failed to initialize software scaling context".to_string(),
                ));
            }

            sws_scale(
                sws_ctx,
                (*self.ptr).data.as_ptr() as *const *const u8,
                (*self.ptr).linesize.as_ptr(),
                0,
                height,
                (*dst_frame).data.as_mut_ptr(),
                (*dst_frame).linesize.as_ptr(),
            );

            sws_freeContext(sws_ctx);

            let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
            let data_ptr = (*dst_frame).data[0];
            let stride = (*dst_frame).linesize[0] as usize;
            for y in 0..height {
                let row_ptr = data_ptr.add(y as usize * stride);
                let row_slice = std::slice::from_raw_parts(row_ptr, (width * 3) as usize);
                rgb_data.extend_from_slice(row_slice);
            }

            av_freep(&mut (*dst_frame).data[0] as *mut *mut u8 as *mut std::ffi::c_void);
            av_frame_free(&mut dst_frame);

            let rgb_img = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                width as u32,
                height as u32,
                rgb_data,
            )
            .ok_or_else(|| {
                VideoError::Decoder("Failed to build image buffer from raw RGB data".to_string())
            })?;
            Ok(image::DynamicImage::ImageRgb8(rgb_img))
        }
    }
}

impl Clone for AVFrameWrap {
    fn clone(&self) -> Self {
        let cloned_ptr = unsafe { av_frame_clone(self.ptr) };
        Self { ptr: cloned_ptr }
    }
}

impl Drop for AVFrameWrap {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                av_frame_free(&mut self.ptr);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn decode_video_to_channel(
    video_path: &Path,
) -> Result<
    (
        Receiver<AVFrameWrap>,
        thread::JoinHandle<Result<(), VideoError>>,
    ),
    VideoError,
> {
    init();

    let video_path = video_path.to_owned();

    let num_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let capacity = num_cores * 2;
    let (sender, receiver) = bounded::<AVFrameWrap>(capacity);

    let sender_clone = sender.clone();
    let receiver_clone = receiver.clone();

    let join_handle = thread::spawn(move || -> Result<(), VideoError> {
        let mut ictx = ffmpeg_next::format::input(&video_path)
            .map_err(|e| VideoError::Decoder(format!("Failed to open input context: {}", e)))?;

        let input_stream = ictx
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or_else(|| VideoError::Decoder("No video stream found".to_string()))?;
        let video_stream_index = input_stream.index();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())
                .map_err(|e| {
                    VideoError::Decoder(format!("Failed to create codec context: {}", e))
                })?;
        let mut decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| VideoError::Decoder(format!("Failed to create video decoder: {}", e)))?;

        let mut decoded = ffmpeg_next::util::frame::Video::empty();

        let push_frame = |frame: &ffmpeg_next::util::frame::Video| -> Result<bool, VideoError> {
            let wrap = AVFrameWrap::new(frame);
            loop {
                match sender_clone.try_send(wrap.clone()) {
                    Ok(_) => return Ok(true),
                    Err(TrySendError::Full(_)) => {
                        // Drop the oldest frame to maintain the sliding window
                        let _ = receiver_clone.try_recv();
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        return Ok(false); // Receiver disconnected, stop decoding
                    }
                }
            }
        };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet).map_err(|e| {
                    VideoError::Decoder(format!("Error sending packet to decoder: {}", e))
                })?;

                while decoder.receive_frame(&mut decoded).is_ok() {
                    if !push_frame(&decoded)? {
                        return Ok(());
                    }
                }
            }
        }

        decoder.send_eof().ok();
        while decoder.receive_frame(&mut decoded).is_ok() {
            if !push_frame(&decoded)? {
                return Ok(());
            }
        }

        Ok(())
    });

    Ok((receiver, join_handle))
}

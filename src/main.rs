use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use minifb::{Key, Window, WindowOptions};
use std::sync::{Arc, Mutex};

const WINDOW_WIDTH: usize = 540;
const WINDOW_HEIGHT: usize = 300;

fn draw_waveform(audio_data: &[f32], buffer: &mut [u32], width: usize, height: usize) {
    // ウィンドウのバッファを黒色で塗りつぶす（内容をクリア）
    for pixel in buffer.iter_mut() {
        *pixel = 0x000000;
    }

    for x in 0..(width - 1) {
        let index1 = x * (audio_data.len() - 1) / (width - 1);
        let sample1 = audio_data[index1];
        let y1 = (sample1 + 1.0) / 2.0 * height as f32;

        let index2 = (x + 1) * (audio_data.len() - 1) / (width - 1);
        let sample2 = audio_data[index2];
        let y2 = (sample2 + 1.0) / 2.0 * height as f32;

        for i in 0..=64 {
            let t = i as f32 / 64.0;
            let y = (1.0 - t) * y1 + t * y2;

            let control_x = x as f32 + t; // 中間点のx座標
            let control_y = y; // 中間点のy座標
            let radius = 1; // 円の半径（ピクセル単位）

            draw_circle(
                buffer,
                width,
                height,
                control_x as usize,
                control_y as usize,
                radius,
            );
        }
    }
}

fn draw_circle(
    buffer: &mut [u32],
    buffer_width: usize,
    buffer_height: usize,
    center_x: usize,
    center_y: usize,
    radius: usize,
) {
    let radius_squared = (radius * radius) as f32;

    let start_x = (center_x as isize - radius as isize).max(0) as usize;
    let end_x = (center_x as isize + radius as isize).min(buffer_width as isize - 1) as usize;

    for x in start_x..=end_x {
        let dx = (x as isize - center_x as isize) as f32;
        let dy_squared = radius_squared - dx.powi(2);

        if dy_squared >= 0.0 {
            let dy = dy_squared.sqrt();

            let upper_y = (center_y as isize - dy as isize).max(0) as usize;
            let lower_y =
                (center_y as isize + dy as isize).min(buffer_height as isize - 1) as usize;

            for y in upper_y..=lower_y {
                buffer[y * buffer_width + x] = 0x00FF00;
            }
        }
    }
}

fn main() {
    let mut window = Window::new(
        "Audio Waveform Display",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // スレッドセーフなディスプレイバッファ
    let display_buffer_size = window.get_size().0 * window.get_size().1;
    let display_buffer = Arc::new(Mutex::new(vec![0u32; display_buffer_size]));

    // cpalの初期化
    let host = cpal::default_host();
    let input_device = host
        .default_output_device()
        .expect("Failed to get default input device");

    let config = input_device.default_output_config().unwrap();

    // 音声入力ストリームの設定
    let display_buffer_clone = display_buffer.clone(); // Arcをクローン
    let stream = input_device
        .build_input_stream(
            &config.config(),
            move |data: &[f32], _: &_| {
                let mut buffer = display_buffer_clone.lock().unwrap();
                // 波形を描画する際に使用する音声データ
                draw_waveform(data, &mut *buffer, WINDOW_WIDTH, WINDOW_HEIGHT);
            },
            |err| eprintln!("Error occurred: {}", err),
            None,
        )
        .expect("Failed to build input stream");

    // 音声入力ストリームの開始
    stream.play().expect("Failed to play stream");

    // ウィンドウのループ
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // ロックしてデータをコピー
        let buffer_copy = {
            let buffer = display_buffer.lock().unwrap();
            buffer.clone()
        };

        // ウィンドウの描画操作をメインスレッドで行う
        window
            .update_with_buffer(&buffer_copy, window.get_size().0, window.get_size().1)
            .unwrap();
        window.update();
    }
}

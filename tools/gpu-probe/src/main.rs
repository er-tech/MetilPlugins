//! Times a model on the CPU and on a GPU through ONNX Runtime: gpu-probe <model> <shape, like 1,3,640,640> <cpu|dml:N> <runs>
use std::time::Instant;

#[cfg(windows)]
use ort::ep;
use ort::session::Session;
use ort::value::Tensor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = &args[1];
    let shape: Vec<usize> = args[2]
        .split(',')
        .map(|part| part.parse().expect("the shape is whole numbers split by commas"))
        .collect();
    let provider = args[3].as_str();
    let runs: usize = args[4].parse().expect("runs is a whole number");

    let environment = ort::init().with_telemetry(false).build().expect("environment");
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut builder = Session::builder(&environment).expect("builder");
    #[cfg(windows)]
    {
        if let Some(device) = provider.strip_prefix("dml:") {
            let device: i32 = device.parse().expect("the adapter is a number");
            builder = builder
                .with_execution_providers([ep::DirectML::default()
                    .with_device_id(device)
                    .build()
                    .error_on_failure()])
                .expect("directml");
        }
    }
    #[cfg(not(windows))]
    assert!(!provider.starts_with("dml:"), "DirectML runs only on Windows");
    let started = Instant::now();
    let mut session = builder.commit_from_file(model).expect("load");
    let load = started.elapsed();
    let name = session.inputs()[0].name().to_string();
    let values: Vec<f32> = (0..shape.iter().product::<usize>())
        .map(|i| ((i * 31 % 255) as f32) / 255.0)
        .collect();
    let mut times = Vec::new();
    let mut checksum = 0.0f32;
    for _ in 0..runs + 3 {
        let input = Tensor::from_array((shape.clone(), values.clone())).unwrap();
        let started = Instant::now();
        let outputs = session
            .run(ort::inputs![name.as_str() => input])
            .expect("run");
        let (_, answer) = outputs[0].try_extract_tensor::<f32>().unwrap();
        checksum = answer.iter().take(1000).sum::<f32>();
        if let Ok(path) = std::env::var("PROBE_OUT") {
            let bytes: Vec<u8> = answer.iter().flat_map(|v| v.to_le_bytes()).collect();
            std::fs::write(path, bytes).unwrap();
        }
        times.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let measured = &times[3..];
    let mean = measured.iter().sum::<f64>() / measured.len() as f64;
    println!(
        "{provider}: load {:.0} ms, first run {:.0} ms, mean of {runs} runs {mean:.1} ms, checksum {checksum:.3}",
        load.as_secs_f64() * 1000.0,
        times[0]
    );
}

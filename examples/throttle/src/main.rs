// RUST_LOG=trace cargo run -p debounce
fn main() {
    println!("Hello, world!");
    tracing_subscriber::fmt::init();
    {
        let throttle_fn = fns::throttle(|param: usize| {
            println!("value: {}", param);
        }, std::time::Duration::from_millis(1000));
        throttle_fn.call(1);
        std::thread::sleep(std::time::Duration::from_millis(1100));
        throttle_fn.call(2);
        throttle_fn.call(3);
        std::thread::sleep(std::time::Duration::from_millis(1100));
        throttle_fn.call(4);
        throttle_fn.call(5);
        throttle_fn.terminate();
    }
    std::thread::park();
}

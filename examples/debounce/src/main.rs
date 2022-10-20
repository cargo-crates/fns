// RUST_LOG=trace cargo run -p debounce
fn main() {
    println!("Hello, world!");
    tracing_subscriber::fmt::init();
    {
        let debounce_fn = fns::debounce(|param: usize| {
            println!("{}", param);
        }, std::time::Duration::from_secs(1));
        debounce_fn.call(1);
        debounce_fn.call(2);
        std::thread::sleep(std::time::Duration::from_secs(1));
        debounce_fn.call(3);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    std::thread::park();
}

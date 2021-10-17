#[macro_export]
macro_rules! runnable_cmd {
    ($name: expr) => {
        Box::new($name) as Box<dyn Runnable>
    };
}

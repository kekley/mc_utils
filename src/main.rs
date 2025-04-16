use spider_eye::{ResourceLoader, SpiderEyeError};

extern crate spider_eye;
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `println!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        ::std::println!("[{}:{}]", ::std::file!(), ::std::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                ::std::println!("[{}:{}] {} = {:#?}",
                    ::std::file!(), ::std::line!(), ::std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(::std::dbg!($val)),+,)
    };
}

fn main() -> Result<(), SpiderEyeError> {
    let loader = ResourceLoader::new();
    let a = loader.load_block_states("cobblestone");
    dbg!(a);
    Ok(())
}

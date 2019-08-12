#[macro_export]
macro_rules! hashmap
{
    ($( $key: expr => $val: expr ),*) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key, $val); )*
        map
    }}
}

#[macro_export]
macro_rules! p {
    ($left:expr, $right:expr) => {
        println!($left, $right);
    };

    ($left:expr) => {
        println!("{}", $left);
    };
}

#[macro_export]
macro_rules! pp {
    ($left:expr, $right:expr) => {
        print!($left, $right);
    };

    ($left:expr) => {
        print!("{}", $left);
    };
}

#[macro_export]
macro_rules! e {
    ($left:expr, $right:expr) => {
        eprintln!($left, $right);
    };

    ($left:expr) => {
        eprintln!($left);
    };
}

#[macro_export]
macro_rules! s {
    ($s: expr) => {
        $s.to_string()
    };

    () => {
        String::new()
    };
}

#[macro_export]
macro_rules! d {
    ($left:expr) => {
        dbg!($left);
    };
}

macro_rules! unpack_structure {
    ($format:expr, $source:expr) => {
        structure!($format).unpack($source).chain_err(|| "failed to unpack defined structure")?
    }
}

macro_rules! pack_structure {
    ($format:expr, $($input:expr),*) => {
        structure!($format).pack($($input),*).chain_err(|| "failed to pack defined structure")?
    }
}

macro_rules! boolean {
    ($set:expr) => {
        if $set { 0b1 } else { 0b0 }
    }
}

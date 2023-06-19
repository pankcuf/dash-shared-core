#[macro_export]
macro_rules! unwrap_or_return {
    ($e: expr, $re: expr) => {
        match $e {
            Some(x) => x,
            None => return $re(),
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_failure {
    ($e: expr) => {
        unwrap_or_return!($e, || boxed(types::MNListDiffResultFFI::default()))
    };
}

#[macro_export]
macro_rules! unwrap_or_qr_result_failure {
    ($e: expr) => {
        unwrap_or_return!($e, || boxed(types::QRInfoResultFFI::default()))
    };
}

#[macro_export]
macro_rules! unwrap_or_qr_processing_failure {
    ($e: expr) => {
        unwrap_or_return!($e, || crate::processing::QRInfoResult::default())
    };
}

#[macro_export]
macro_rules! unwrap_or_diff_processing_failure {
    ($e: expr) => {
        unwrap_or_return!($e, || crate::processing::MNListDiffResult::default())
    };
}

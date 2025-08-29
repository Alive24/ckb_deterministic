//! Enhanced debug utilities for CKB smart contracts
//! 
//! This module provides enhanced debug macros that automatically include
//! context information such as file name, line number, and module path.

extern crate alloc;

/// Helper function to convert absolute paths to relative paths
#[doc(hidden)]
#[macro_export]
macro_rules! relative_file_path {
    () => {{
        const FILE_PATH: &str = file!();
        // Try to find common project roots and make paths relative
        // Generic approach: find the last occurrence of a known project structure
        if let Some(src_idx) = FILE_PATH.rfind("/src/") {
            // Find the project name before /src/
            let before_src = &FILE_PATH[..src_idx];
            if let Some(proj_idx) = before_src.rfind('/') {
                &FILE_PATH[proj_idx + 1..]
            } else {
                &FILE_PATH[src_idx + 1..]
            }
        } else if let Some(tests_idx) = FILE_PATH.rfind("/tests/") {
            // Find the project name before /tests/
            let before_tests = &FILE_PATH[..tests_idx];
            if let Some(proj_idx) = before_tests.rfind('/') {
                &FILE_PATH[proj_idx + 1..]
            } else {
                &FILE_PATH[tests_idx + 1..]
            }
        } else if FILE_PATH.rfind("/target/").is_some() {
            // For compiled code in target directory, just use the filename
            FILE_PATH.rsplit('/').next().unwrap_or(FILE_PATH)
        } else {
            // Last resort: just use the filename
            FILE_PATH.rsplit('/').next().unwrap_or(FILE_PATH)
        }
    }};
}

/// Enhanced debug macro that automatically includes context information
/// 
/// # Usage Examples
/// 
/// ```ignore
/// // Simple message
/// debug_info!("Transaction validation started");
/// // Output: [INFO | main.rs:42 | my_module] Transaction validation started
/// 
/// // With formatting
/// debug_info!("Processing {} cells", cell_count);
/// // Output: [INFO | main.rs:43 | my_module] Processing 5 cells
/// 
/// // With key-value pairs
/// debug_info!("method_path" => method_path);
/// // Output: [INFO | main.rs:44 | my_module] method_path = b"openVault"
/// 
/// // With contract name prefix (set via constant)
/// const CONTRACT_NAME: &str = "CDP_VAULT";
/// debug_info!(CONTRACT_NAME, "Vault opened successfully");
/// // Output: [INFO | CDP_VAULT | main.rs:47 | my_module] Vault opened successfully
/// ```
#[macro_export]
macro_rules! debug_info {
    // Pattern: debug_info!(CONTRACT_NAME, message)
    ($contract:expr, $msg:literal) => {
        ::ckb_std::debug!(
            "[INFO | {} | {}:{} | {}] {}",
            $contract,
            $crate::relative_file_path!(),
            line!(),
            module_path!(),
            $msg
        )
    };
    
    // Pattern: debug_info!(CONTRACT_NAME, format, args...)
    ($contract:expr, $fmt:literal, $($arg:expr),+ $(,)?) => {
        {
            extern crate alloc;
            use alloc::format;
            let _msg = format!($fmt, $($arg),+);
            ::ckb_std::debug!(
                "[INFO | {} | {}:{} | {}] {}",
                $contract,
                $crate::relative_file_path!(),
                line!(),
                module_path!(),
                _msg
            );
        }
    };
    
    // Pattern: debug_info!("key" => value)
    ($key:literal => $value:expr) => {
        ::ckb_std::debug!(
            "[INFO | {}:{} | {}] {} = {:?}",
            $crate::relative_file_path!(),
            line!(),
            module_path!(),
            $key,
            $value
        )
    };
    
    // Pattern: debug_info!(message)
    ($msg:literal) => {
        ::ckb_std::debug!(
            "[INFO | {}:{} | {}] {}",
            $crate::relative_file_path!(),
            line!(),
            module_path!(),
            $msg
        )
    };
    
    // Pattern: debug_info!(format, args...)
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        {
            extern crate alloc;
            use alloc::format;
            let _msg = format!($fmt, $($arg),+);
            ::ckb_std::debug!(
                "[INFO | {}:{} | {}] {}",
                $crate::relative_file_path!(),
                line!(),
                module_path!(),
                _msg
            );
        }
    };
}

/// Debug macro for error conditions with automatic context
/// 
/// # Usage Examples
/// 
/// ```ignore
/// debug_error!("Validation failed");
/// // Output: [ERROR | main.rs:42 | my_module] Validation failed
/// 
/// debug_error!("Validation failed: {:?}", error);
/// // Output: [ERROR | main.rs:42 | my_module] Validation failed: InvalidSignature
/// 
/// const CONTRACT_NAME: &str = "CDP_VAULT";
/// debug_error!(CONTRACT_NAME, "Critical failure in {}", function_name);
/// // Output: [ERROR | CDP_VAULT | main.rs:43 | my_module] Critical failure in open_vault
/// ```
#[macro_export]
macro_rules! debug_error {
    // Pattern: debug_error!(CONTRACT_NAME, message)
    ($contract:expr, $msg:literal) => {
        ::ckb_std::debug!(
            "[ERROR | {} | {}:{} | {}] {}",
            $contract,
            $crate::relative_file_path!(),
            line!(),
            module_path!(),
            $msg
        )
    };
    
    // Pattern: debug_error!(CONTRACT_NAME, format, args...)
    ($contract:expr, $fmt:literal, $($arg:expr),+ $(,)?) => {
        {
            extern crate alloc;
            use alloc::format;
            let _msg = format!($fmt, $($arg),+);
            ::ckb_std::debug!(
                "[ERROR | {} | {}:{} | {}] {}",
                $contract,
                $crate::relative_file_path!(),
                line!(),
                module_path!(),
                _msg
            );
        }
    };
    
    // Pattern: debug_error!(message)
    ($msg:literal) => {
        ::ckb_std::debug!(
            "[ERROR | {}:{} | {}] {}",
            $crate::relative_file_path!(),
            line!(),
            module_path!(),
            $msg
        )
    };
    
    // Pattern: debug_error!(format, args...)
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        {
            extern crate alloc;
            use alloc::format;
            let _msg = format!($fmt, $($arg),+);
            ::ckb_std::debug!(
                "[ERROR | {}:{} | {}] {}",
                $crate::relative_file_path!(),
                line!(),
                module_path!(),
                _msg
            );
        }
    };
}

/// Debug macro for function entry/exit tracing
/// 
/// # Usage Examples
/// 
/// ```ignore
/// debug_trace!("ENTER");
/// // Output: [TRACE | main.rs:42 | my_module::my_function] ENTER
/// 
/// debug_trace!("EXIT => {:?}", result);
/// // Output: [TRACE | main.rs:43 | my_module::my_function] EXIT => Ok(())
/// ```
#[macro_export]
macro_rules! debug_trace {
    // Pattern: debug_trace!(message)
    ($msg:literal) => {
        ::ckb_std::debug!(
            "[TRACE |{}:{}] {}",
            $crate::relative_file_path!(),
            line!(),
            $msg
        )
    };
    
    // Pattern: debug_trace!(format, args...)
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        {
            extern crate alloc;
            use alloc::format;
            let _msg = format!($fmt, $($arg),+);
            ::ckb_std::debug!(
                "[TRACE |{}:{}] {}",
                $crate::relative_file_path!(),
                line!(),
                _msg
            );
        }
    };
}

/// Helper macro to conditionally include debug output based on a debug level
/// 
/// This allows for more granular control over debug output verbosity
/// 
/// # Usage Examples
/// 
/// ```ignore
/// const DEBUG_LEVEL: u8 = 2;
/// 
/// debug_if!(DEBUG_LEVEL >= 1, "Basic info");  // Will print
/// debug_if!(DEBUG_LEVEL >= 3, "Verbose details");  // Won't print
/// ```
#[macro_export]
macro_rules! debug_if {
    ($condition:expr, $($arg:tt)*) => {
        if $condition {
            $crate::debug_info!($($arg)*);
        }
    };
}
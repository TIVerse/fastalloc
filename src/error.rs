//! Error types for the fastalloc crate.

use core::fmt;

/// Result type alias using the fastalloc error type.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur when working with memory pools.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The pool has reached its capacity and cannot allocate more objects.
    PoolExhausted {
        /// Current capacity of the pool
        capacity: usize,
        /// Number of objects currently allocated
        allocated: usize,
    },
    
    /// Invalid configuration was provided when building a pool.
    InvalidConfiguration {
        /// Description of what was invalid
        message: &'static str,
    },
    
    /// Attempted to perform an operation on an uninitialized pool.
    UninitializedPool,
    
    /// An alignment value was invalid (must be a power of two).
    InvalidAlignment {
        /// The invalid alignment value
        alignment: usize,
    },
    
    /// Maximum capacity would be exceeded by the requested growth.
    MaxCapacityExceeded {
        /// Current capacity
        current: usize,
        /// Requested capacity
        requested: usize,
        /// Maximum allowed capacity
        max: usize,
    },
    
    /// A handle reference was invalid or expired.
    InvalidHandle,
    
    /// Attempted to free an object that was already freed (double-free).
    DoubleFree,
    
    /// Memory allocation from the system allocator failed.
    AllocationFailed,
    
    /// Custom error with a message (for extensibility).
    Custom {
        /// Error message
        message: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PoolExhausted { capacity, allocated } => {
                write!(
                    f,
                    "Pool exhausted: allocated {}/{} objects. Consider using a growing pool or increasing capacity.",
                    allocated, capacity
                )
            }
            Error::InvalidConfiguration { message } => {
                write!(f, "Invalid pool configuration: {}", message)
            }
            Error::UninitializedPool => {
                write!(f, "Attempted to use an uninitialized pool")
            }
            Error::InvalidAlignment { alignment } => {
                write!(
                    f,
                    "Invalid alignment: {}. Alignment must be a power of two.",
                    alignment
                )
            }
            Error::MaxCapacityExceeded { current, requested, max } => {
                write!(
                    f,
                    "Maximum capacity exceeded: current={}, requested={}, max={}",
                    current, requested, max
                )
            }
            Error::InvalidHandle => {
                write!(f, "Invalid or expired handle")
            }
            Error::DoubleFree => {
                write!(f, "Attempted to free an already freed object (double-free)")
            }
            Error::AllocationFailed => {
                write!(f, "System memory allocation failed")
            }
            Error::Custom { message } => {
                write!(f, "Error: {}", message)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl Error {
    /// Creates a new invalid configuration error.
    #[inline]
    pub fn invalid_config(message: &'static str) -> Self {
        Error::InvalidConfiguration { message }
    }
    
    /// Creates a new custom error.
    #[inline]
    pub fn custom(message: &'static str) -> Self {
        Error::Custom { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn error_display() {
        let err = Error::PoolExhausted {
            capacity: 100,
            allocated: 100,
        };
        assert!(err.to_string().contains("exhausted"));
        
        let err = Error::InvalidConfiguration {
            message: "capacity must be positive",
        };
        assert!(err.to_string().contains("capacity must be positive"));
        
        let err = Error::InvalidAlignment { alignment: 7 };
        assert!(err.to_string().contains("power of two"));
    }
    
    #[test]
    fn error_helpers() {
        let err = Error::invalid_config("test");
        assert!(matches!(err, Error::InvalidConfiguration { .. }));
        
        let err = Error::custom("custom message");
        assert!(matches!(err, Error::Custom { .. }));
    }
}

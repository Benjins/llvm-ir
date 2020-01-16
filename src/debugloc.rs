use std::cmp::{PartialOrd, Ordering};

/// Describes a "debug location" (source location)
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct DebugLoc {
    /// The source line number
    pub line: u32,
    /// The source column number
    ///
    /// `Instruction`s and `Terminator`s have this info (and will have `Some`
    /// here), while `GlobalVariable`s and `Function`s do not have this info (and
    /// will have `None`)
    pub col: Option<u32>,
    /// The source filename
    pub filename: String,
    /// The source directory, if available
    pub directory: Option<String>,
}

impl PartialOrd for DebugLoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // compare in the order (directory, filename, line, col)
        Some(
            (&self.directory, &self.filename, &self.line, &self.col)
                .cmp(&(&other.directory, &other.filename, &other.line, &other.col))
        )
    }
}

impl Ord for DebugLoc {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare in the order (directory, filename, line, col)
        (&self.directory, &self.filename, &self.line, &self.col)
            .cmp(&(&other.directory, &other.filename, &other.line, &other.col))
    }
}

pub trait HasDebugLoc {
    /// Returns the `DebugLoc` associated with the given `Instruction`,
    /// `Terminator`, `GlobalVariable`, or `Function`; or `None` if it doesn't
    /// have a `DebugLoc`.
    ///
    /// Reasons something might not have a `DebugLoc` include:
    ///     - the file was compiled without debuginfo
    ///     - for an `Instruction`, it might not directly correspond to any source
    ///     line. For instance, it may be just setting up the stack frame for a
    ///     function.
    fn get_debug_loc(&self) -> &Option<DebugLoc>;
}

// ********* //
// from_llvm //
// ********* //

use crate::from_llvm::*;

impl DebugLoc {
    /// `value`: must represent an Instruction, Terminator, GlobalVariable, or Function
    ///
    /// Returns `None` if the object does not have a `DebugLoc`
    pub(crate) fn from_llvm_no_col(value: LLVMValueRef) -> Option<Self> {
        match unsafe { get_debugloc_filename(value) } {
            None => None,  // if no filename, assume no debugloc. To my knowledge, everything with a debugloc has a filename.
            Some(filename) => Some(Self {
                line: unsafe { LLVMGetDebugLocLine(value) },
                col: None,
                filename,
                directory: unsafe { get_debugloc_directory(value) },
            }),
        }
    }

    /// `value`: must represent an Instruction or Terminator
    ///
    /// Returns `None` if the object does not have a `DebugLoc`
    pub(crate) fn from_llvm_with_col(value: LLVMValueRef) -> Option<Self> {
        match Self::from_llvm_no_col(value) {
            Some(mut debugloc) => {
                debugloc.col = Some(unsafe { LLVMGetDebugLocColumn(value) });
                Some(debugloc)
            },
            None => None,
        }
    }
}

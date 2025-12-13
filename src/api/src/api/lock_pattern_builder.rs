use protocl::types::{LockAction, LockableMemoryBank, PasswordLockAction};

/// Builder for creating complex lock patterns
pub struct LockPatternBuilder {}

impl LockPatternBuilder {
    /// Create a lock pattern u16 value for a specific memory bank and action.
    #[must_use]
    pub fn memory_bank(bank: LockableMemoryBank, action: LockAction, apply_mask: bool) -> u16 {
        let mut pattern = 0u16;

        // Determine bit positions based on the memory bank
        let (data_bit_pos, perm_bit_pos) = match bank {
            LockableMemoryBank::User => (8, 9),
            LockableMemoryBank::Tid => (6, 7),
            LockableMemoryBank::Epc => (4, 5),
            LockableMemoryBank::AccessPassword => (2, 3),
            LockableMemoryBank::KillPassword => (0, 1),
        };

        // Set appropriate bits based on the lock action
        match action {
            LockAction::Writeable => {
                // Both bits 0 = writeable without a password
            }
            LockAction::PermanentlyWriteable => {
                // Set permanent bit only
                pattern |= 1 << perm_bit_pos;
            }
            LockAction::SecureWriteable => {
                // Set data bit only
                pattern |= 1 << data_bit_pos;
            }
            LockAction::NotWriteable => {
                // Set both bits
                pattern |= (1 << data_bit_pos) | (1 << perm_bit_pos);
            }
        }

        // Set mask bits as well.
        if apply_mask {
            // Set mask bits for the modifying bits
            pattern |= (1 << (data_bit_pos + 10)) | (1 << (perm_bit_pos + 10));
        }

        pattern
    }

    /// Create a password lock pattern as an u16 value for password memory banks with special lock actions.
    #[must_use]
    /// # Panics
    /// Panics if `bank` is not `AccessPassword` or `KillPassword`.
    pub fn password(bank: LockableMemoryBank, action: PasswordLockAction, apply_mask: bool) -> u16 {
        let mut pattern = 0u16;

        // Only Access and Kill passwords valid for this function
        let (read_bit_pos, write_bit_pos) = match bank {
            LockableMemoryBank::AccessPassword => (2, 3),
            LockableMemoryBank::KillPassword => (0, 1),
            _ => panic!("Only AccessPassword and KillPassword can be used with PasswordLockAction"),
        };

        // Set appropriate bits based on the lock action
        match action {
            PasswordLockAction::ReadWriteable => {
                // Both bits 0 = readable/writeable without a password
            }
            PasswordLockAction::PermanentlyReadWriteable => {
                // Set permanent bits for both read and write
                pattern |= 1 << write_bit_pos;
            }
            PasswordLockAction::SecureReadWriteable => {
                // Set data bits for both read and write
                pattern |= 1 << read_bit_pos;
            }
            PasswordLockAction::NotReadWriteable => {
                // Set both bits for both read and write
                pattern |= (1 << read_bit_pos) | (1 << write_bit_pos);
            }
        }

        // Set mask bits as well.
        if apply_mask {
            // Set mask bits for the modifying bits
            pattern |= (1 << (read_bit_pos + 10)) | (1 << (write_bit_pos + 10));
        }

        pattern
    }
}

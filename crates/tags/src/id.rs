#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TagId(pub(super) u16);

impl TagId {
    pub const NONE: Self = Self(u16::MAX);

    #[inline]
    pub fn bit_index(self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for TagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ArchetypeId(pub(super) u8);

impl ArchetypeId {
    pub const NONE: Self = Self(u8::MAX);
}

impl std::fmt::Display for ArchetypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArchetypeId({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_id_none_is_max_u16() {
        assert_eq!(TagId::NONE.0, u16::MAX);
    }

    #[test]
    fn tag_id_bit_index() {
        let id = TagId(42);
        assert_eq!(id.bit_index(), 42);
    }

    #[test]
    fn tag_id_display() {
        assert_eq!(format!("{}", TagId(7)), "TagId(7)");
        assert_eq!(format!("{}", TagId::NONE), format!("TagId({})", u16::MAX));
    }

    #[test]
    fn archetype_id_none_is_max_u8() {
        assert_eq!(ArchetypeId::NONE.0, u8::MAX);
    }

    #[test]
    fn archetype_id_display() {
        assert_eq!(format!("{}", ArchetypeId(3)), "ArchetypeId(3)");
        assert_eq!(format!("{}", ArchetypeId::NONE), format!("ArchetypeId({})", u8::MAX));
    }

    #[test]
    fn tag_id_ordering() {
        assert!(TagId(1) < TagId(2));
        assert_eq!(TagId(5), TagId(5));
    }
}

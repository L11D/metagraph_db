use std::{collections::{HashMap, HashSet}, fmt};

use bincode::{Decode, Encode};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    attribute::Attribute,
    metagraph::ChildConstraint,
    storage::{SlottedRecord, StorageError, common::PageDataType, manager::FixedRecord},
};

// ---------------------------------------------------------------------------
// VertRecord
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy, Default, PartialEq)]
pub struct VertRecord {
    attr_record_id: SlottedRecordIdRaw,
    next_edge_id: SlottedRecordIdRaw,
    parent: SlottedRecordIdRaw,
}
impl FixedRecord for VertRecord {
    const PAGE_DATA_TYPE: PageDataType = PageDataType::Vert;

    const FSM_PAGE_DATA_TYPE: PageDataType = PageDataType::VertFsm;
}
impl VertRecord {
    pub fn new(
        attr_record_id: SlottedRecordId,
        next_edge_id: SlottedRecordId,
        parent: SlottedRecordId,
    ) -> Self {
        Self {
            attr_record_id: attr_record_id.into(),
            next_edge_id: next_edge_id.into(),
            parent: parent.into(),
        }
    }
    pub fn attr_record_id(&self) -> SlottedRecordId {
        self.attr_record_id.into()
    }
    pub fn next_edge_id(&self) -> SlottedRecordId {
        self.next_edge_id.into()
    }
    pub fn parent(&self) -> SlottedRecordId {
        self.parent.into()
    }

    pub fn set_next_edge_id(&mut self, id: SlottedRecordId) {
        self.next_edge_id = id.into();
    }

    pub fn set_attr_id(&mut self, id: SlottedRecordId) {
        self.attr_record_id = id.into();
    }
}

// ---------------------------------------------------------------------------
// EdgeRecord
// ---------------------------------------------------------------------------
pub enum EdgeDestination {
    Vert(SlottedRecordId),
    Meta(SlottedRecordId),
}
impl EdgeDestination {
    pub fn id(&self) -> SlottedRecordId {
        match *self {
            EdgeDestination::Vert(id) | EdgeDestination::Meta(id) => id,
        }
    }
}

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeRecord {
    from: SlottedRecordIdRaw,
    to: SlottedRecordIdRaw,
    destination: u8,
    attr_record_id: SlottedRecordIdRaw,
    from_next_edge_id: SlottedRecordIdRaw,
    to_next_edge_id: SlottedRecordIdRaw,
}
impl EdgeRecord {

    pub fn new_last(
        from: EdgeDestination,
        to: EdgeDestination,
        attr_record_id: SlottedRecordId,
    ) -> Self {
        Self::new(from, to, attr_record_id, SlottedRecordId::Empty, SlottedRecordId::Empty)
    }

    pub fn new(
        from: EdgeDestination,
        to: EdgeDestination,
        attr_record_id: SlottedRecordId,
        from_next_edge_id: SlottedRecordId,
        to_next_edge_id: SlottedRecordId,
    ) -> Self {
        let mut destination = 0u8;

        let from_record = match from {
            EdgeDestination::Vert(id) => id,
            EdgeDestination::Meta(id) => {
                destination |= 1 << 0;
                id
            }
        };

        let to_record = match to {
            EdgeDestination::Vert(id) => id,
            EdgeDestination::Meta(id) => {
                destination |= 1 << 1;
                id
            }
        };

        Self {
            from: from_record.into(),
            to: to_record.into(),
            destination: destination,
            attr_record_id: attr_record_id.into(),
            from_next_edge_id: from_next_edge_id.into(),
            to_next_edge_id: to_next_edge_id.into(),
        }
    }

    pub fn from(&self) -> EdgeDestination {
        if (self.destination & (1 << 0)) != 0 {
            EdgeDestination::Meta(self.from.into())
        } else {
            EdgeDestination::Vert(self.from.into())
        }
    }

    pub fn to(&self) -> EdgeDestination {
        if (self.destination & (1 << 1)) != 0 {
            EdgeDestination::Meta(self.to.into())
        } else {
            EdgeDestination::Vert(self.to.into())
        }
    }

    pub fn attr_record_id(&self) -> SlottedRecordId {
        self.attr_record_id.into()
    }

    pub fn from_next_edge_id(&self) -> SlottedRecordId {
        self.from_next_edge_id.into()
    }

    pub fn to_next_edge_id(&self) -> SlottedRecordId {
        self.to_next_edge_id.into()
    }

    pub fn set_from_next_edge_id(&mut self, id: SlottedRecordId) {
        self.from_next_edge_id = id.into();
    }

    pub fn set_attr_id(&mut self, id: SlottedRecordId) {
        self.attr_record_id = id.into();
    }

}
impl FixedRecord for EdgeRecord {
    const PAGE_DATA_TYPE: PageDataType = PageDataType::Edge;

    const FSM_PAGE_DATA_TYPE: PageDataType = PageDataType::EdgeFsm;
}

// ---------------------------------------------------------------------------
// MetaRecord
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy, Default, PartialEq)]
pub struct MetaRecord {
    vert: VertRecord,
    meta_info_id: SlottedRecordIdRaw,
}
impl MetaRecord {
    pub fn new(
        attr_record_id: SlottedRecordId,
        next_edge_id: SlottedRecordId,
        parent: SlottedRecordId,
        meta_info_id: SlottedRecordId,
    ) -> Self {
        let vert = VertRecord::new(attr_record_id, next_edge_id, parent);
        Self {
            vert,
            meta_info_id: meta_info_id.into(),
        }
    }
    pub fn attr_record_id(&self) -> SlottedRecordId {
        self.vert.attr_record_id.into()
    }
    pub fn next_edge_id(&self) -> SlottedRecordId {
        self.vert.next_edge_id.into()
    }
    pub fn parent(&self) -> SlottedRecordId {
        self.vert.parent.into()
    }
    pub fn meta_info_id(&self) -> SlottedRecordId {
        self.meta_info_id.into()
    }

    pub fn set_meta_info_id(&mut self, id: SlottedRecordId) {
        self.meta_info_id = id.into();
    }

    pub fn set_attr_id(&mut self, id: SlottedRecordId) {
        self.vert.set_attr_id(id);
    }

}
impl FixedRecord for MetaRecord {
    const PAGE_DATA_TYPE: PageDataType = PageDataType::Meta;

    const FSM_PAGE_DATA_TYPE: PageDataType = PageDataType::MetaFsm;
}

// ---------------------------------------------------------------------------
// AttributesRecord
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct AttributesRecord {
    pub attrs: HashMap<String, Attribute>,
}
impl AttributesRecord {
    pub fn new(attrs: HashMap<String, Attribute>) -> Self {
        assert!(!attrs.is_empty(), "attrs map must not be empty.");
        Self { attrs }
    }
    pub fn new_empty() -> Self {
        Self { attrs: HashMap::new() }
    }
}
impl SlottedRecord for AttributesRecord {
    const PAGE_DATA_TYPE: PageDataType = PageDataType::Attribute;

    const FSM_PAGE_DATA_TYPE: PageDataType = PageDataType::AttributeFsm;
}

// ---------------------------------------------------------------------------
// ParentsRecord
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct ParentsRecord {
    pub parents: HashSet<SlottedRecordId>,
}
impl ParentsRecord {
    pub fn new(parents: HashSet<SlottedRecordId>) -> Self {
        assert!(!parents.is_empty(), "Parents set must not be empty.");
        Self { parents }
    }
}
impl SlottedRecord for ParentsRecord {
    const PAGE_DATA_TYPE: PageDataType = PageDataType::Parents;

    const FSM_PAGE_DATA_TYPE: PageDataType = PageDataType::ParentsFsm;
}

// ---------------------------------------------------------------------------
// MetaInfoRecord
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct MetaInfoRecord {
    pub v_children: HashSet<SlottedRecordId>,
    pub mv_children: HashSet<SlottedRecordId>,
    pub constraints: HashSet<ChildConstraint>,
}
impl MetaInfoRecord {
    pub fn new(
        v_children: HashSet<SlottedRecordId>,
        mv_children: HashSet<SlottedRecordId>,
        constraints: HashSet<ChildConstraint>,
    ) -> Self {
        assert!(
            v_children.is_empty() || mv_children.is_empty() || constraints.is_empty(),
            "MetaInfoRecord must not be empty."
        );

        Self {
            v_children,
            mv_children,
            constraints: constraints,
        }
    }

    pub fn add_v_children(&mut self, v_children: HashSet<SlottedRecordId>) {
        self.v_children.extend(v_children);
    }

    pub fn remove_v_children(&mut self, v_children: HashSet<SlottedRecordId>) {
        self.v_children.retain(|id| !v_children.contains(id));
    }

    pub fn add_mv_children(&mut self, mv_children: HashSet<SlottedRecordId>) {
        self.mv_children.extend(mv_children);
    }

    pub fn remove_mv_children(&mut self, mv_children: HashSet<SlottedRecordId>) {
        self.mv_children.retain(|id| !mv_children.contains(id));
    }

}
impl SlottedRecord for MetaInfoRecord {
    const PAGE_DATA_TYPE: PageDataType = PageDataType::MetaInfo;

    const FSM_PAGE_DATA_TYPE: PageDataType = PageDataType::MetaInfoFsm;
}

// ---------------------------------------------------------------------------
// SlottedRecordId
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(
    FromBytes,
    IntoBytes,
    KnownLayout,
    Immutable,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Encode,
    Decode,
    Hash,
)]
pub struct SlottedRecordIdRaw {
    data: [u8; 6],
}
impl SlottedRecordIdRaw {
    // Маски для последнего байта (data[5])
    const EMPTY_MASK: u8 = 0b1000_0000; // Последний бит
    const LINK_MASK: u8 = 0b0100_0000; // Предпоследний бит

    fn new(page_id: u32, slot_id: u16) -> Self {
        assert!(slot_id < 16384, "slot_id exceeds 14 bits");
        let mut data = [0u8; 6];
        data[0..4].copy_from_slice(&page_id.to_le_bytes());
        data[4..6].copy_from_slice(&slot_id.to_le_bytes());
        Self { data }
    }

    fn new_link(page_id: u32, slot_id: u16) -> Self {
        let mut s = Self::new(page_id, slot_id);
        s.data[5] |= Self::LINK_MASK;
        s
    }

    fn new_empty() -> Self {
        let mut s = Self { data: [0u8; 6] };
        s.data[5] |= Self::EMPTY_MASK;
        s
    }

    fn page_id(&self) -> u32 {
        u32::from_le_bytes(self.data[0..4].try_into().unwrap())
    }

    fn slot_id(&self) -> u16 {
        // Очищаем биты флагов перед чтением slot_id
        let mut raw_slot = [self.data[4], self.data[5]];
        raw_slot[1] &= !(Self::EMPTY_MASK | Self::LINK_MASK);
        u16::from_le_bytes(raw_slot)
    }

    /// Определяет тип записи на основе битовых флагов
    fn get_type(&self) -> SlottedRecordId {
        if (self.data[5] & Self::EMPTY_MASK) != 0 {
            SlottedRecordId::Empty
        } else if (self.data[5] & Self::LINK_MASK) != 0 {
            SlottedRecordId::Link(*self)
        } else {
            SlottedRecordId::Id(*self)
        }
    }

    fn to_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        buf[0..6].copy_from_slice(&self.data);
        u64::from_le_bytes(buf)
    }

    fn from_u64(val: u64) -> Self {
        let buf = val.to_le_bytes();
        let mut data = [0u8; 6];
        data.copy_from_slice(&buf[0..6]);
        Self { data }
    }

}
impl fmt::Debug for SlottedRecordIdRaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_empty = (self.data[5] & Self::EMPTY_MASK) != 0;
        let is_link = (self.data[5] & Self::LINK_MASK) != 0;

        f.debug_struct("SlottedRecordIdRaw")
            .field("page_id", &self.page_id())
            .field("slot_id", &self.slot_id())
            .field("is_empty", &is_empty)
            .field("is_link", &is_link)
            .field("raw", &format_args!("{:02X?}", self.data))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Hash)]
pub enum SlottedRecordId {
    Id(SlottedRecordIdRaw),
    Link(SlottedRecordIdRaw),
    Empty,
}

impl SlottedRecordId {
    pub fn new(page_id: u32, slot_id: u16) -> Self {
        Self::Id(SlottedRecordIdRaw::new(page_id, slot_id))
    }

    pub fn new_link(page_id: u32, slot_id: u16) -> Self {
        Self::Link(SlottedRecordIdRaw::new_link(page_id, slot_id))
    }

    pub fn new_empty() -> Self {
        Self::Empty
    }

    pub fn page_id(&self) -> u32 {
        match self {
            Self::Id(id) | Self::Link(id) => id.page_id(),
            _ => 0,
        }
    }

    pub fn slot_id(&self) -> u16 {
        match self {
            Self::Id(id) | Self::Link(id) => id.slot_id(),
            _ => 0,
        }
    }

    pub fn make_error_if_empty(&self) -> Result<(), StorageError> {
        match self {
            Self::Id(_) | Self::Link(_) => Ok(()),
            Self::Empty => Err(StorageError::UsingEmptyId),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    pub fn human_readable(&self) -> String {
        match self {
            SlottedRecordId::Id(id) => id.to_u64().to_string(),
            SlottedRecordId::Link(_) => "link".to_string(),
            SlottedRecordId::Empty => "empty".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, StorageError> {
        let val = s.parse::<u64>().map_err(|_| StorageError::InvalidId(s.to_string()))?;
        Ok(SlottedRecordId::Id(SlottedRecordIdRaw::from_u64(val)))
    }

}

// Методы конвертации
impl From<SlottedRecordIdRaw> for SlottedRecordId {
    fn from(raw: SlottedRecordIdRaw) -> Self {
        raw.get_type()
    }
}

impl From<SlottedRecordId> for SlottedRecordIdRaw {
    fn from(id: SlottedRecordId) -> Self {
        match id {
            SlottedRecordId::Id(raw) => raw,
            SlottedRecordId::Link(raw) => raw,
            SlottedRecordId::Empty => SlottedRecordIdRaw::new_empty(),
        }
    }
}

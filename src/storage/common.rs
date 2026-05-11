
pub const PAGE_SIZE: usize = 4096;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum PageDataType {
    Attribute,
    Parents,
    MetaInfo,

    Vert,
    Meta,
    Edge,

    AttributeFsm,
    ParentsFsm,
    MetaInfoFsm,

    VertFsm,
    MetaFsm,
    EdgeFsm,
}
impl PageDataType {
    pub const ALL: &'static [PageDataType] = &[
        PageDataType::Attribute,
        PageDataType::Parents,
        PageDataType::MetaInfo,
        PageDataType::Vert,
        PageDataType::Meta,
        PageDataType::Edge,
        PageDataType::AttributeFsm,
        PageDataType::ParentsFsm,
        PageDataType::MetaInfoFsm,
        PageDataType::VertFsm,
        PageDataType::MetaFsm,
        PageDataType::EdgeFsm,
    ];

    pub const FIXED: &'static [PageDataType] = &[
        PageDataType::Vert,
        PageDataType::Meta,
        PageDataType::Edge,
        PageDataType::AttributeFsm,
        PageDataType::ParentsFsm,
        PageDataType::MetaInfoFsm,
        PageDataType::VertFsm,
        PageDataType::MetaFsm,
        PageDataType::EdgeFsm,
    ];

    pub const SLOTTED: &'static [PageDataType] = &[
        PageDataType::Attribute, 
        PageDataType::Parents,
        PageDataType::MetaInfo,
    ];
}


//! Fixturators for zome types

use ::fixt::prelude::*;
use holo_hash::EntryHash;
use holochain_serialized_bytes::prelude::SerializedBytes;

use crate::entry_def::EntryVisibility;
use crate::header::*;
use crate::link::LinkTag;
use crate::timestamp::Timestamp;

pub use holo_hash::fixt::*;

fixturator!(
    Timestamp;
    curve Empty (
        Timestamp(I64Fixturator::new(Empty).next().unwrap(), U32Fixturator::new(Empty).next().unwrap())
    );
    curve Unpredictable (
        Timestamp(I64Fixturator::new(Unpredictable).next().unwrap(), U32Fixturator::new(Unpredictable).next().unwrap())
    );
    curve Predictable (
        Timestamp(I64Fixturator::new(Predictable).next().unwrap(), U32Fixturator::new(Predictable).next().unwrap())
    );
);

fixturator!(
    EntryVisibility;
    unit variants [ Public Private ] empty Public;
);

fixturator!(
    AppEntryType;
    constructor fn new(U8, U8, EntryVisibility);
);

impl Iterator for AppEntryTypeFixturator<EntryVisibility> {
    type Item = AppEntryType;
    fn next(&mut self) -> Option<Self::Item> {
        let app_entry = AppEntryTypeFixturator::new(Unpredictable).next().unwrap();
        Some(AppEntryType::new(
            app_entry.id(),
            app_entry.zome_id(),
            self.0.curve,
        ))
    }
}

/// Alias
pub type MaybeSerializedBytes = Option<SerializedBytes>;

fixturator! {
    MaybeSerializedBytes;
    enum [ Some None ];
    curve Empty MaybeSerializedBytes::None;
    curve Unpredictable match MaybeSerializedBytesVariant::random() {
        MaybeSerializedBytesVariant::None => MaybeSerializedBytes::None,
        MaybeSerializedBytesVariant::Some => MaybeSerializedBytes::Some(fixt!(SerializedBytes)),
    };
    curve Predictable match MaybeSerializedBytesVariant::nth(self.0.index) {
        MaybeSerializedBytesVariant::None => MaybeSerializedBytes::None,
        MaybeSerializedBytesVariant::Some => MaybeSerializedBytes::Some(SerializedBytesFixturator::new_indexed(Predictable, self.0.index).next().unwrap()),
    };
}

fixturator! {
    EntryType;
    enum [ AgentPubKey App CapClaim CapGrant ];
    curve Empty EntryType::AgentPubKey;
    curve Unpredictable match EntryTypeVariant::random() {
        EntryTypeVariant::AgentPubKey => EntryType::AgentPubKey,
        EntryTypeVariant::App => EntryType::App(fixt!(AppEntryType)),
        EntryTypeVariant::CapClaim => EntryType::CapClaim,
        EntryTypeVariant::CapGrant => EntryType::CapGrant,
    };
    curve Predictable match EntryTypeVariant::nth(self.0.index) {
        EntryTypeVariant::AgentPubKey => EntryType::AgentPubKey,
        EntryTypeVariant::App => EntryType::App(AppEntryTypeFixturator::new_indexed(Predictable, self.0.index).next().unwrap()),
        EntryTypeVariant::CapClaim => EntryType::CapClaim,
        EntryTypeVariant::CapGrant => EntryType::CapGrant,
    };
}

fixturator!(
    HeaderBuilderCommon;
    constructor fn new(AgentPubKey, Timestamp, u32, HeaderHash);
);

fixturator!(
    AgentValidationPkg;
    constructor fn from_builder(HeaderBuilderCommon, MaybeSerializedBytes);
);

fixturator!(
    InitZomesComplete;
    constructor fn from_builder(HeaderBuilderCommon);
);

fixturator!(
    ChainOpen;
    constructor fn from_builder(HeaderBuilderCommon, DnaHash);
);

fixturator!(
    ChainClose;
    constructor fn from_builder(HeaderBuilderCommon, DnaHash);
);

fixturator!(
    CreateEntry;
    constructor fn from_builder(HeaderBuilderCommon, EntryType, EntryHash);
);

fixturator!(
    EntryUpdate;
    constructor fn from_builder(HeaderBuilderCommon, EntryHash, HeaderHash, EntryType, EntryHash);
);

fixturator!(
    ElementDelete;
    constructor fn from_builder(HeaderBuilderCommon, HeaderHash, EntryHash);
);

fixturator!(
    LinkRemove;
    constructor fn from_builder(HeaderBuilderCommon, HeaderHash, EntryHash);
);

fixturator!(
    LinkAdd;
    constructor fn from_builder(HeaderBuilderCommon, EntryHash, EntryHash, u8, LinkTag);
);

fixturator!(
    LinkTag; from Bytes;
);

pub struct KnownLinkAdd {
    pub base_address: EntryHash,
    pub target_address: EntryHash,
    pub tag: LinkTag,
    pub zome_id: ZomeId,
}

pub struct KnownLinkRemove {
    pub link_add_address: holo_hash::HeaderHash,
}

impl Iterator for LinkAddFixturator<KnownLinkAdd> {
    type Item = LinkAdd;
    fn next(&mut self) -> Option<Self::Item> {
        let mut f = fixt!(LinkAdd);
        f.base_address = self.0.curve.base_address.clone();
        f.target_address = self.0.curve.target_address.clone();
        f.tag = self.0.curve.tag.clone();
        f.zome_id = self.0.curve.zome_id;
        Some(f)
    }
}

impl Iterator for LinkRemoveFixturator<KnownLinkRemove> {
    type Item = LinkRemove;
    fn next(&mut self) -> Option<Self::Item> {
        let mut f = fixt!(LinkRemove);
        f.link_add_address = self.0.curve.link_add_address.clone();
        Some(f)
    }
}
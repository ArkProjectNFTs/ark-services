pub(crate) mod sql_order_event_type {
    pub const PLACED: &str = "Placed";
    pub const CANCELLED: &str = "Cancelled";
    pub const FULFILLED: &str = "Fulfilled";
    pub const EXECUTED: &str = "Executed";
    ///
    pub const TYPE_NAME: &str = "order_event_type";
}

pub(crate) mod sql_order_type {
    pub const LISTING: &str = "Listing";
    pub const AUCTION: &str = "Auction";
    pub const OFFER: &str = "Offer";
    pub const COLLECTION_OFFER: &str = "CollectionOffer";
    ///
    pub const TYPE_NAME: &str = "order_type";
}

pub(crate) mod sql_route_type {
    pub const ERC20_TO_ERC721: &str = "Erc20ToErc721";
    pub const ERC721_TO_ERC20: &str = "Erc721ToErc20";
    pub const ERC20_TO_ERC1155: &str = "Erc20ToErc1155";
    pub const ERC1155_TO_ERC20: &str = "Erc1155ToErc20";
    ///
    pub const TYPE_NAME: &str = "route_type";
}

pub(crate) mod sql_cancelled_reason_type {
    pub const USER: &str = "CancelledUser";
    pub const BY_NEW_ORDER: &str = "CancelledByNewOrder";
    pub const ASSET_FAULT: &str = "CancelledAssetFault";
    pub const OWNERSHIP: &str = "CancelledOwnership";
    pub const UNKNOWN: &str = "Unknown";
    ///
    pub const TYPE_NAME: &str = "cancelled_reason_type";
}

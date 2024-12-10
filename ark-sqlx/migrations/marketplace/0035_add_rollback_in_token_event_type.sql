-- Add Rollback to the event type constraint
ALTER TABLE token_event 
DROP CONSTRAINT token_event_event_type_check,
ADD CONSTRAINT token_event_event_type_check CHECK (event_type IN 
    ('Listing', 'CollectionOffer', 'Offer', 'Auction', 'Fulfill', 'Cancelled', 'Executed', 'Sale', 'Mint', 'Burn', 'Transfer', 'Rollback',
        'ListingCancelled', 'AuctionCancelled', 'OfferCancelled', 
        'ListingExpired', 'OfferExpired'
    ));

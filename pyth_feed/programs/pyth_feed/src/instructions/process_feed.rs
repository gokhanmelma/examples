use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::thread_program::accounts::{Thread, ThreadAccount},
    pyth_sdk_solana::load_price_feed_from_account_info,
};

#[derive(Accounts)]
pub struct ProcessFeed<'info> {
    #[account(mut, seeds = [SEED_FEED, feed.authority.as_ref()], bump)]
    pub feed: Account<'info, Feed>,

    /// CHECK: this account is manually being checked against the feed account's feed field
    #[account(
        constraint = pyth_price_feed.key() == feed.pyth_price_feed
    )]
    pub pyth_price_feed: AccountInfo<'info>,

    #[account(
        address = thread.pubkey(),
        constraint = thread.id.eq("feed"),
        signer,
        constraint = thread.authority == feed.key()
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<ProcessFeed>) -> Result<()> {
    let feed = &mut ctx.accounts.feed;
    let pyth_data_feed = &ctx.accounts.pyth_price_feed;

    // load price feed
    let price_feed = load_price_feed_from_account_info(&pyth_data_feed.to_account_info()).unwrap();

    // update last publish time
    feed.publish_time = price_feed.publish_time;

    match price_feed.get_current_price() {
        Some(price) => {
            msg!(
                "Price change for {}: {}",
                price_feed.product_id.to_string(),
                price.price,
            );
        }
        None => {}
    }

    Ok(())
}

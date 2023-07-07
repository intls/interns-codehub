use crate::error::CompressedNftVoterError;
use anchor_lang::prelude::*;
use mpl_bubblegum::hash_metadata;
use mpl_bubblegum::state::leaf_schema::LeafSchema;
use mpl_bubblegum::state::metaplex_adapter::MetadataArgs;
use spl_account_compression::cpi::accounts::VerifyLeaf;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompressedNftAsset {
    pub root: [u8; 32],
    pub asset_id: Pubkey,
    pub metadata: MetadataArgs,
    pub data_hash: [u8; 32],
    pub creator_hash: [u8; 32],
    pub nonce: u64,
    pub index: u32,
}

pub fn verify_cnft<'info>(
    merkle_tree: &AccountInfo<'info>,
    leaf_owner: &AccountInfo<'info>,
    leaf_delegate: &AccountInfo<'info>,
    asset_id: &Pubkey,
    params: &CompressedNftAsset,
    proofs: Vec<AccountInfo<'info>>,
    compression_program: &AccountInfo<'info>,
) -> Result<()> {
    // require!(
    //     leaf_owner.is_signer || leaf_delegate.is_signer,
    //     BubblegumError::LeafAuthorityMustSign
    // );
    require_eq!(
        *asset_id,
        params.asset_id,
        CompressedNftVoterError::InvalidAssetId
    );

    let metadata = params.metadata.clone();
    let data_hash = hash_metadata(&metadata).unwrap();
    require!(
        data_hash == params.data_hash,
        CompressedNftVoterError::TokenMetadataDoesNotMatch
    );

    let leaf = LeafSchema::new_v0(
        *asset_id,
        leaf_owner.key(),
        leaf_delegate.key(),
        params.nonce,
        params.data_hash,
        params.creator_hash,
    );

    let cpi_ctx = CpiContext::new(
        compression_program.clone(),
        VerifyLeaf {
            merkle_tree: merkle_tree.clone(),
        },
    )
    .with_remaining_accounts(proofs);

    spl_account_compression::cpi::verify_leaf(cpi_ctx, params.root, leaf.to_node(), params.index)?;

    Ok(())
}

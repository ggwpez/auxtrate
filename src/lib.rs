use frame_support::traits::{
	GetStorageVersion, OnRuntimeUpgrade, PalletsInfoAccess, StorageVersion,
};
use sp_weights::Weight;

/// Check that the on-chain version matches the code version for all pallets.
///
/// Put this as last into your Migrations with `AllPalletsWithSystem` as `T`.
/// This can be used to check that no migration was forgotten.
pub struct CheckPalletVersions<T>(core::marker::PhantomData<T>);

impl<T: PalletsInfoAccess + GetStorageVersions> OnRuntimeUpgrade for CheckPalletVersions<T> {
	#[cfg(feature = "try-runtime")]
	fn on_runtime_upgrade() -> Weight {
		Weight::zero() // Check done by post_upgrade.
	}

	#[cfg(not(feature = "try-runtime"))]
	fn on_runtime_upgrade() -> Weight {
		if let Err(err) = Self::check() {
			// Just log it. If you see this in production, then it is too late anyway.
			log::error!(
				target: "CheckPalletVersions",
				"{:?}",
				err
			);
			// TODO weight
		};
		Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		Self::check()
	}
}

impl<T: PalletsInfoAccess + GetStorageVersions> CheckPalletVersions<T> {
	fn check() -> Result<(), &'static str> {
		let pallets = <T as PalletsInfoAccess>::infos();
		let onchain = <T as GetStorageVersions>::on_chain_storage_versions();
		let storage = <T as GetStorageVersions>::current_storage_versions();
		let versions = onchain.iter().zip(storage.iter());

		let mut failed = false;
		for (pallet, (onchain, storage)) in pallets.iter().zip(versions) {
			if onchain != storage {
				log::error!(
					target: "CheckPalletVersions",
					"Chain and code version mismatch for pallet {}: {:?} != {:?}",
					pallet.name,
					onchain,
					storage,
				);
				failed = true;
			}
		}
		if failed {
			Err("One or more pallets have incorrect versions, see log.")
		} else {
			log::info!(
				target: "CheckPalletVersions",
				"All pallets have the correct chain version."
			);
			Ok(())
		}
	}
}

/// Tuple version of [`GetStorageVersion`].
pub trait GetStorageVersions {
	/// Returns the current storage version as supported by the pallet.
	fn current_storage_versions() -> Vec<StorageVersion>;
	/// Returns the on-chain storage version of the pallet as stored in the storage.
	fn on_chain_storage_versions() -> Vec<StorageVersion>;
}

// Recursion anchor for the tuple implementation.
impl<T: GetStorageVersion> GetStorageVersions for T {
	fn current_storage_versions() -> Vec<StorageVersion> {
		vec![T::current_storage_version()]
	}

	fn on_chain_storage_versions() -> Vec<StorageVersion> {
		vec![T::on_chain_storage_version()]
	}
}

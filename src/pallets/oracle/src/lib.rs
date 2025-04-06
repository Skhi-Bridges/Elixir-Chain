#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Get},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use sp_runtime::traits::{Zero, StaticLookup};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: ReservableCurrency<Self::AccountId>;
        #[pallet::constant]
        type OracleDeposit: Get<BalanceOf<Self>>;
        type MaxDataLength: Get<u32>;
        type MaxValidatorCount: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn price_feeds)]
    pub type PriceFeeds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>, // Asset identifier
        PriceData<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorInfo<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorRegistered(T::AccountId),
        ValidatorRemoved(T::AccountId),
        PriceUpdated(Vec<u8>, BalanceOf<T>, T::BlockNumber),
        PriceAggregated(Vec<u8>, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotFound,
        ValidatorLimitReached,
        DataTooLong,
        InsufficientBalance,
        NotAuthorized,
        InvalidPrice,
        AssetNotFound,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct PriceData<T: Config> {
        pub price: BalanceOf<T>,
        pub last_updated: T::BlockNumber,
        pub update_count: u32,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorInfo<T: Config> {
        pub account: T::AccountId,
        pub registered_at: T::BlockNumber,
        pub update_count: u32,
        pub last_update: T::BlockNumber,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_validator(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyRegistered);
            
            let count = ValidatorCount::<T>::get();
            ensure!(count < T::MaxValidatorCount::get(), Error::<T>::ValidatorLimitReached);
            
            let deposit = T::OracleDeposit::get();
            T::Currency::reserve(&who, deposit)?;
            
            let validator_info = ValidatorInfo {
                account: who.clone(),
                registered_at: <frame_system::Pallet<T>>::block_number(),
                update_count: 0,
                last_update: <frame_system::Pallet<T>>::block_number(),
            };
            
            Validators::<T>::insert(&who, validator_info);
            ValidatorCount::<T>::put(count + 1);
            
            Self::deposit_event(Event::ValidatorRegistered(who));
            Ok(())
        }
        
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn remove_validator(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Validators::<T>::contains_key(&who), Error::<T>::ValidatorNotFound);
            
            let deposit = T::OracleDeposit::get();
            T::Currency::unreserve(&who, deposit);
            
            Validators::<T>::remove(&who);
            
            let count = ValidatorCount::<T>::get();
            ValidatorCount::<T>::put(count - 1);
            
            Self::deposit_event(Event::ValidatorRemoved(who));
            Ok(())
        }
        
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn update_price(
            origin: OriginFor<T>,
            asset_id: Vec<u8>,
            price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Validators::<T>::contains_key(&who), Error::<T>::NotAuthorized);
            ensure!(asset_id.len() <= T::MaxDataLength::get() as usize, Error::<T>::DataTooLong);
            ensure!(!price.is_zero(), Error::<T>::InvalidPrice);
            
            let current_block = <frame_system::Pallet<T>>::block_number();
            
            // Update validator info
            Validators::<T>::mutate(&who, |validator| {
                if let Some(v) = validator {
                    v.update_count += 1;
                    v.last_update = current_block;
                }
            });
            
            // Update or create price feed
            if PriceFeeds::<T>::contains_key(&asset_id) {
                PriceFeeds::<T>::mutate(&asset_id, |price_data| {
                    if let Some(pd) = price_data {
                        pd.price = price;
                        pd.last_updated = current_block;
                        pd.update_count += 1;
                    }
                });
            } else {
                let price_data = PriceData {
                    price,
                    last_updated: current_block,
                    update_count: 1,
                };
                PriceFeeds::<T>::insert(&asset_id, price_data);
            }
            
            Self::deposit_event(Event::PriceUpdated(asset_id, price, current_block));
            Ok(())
        }
        
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn aggregate_prices(
            origin: OriginFor<T>,
            asset_id: Vec<u8>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            
            ensure!(PriceFeeds::<T>::contains_key(&asset_id), Error::<T>::AssetNotFound);
            
            // In a real implementation, this would aggregate prices from multiple validators
            // For simplicity, we're just using the latest price
            let price_data = PriceFeeds::<T>::get(&asset_id).ok_or(Error::<T>::AssetNotFound)?;
            
            Self::deposit_event(Event::PriceAggregated(asset_id, price_data.price));
            Ok(())
        }
    }

    // Implement error correction mechanisms as per project requirements
    impl<T: Config> Pallet<T> {
        // Classical error correction
        pub fn verify_and_correct_data(data: &mut Vec<u8>) -> Result<(), Error<T>> {
            // Reed-Solomon error correction implementation
            // This is a placeholder for the actual implementation
            Ok(())
        }

        // Bridge error correction for classical-quantum interface
        pub fn bridge_error_correction(data: &mut Vec<u8>) -> Result<(), Error<T>> {
            // Implement redundancy and verification protocols
            // This is a placeholder for the actual implementation
            Ok(())
        }

        // Quantum error correction
        pub fn quantum_error_correction(data: &mut Vec<u8>) -> Result<(), Error<T>> {
            // Surface code implementation for quantum error correction
            // This is a placeholder for the actual implementation
            Ok(())
        }
    }
}

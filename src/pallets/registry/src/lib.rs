#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, ExistenceRequirement},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use sp_runtime::traits::{StaticLookup, Zero};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: ReservableCurrency<Self::AccountId>;
        #[pallet::constant]
        type RegistrationDeposit: Get<BalanceOf<Self>>;
        type MaxFacilityNameLength: Get<u32>;
        type MaxLocationLength: Get<u32>;
        type MaxCertificationLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn facilities)]
    pub type Facilities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        FacilityInfo<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn facility_count)]
    pub type FacilityCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn batches)]
    pub type Batches<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        BatchInfo<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn batch_count)]
    pub type BatchCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FacilityRegistered(T::AccountId, Vec<u8>),
        FacilityUpdated(T::AccountId, Vec<u8>),
        BatchRegistered(T::AccountId, T::Hash, Vec<u8>),
        BatchCertified(T::Hash, Vec<u8>),
        BatchShipped(T::Hash, T::AccountId),
        BatchReceived(T::Hash, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        FacilityAlreadyRegistered,
        FacilityNotFound,
        BatchAlreadyRegistered,
        BatchNotFound,
        NotFacilityOwner,
        NameTooLong,
        LocationTooLong,
        CertificationTooLong,
        InsufficientBalance,
        NotAuthorized,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct FacilityInfo<T: Config> {
        pub owner: T::AccountId,
        pub name: Vec<u8>,
        pub location: Vec<u8>,
        pub certification: Vec<u8>,
        pub registered_at: T::BlockNumber,
        pub batch_count: u32,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BatchInfo<T: Config> {
        pub facility: T::AccountId,
        pub batch_id: Vec<u8>,
        pub production_date: T::BlockNumber,
        pub certification: Vec<u8>,
        pub current_owner: T::AccountId,
        pub status: BatchStatus,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum BatchStatus {
        Produced,
        Certified,
        InTransit,
        Delivered,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_facility(
            origin: OriginFor<T>,
            name: Vec<u8>,
            location: Vec<u8>,
            certification: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(!Facilities::<T>::contains_key(&who), Error::<T>::FacilityAlreadyRegistered);
            ensure!(name.len() <= T::MaxFacilityNameLength::get() as usize, Error::<T>::NameTooLong);
            ensure!(location.len() <= T::MaxLocationLength::get() as usize, Error::<T>::LocationTooLong);
            ensure!(certification.len() <= T::MaxCertificationLength::get() as usize, Error::<T>::CertificationTooLong);
            
            let deposit = T::RegistrationDeposit::get();
            T::Currency::reserve(&who, deposit)?;
            
            let facility_info = FacilityInfo {
                owner: who.clone(),
                name: name.clone(),
                location,
                certification,
                registered_at: <frame_system::Pallet<T>>::block_number(),
                batch_count: 0,
            };
            
            Facilities::<T>::insert(&who, facility_info);
            let count = FacilityCount::<T>::get();
            FacilityCount::<T>::put(count + 1);
            
            Self::deposit_event(Event::FacilityRegistered(who, name));
            Ok(())
        }
        
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn update_facility(
            origin: OriginFor<T>,
            name: Vec<u8>,
            location: Vec<u8>,
            certification: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Facilities::<T>::contains_key(&who), Error::<T>::FacilityNotFound);
            ensure!(name.len() <= T::MaxFacilityNameLength::get() as usize, Error::<T>::NameTooLong);
            ensure!(location.len() <= T::MaxLocationLength::get() as usize, Error::<T>::LocationTooLong);
            ensure!(certification.len() <= T::MaxCertificationLength::get() as usize, Error::<T>::CertificationTooLong);
            
            Facilities::<T>::mutate(&who, |facility| {
                if let Some(f) = facility {
                    f.name = name.clone();
                    f.location = location;
                    f.certification = certification;
                }
            });
            
            Self::deposit_event(Event::FacilityUpdated(who, name));
            Ok(())
        }
        
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn register_batch(
            origin: OriginFor<T>,
            batch_id: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Facilities::<T>::contains_key(&who), Error::<T>::FacilityNotFound);
            
            let batch_hash = T::Hashing::hash_of(&batch_id);
            ensure!(!Batches::<T>::contains_key(batch_hash), Error::<T>::BatchAlreadyRegistered);
            
            let batch_info = BatchInfo {
                facility: who.clone(),
                batch_id: batch_id.clone(),
                production_date: <frame_system::Pallet<T>>::block_number(),
                certification: Vec::new(),
                current_owner: who.clone(),
                status: BatchStatus::Produced,
            };
            
            Batches::<T>::insert(batch_hash, batch_info);
            
            Facilities::<T>::mutate(&who, |facility| {
                if let Some(f) = facility {
                    f.batch_count += 1;
                }
            });
            
            let count = BatchCount::<T>::get();
            BatchCount::<T>::put(count + 1);
            
            Self::deposit_event(Event::BatchRegistered(who, batch_hash, batch_id));
            Ok(())
        }
        
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn certify_batch(
            origin: OriginFor<T>,
            batch_hash: T::Hash,
            certification: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Batches::<T>::contains_key(batch_hash), Error::<T>::BatchNotFound);
            ensure!(certification.len() <= T::MaxCertificationLength::get() as usize, Error::<T>::CertificationTooLong);
            
            let batch = Batches::<T>::get(batch_hash).ok_or(Error::<T>::BatchNotFound)?;
            ensure!(batch.facility == who, Error::<T>::NotAuthorized);
            
            Batches::<T>::mutate(batch_hash, |b| {
                if let Some(batch) = b {
                    batch.certification = certification.clone();
                    batch.status = BatchStatus::Certified;
                }
            });
            
            Self::deposit_event(Event::BatchCertified(batch_hash, certification));
            Ok(())
        }
        
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn ship_batch(
            origin: OriginFor<T>,
            batch_hash: T::Hash,
            destination: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(destination)?;
            
            ensure!(Batches::<T>::contains_key(batch_hash), Error::<T>::BatchNotFound);
            
            let batch = Batches::<T>::get(batch_hash).ok_or(Error::<T>::BatchNotFound)?;
            ensure!(batch.current_owner == who, Error::<T>::NotAuthorized);
            
            Batches::<T>::mutate(batch_hash, |b| {
                if let Some(batch) = b {
                    batch.status = BatchStatus::InTransit;
                }
            });
            
            Self::deposit_event(Event::BatchShipped(batch_hash, to.clone()));
            Ok(())
        }
        
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn receive_batch(
            origin: OriginFor<T>,
            batch_hash: T::Hash>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Batches::<T>::contains_key(batch_hash), Error::<T>::BatchNotFound);
            
            Batches::<T>::mutate(batch_hash, |b| {
                if let Some(batch) = b {
                    batch.current_owner = who.clone();
                    batch.status = BatchStatus::Delivered;
                }
            });
            
            Self::deposit_event(Event::BatchReceived(batch_hash, who));
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

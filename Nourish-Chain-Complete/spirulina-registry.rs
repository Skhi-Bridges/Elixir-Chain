#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod spirulina_registry {
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    /// Represents a registered spirulina cultivation facility
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CultivationFacility {
        /// Unique ID for the facility
        id: String,
        /// Public name of the facility
        name: String,
        /// Geographic coordinates
        location: (i32, i32),
        /// Cultivation capacity in square meters
        capacity: u32,
        /// ISO certification details
        certifications: Vec<Certification>,
        /// Cultivation methods used
        methods: Vec<CultivationMethod>,
        /// Status of the facility
        status: FacilityStatus,
        /// Owner account
        owner: AccountId,
        /// Timestamp of registration
        registered_at: Timestamp,
        /// Latest audit timestamp
        last_audit: Timestamp,
    }

    /// Certification information
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Certification {
        /// Type of certification
        cert_type: CertificationType,
        /// Identification number
        cert_id: String,
        /// Issuing authority
        issuer: String,
        /// Expiration timestamp
        valid_until: Timestamp,
    }

    /// Types of certifications
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CertificationType {
        Organic,
        GMP,
        HACCP,
        ISO22000,
        FairTrade,
        Kosher,
        Halal,
        Other,
    }

    /// Cultivation methods
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CultivationMethod {
        OpenPond,
        Raceway,
        Photobioreactor,
        Greenhouse,
        SemiClosed,
        FullyClosed,
        Hybrid,
    }

    /// Status of facility registration
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FacilityStatus {
        Pending,
        Active,
        Suspended,
        Revoked,
    }

    /// Represents an authorized telemetry device
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct TelemetryDevice {
        /// Unique device identifier
        device_id: String,
        /// Facility ID associated with the device
        facility_id: String,
        /// Public key for quantum-resistant authentication
        public_key: Vec<u8>,
        /// Status of the device
        status: DeviceStatus,
        /// Registration timestamp
        registered_at: Timestamp,
        /// Latest activity timestamp
        last_active: Timestamp,
        /// Device firmware version
        firmware_version: String,
    }

    /// Status of a telemetry device
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum DeviceStatus {
        Authorized,
        Suspended,
        Revoked,
    }

    /// Cultivation parameters for a facility
    #[derive(Debug, Encode, Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CultivationParameters {
        /// Optimal pH range
        ph_range: (u32, u32),
        /// Optimal temperature range in Celsius (scaled by 100)
        temp_range: (u32, u32),
        /// Optimal light range in lux (scaled by 10)
        light_range: (u32, u32),
        /// Optimal density range in g/L (scaled by 1000)
        density_range: (u32, u32),
        /// Optimal dissolved oxygen range in mg/L (scaled by 100)
        dissolved_oxygen_range: (u32, u32),
        /// Optimal nitrate range in mg/L (scaled by 10)
        nitrate_range: (u32, u32),
        /// Optimal salinity range in g/L (scaled by 10)
        salinity_range: (u32, u32),
    }

    /// Simple timestamp type (Unix timestamp)
    pub type Timestamp = u64;

    #[ink(storage)]
    pub struct SpirulinaRegistry {
        /// Contract owner
        owner: AccountId,
        /// Map of registered cultivation facilities
        facilities: StorageHashMap<String, CultivationFacility>,
        /// Map of authorized telemetry devices
        devices: StorageHashMap<String, TelemetryDevice>,
        /// Map of cultivation parameters by facility ID
        parameters: StorageHashMap<String, CultivationParameters>,
        /// Map of authorized auditors
        auditors: StorageHashMap<AccountId, bool>,
        /// Map of facility IDs by owner
        facilities_by_owner: StorageHashMap<AccountId, Vec<String>>,
        /// Default parameters for new facilities
        default_parameters: CultivationParameters,
        /// Total number of registered facilities
        facilities_count: u32,
        /// Total number of authorized devices
        devices_count: u32,
    }

    /// Errors that can occur in the registry
    #[derive(Debug, Encode, Decode, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller is not authorized
        Unauthorized,
        /// Facility ID already exists
        FacilityAlreadyExists,
        /// Facility ID does not exist
        FacilityNotFound,
        /// Device ID already exists
        DeviceAlreadyExists,
        /// Device ID does not exist
        DeviceNotFound,
        /// Invalid parameters
        InvalidParameters,
        /// Facility is not active
        FacilityNotActive,
        /// Device is not authorized
        DeviceNotAuthorized,
        /// Certification has expired
        CertificationExpired,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct FacilityRegistered {
        #[ink(topic)]
        facility_id: String,
        owner: AccountId,
    }

    #[ink(event)]
    pub struct FacilityStatusChanged {
        #[ink(topic)]
        facility_id: String,
        new_status: FacilityStatus,
    }

    #[ink(event)]
    pub struct DeviceAuthorized {
        #[ink(topic)]
        device_id: String,
        #[ink(topic)]
        facility_id: String,
    }

    #[ink(event)]
    pub struct DeviceStatusChanged {
        #[ink(topic)]
        device_id: String,
        new_status: DeviceStatus,
    }

    #[ink(event)]
    pub struct ParametersUpdated {
        #[ink(topic)]
        facility_id: String,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl SpirulinaRegistry {
        /// Creates a new registry with the caller as owner
        #[ink(constructor)]
        pub fn new() -> Self {
            let default_parameters = CultivationParameters {
                ph_range: (850, 1050),          // 8.5 - 10.5
                temp_range: (3000, 3700),       // 30.0°C - 37.0°C
                light_range: (2500, 10000),     // 2500 - 10000 lux
                density_range: (1000, 3000),    // 1.0 - 3.0 g/L
                dissolved_oxygen_range: (600, 900), // 6.0 - 9.0 mg/L
                nitrate_range: (100, 300),      // 10.0 - 30.0 mg/L
                salinity_range: (100, 200),     // 10.0 - 20.0 g/L
            };

            Self {
                owner: Self::env().caller(),
                facilities: StorageHashMap::new(),
                devices: StorageHashMap::new(),
                parameters: StorageHashMap::new(),
                auditors: StorageHashMap::new(),
                facilities_by_owner: StorageHashMap::new(),
                default_parameters,
                facilities_count: 0,
                devices_count: 0,
            }
        }

        /// Registers a new cultivation facility
        #[ink(message)]
        pub fn register_facility(
            &mut self,
            id: String,
            name: String,
            location: (i32, i32),
            capacity: u32,
            methods: Vec<CultivationMethod>,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Check if facility ID already exists
            if self.facilities.contains_key(&id) {
                return Err(Error::FacilityAlreadyExists);
            }

            // Create new facility with pending status
            let facility = CultivationFacility {
                id: id.clone(),
                name,
                location,
                capacity,
                certifications: Vec::new(),
                methods,
                status: FacilityStatus::Pending,
                owner: caller,
                registered_at: self.env().block_timestamp(),
                last_audit: 0, // No audit yet
            };

            // Add facility to storage
            self.facilities.insert(id.clone(), facility);
            
            // Add to owner's facilities
            let mut owner_facilities = self.facilities_by_owner.get(&caller).unwrap_or(&Vec::new()).clone();
            owner_facilities.push(id.clone());
            self.facilities_by_owner.insert(caller, owner_facilities);

            // Set default parameters
            self.parameters.insert(id.clone(), self.default_parameters.clone());

            // Increment counter
            self.facilities_count += 1;

            // Emit event
            self.env().emit_event(FacilityRegistered {
                facility_id: id,
                owner: caller,
            });

            Ok(())
        }

        /// Updates the status of a facility
        #[ink(message)]
        pub fn update_facility_status(
            &mut self,
            facility_id: String,
            new_status: FacilityStatus,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Only owner or auditor can update status
            if caller != self.owner && !self.is_auditor(caller) {
                return Err(Error::Unauthorized);
            }

            // Check if facility exists
            let mut facility = match self.facilities.get(&facility_id) {
                Some(f) => f.clone(),
                None => return Err(Error::FacilityNotFound),
            };

            // Update status
            facility.status = new_status.clone();
            self.facilities.insert(facility_id.clone(), facility);

            // Emit event
            self.env().emit_event(FacilityStatusChanged {
                facility_id,
                new_status,
            });

            Ok(())
        }

        /// Registers a new telemetry device for a facility
        #[ink(message)]
        pub fn register_device(
            &mut self,
            device_id: String,
            facility_id: String,
            public_key: Vec<u8>,
            firmware_version: String,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Check if facility exists and is owned by caller
            let facility = match self.facilities.get(&facility_id) {
                Some(f) => f.clone(),
                None => return Err(Error::FacilityNotFound),
            };

            // Only facility owner can register devices
            if facility.owner != caller {
                return Err(Error::Unauthorized);
            }

            // Facility must be active
            if facility.status != FacilityStatus::Active {
                return Err(Error::FacilityNotActive);
            }

            // Check if device ID already exists
            if self.devices.contains_key(&device_id) {
                return Err(Error::DeviceAlreadyExists);
            }

            // Create new device
            let device = TelemetryDevice {
                device_id: device_id.clone(),
                facility_id: facility_id.clone(),
                public_key,
                status: DeviceStatus::Authorized,
                registered_at: self.env().block_timestamp(),
                last_active: self.env().block_timestamp(),
                firmware_version,
            };

            // Add device to storage
            self.devices.insert(device_id.clone(), device);
            self.devices_count += 1;

            // Emit event
            self.env().emit_event(DeviceAuthorized {
                device_id,
                facility_id,
            });

            Ok(())
        }

        /// Updates device status
        #[ink(message)]
        pub fn update_device_status(
            &mut self,
            device_id: String,
            new_status: DeviceStatus,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Check if device exists
            let mut device = match self.devices.get(&device_id) {
                Some(d) => d.clone(),
                None => return Err(Error::DeviceNotFound),
            };

            // Check if facility exists
            let facility = match self.facilities.get(&device.facility_id) {
                Some(f) => f.clone(),
                None => return Err(Error::FacilityNotFound),
            };

            // Only facility owner or contract owner can update device status
            if facility.owner != caller && caller != self.owner {
                return Err(Error::Unauthorized);
            }

            // Update status
            device.status = new_status.clone();
            self.devices.insert(device_id.clone(), device);

            // Emit event
            self.env().emit_event(DeviceStatusChanged {
                device_id,
                new_status,
            });

            Ok(())
        }

        /// Updates cultivation parameters for a facility
        #[ink(message)]
        pub fn update_parameters(
            &mut self,
            facility_id: String,
            parameters: CultivationParameters,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Check if facility exists
            let facility = match self.facilities.get(&facility_id) {
                Some(f) => f.clone(),
                None => return Err(Error::FacilityNotFound),
            };

            // Only facility owner or auditor can update parameters
            if facility.owner != caller && !self.is_auditor(caller) {
                return Err(Error::Unauthorized);
            }

            // Validate parameters
            if !self.is_valid_parameters(&parameters) {
                return Err(Error::InvalidParameters);
            }

            // Update parameters
            self.parameters.insert(facility_id.clone(), parameters);

            // Emit event
            self.env().emit_event(ParametersUpdated {
                facility_id,
            });

            Ok(())
        }

        /// Adds a certification to a facility
        #[ink(message)]
        pub fn add_certification(
            &mut self,
            facility_id: String,
            cert_type: CertificationType,
            cert_id: String,
            issuer: String,
            valid_until: Timestamp,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Only auditors can add certifications
            if !self.is_auditor(caller) {
                return Err(Error::Unauthorized);
            }

            // Check if facility exists
            let mut facility = match self.facilities.get(&facility_id) {
                Some(f) => f.clone(),
                None => return Err(Error::FacilityNotFound),
            };

            // Create certification
            let certification = Certification {
                cert_type,
                cert_id,
                issuer,
                valid_until,
            };

            // Add certification to facility
            facility.certifications.push(certification);
            self.facilities.insert(facility_id, facility);

            Ok(())
        }

        /// Performs an audit on a facility
        #[ink(message)]
        pub fn perform_audit(
            &mut self,
            facility_id: String,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Only auditors can perform audits
            if !self.is_auditor(caller) {
                return Err(Error::Unauthorized);
            }

            // Check if facility exists
            let mut facility = match self.facilities.get(&facility_id) {
                Some(f) => f.clone(),
                None => return Err(Error::FacilityNotFound),
            };

            // Update last audit timestamp
            facility.last_audit = self.env().block_timestamp();
            self.facilities.insert(facility_id, facility);

            Ok(())
        }

        /// Adds an auditor
        #[ink(message)]
        pub fn add_auditor(
            &mut self,
            auditor: AccountId,
        ) -> Result<()> {
            // Only owner can add auditors
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            // Add auditor
            self.auditors.insert(auditor, true);

            Ok(())
        }

        /// Removes an auditor
        #[ink(message)]
        pub fn remove_auditor(
            &mut self,
            auditor: AccountId,
        ) -> Result<()> {
            // Only owner can remove auditors
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            // Remove auditor
            self.auditors.insert(auditor, false);

            Ok(())
        }

        /// Updates the default parameters for new facilities
        #[ink(message)]
        pub fn update_default_parameters(
            &mut self,
            parameters: CultivationParameters,
        ) -> Result<()> {
            // Only owner can update default parameters
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            // Validate parameters
            if !self.is_valid_parameters(&parameters) {
                return Err(Error::InvalidParameters);
            }

            // Update default parameters
            self.default_parameters = parameters;

            Ok(())
        }

        /// Checks if an account is an authorized auditor
        #[ink(message)]
        pub fn is_auditor(&self, account: AccountId) -> bool {
            self.auditors.get(&account).copied().unwrap_or(false)
        }

        /// Gets a facility by ID
        #[ink(message)]
        pub fn get_facility(&self, facility_id: String) -> Option<CultivationFacility> {
            self.facilities.get(&facility_id).cloned()
        }

        /// Gets a device by ID
        #[ink(message)]
        pub fn get_device(&self, device_id: String) -> Option<TelemetryDevice> {
            self.devices.get(&device_id).cloned()
        }

        /// Gets cultivation parameters for a facility
        #[ink(message)]
        pub fn get_parameters(&self, facility_id: String) -> Option<CultivationParameters> {
            self.parameters.get(&facility_id).cloned()
        }

        /// Gets the default parameters
        #[ink(message)]
        pub fn get_default_parameters(&self) -> CultivationParameters {
            self.default_parameters.clone()
        }

        /// Gets facilities owned by an account
        #[ink(message)]
        pub fn get_facilities_by_owner(&self, owner: AccountId) -> Vec<String> {
            self.facilities_by_owner.get(&owner).cloned().unwrap_or_default()
        }

        /// Gets the total number of registered facilities
        #[ink(message)]
        pub fn get_facilities_count(&self) -> u32 {
            self.facilities_count
        }

        /// Gets the total number of authorized devices
        #[ink(message)]
        pub fn get_devices_count(&self) -> u32 {
            self.devices_count
        }

        /// Updates the activity timestamp for a device
        #[ink(message)]
        pub fn update_device_activity(&mut self, device_id: String) -> Result<()> {
            // Check if device exists
            let mut device = match self.devices.get(&device_id) {
                Some(d) => d.clone(),
                None => return Err(Error::DeviceNotFound),
            };

            // Device must be authorized
            if device.status != DeviceStatus::Authorized {
                return Err(Error::DeviceNotAuthorized);
            }

            // Update last active timestamp
            device.last_active = self.env().block_timestamp();
            self.devices.insert(device_id, device);

            Ok(())
        }

        /// Validates if a device is authorized for a specific facility
        #[ink(message)]
        pub fn is_device_authorized(&self, device_id: String, facility_id: String) -> bool {
            match self.devices.get(&device_id) {
                Some(device) => {
                    device.facility_id == facility_id
                        && device.status == DeviceStatus::Authorized
                },
                None => false,
            }
        }

        /// Validates that parameters are within reasonable bounds
        fn is_valid_parameters(&self, parameters: &CultivationParameters) -> bool {
            // Check that minimum values are less than maximum values
            if parameters.ph_range.0 >= parameters.ph_range.1
                || parameters.temp_range.0 >= parameters.temp_range.1
                || parameters.light_range.0 >= parameters.light_range.1
                || parameters.density_range.0 >= parameters.density_range.1
                || parameters.dissolved_oxygen_range.0 >= parameters.dissolved_oxygen_range.1
                || parameters.nitrate_range.0 >= parameters.nitrate_range.1
                || parameters.salinity_range.0 >= parameters.salinity_range.1
            {
                return false;
            }

            // Check that values are within reasonable bounds
            if parameters.ph_range.0 < 500 || parameters.ph_range.1 > 1400 // pH 5.0 - 14.0
                || parameters.temp_range.0 < 1500 || parameters.temp_range.1 > 4500 // 15°C - 45°C
                || parameters.light_range.0 < 0 || parameters.light_range.1 > 50000 // 0 - 50000 lux
                || parameters.density_range.0 < 500 || parameters.density_range.1 > 10000 // 0.5 - 10 g/L
                || parameters.dissolved_oxygen_range.0 < 0 || parameters.dissolved_oxygen_range.1 > 2000 // 0 - 20 mg/L
                || parameters.nitrate_range.0 < 0 || parameters.nitrate_range.1 > 1000 // 0 - 100 mg/L
                || parameters.salinity_range.0 < 0 || parameters.salinity_range.1 > 500 // 0 - 50 g/L
            {
                return false;
            }

            true
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn registry_works() {
            let mut registry = SpirulinaRegistry::new();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            
            // Register a facility
            let result = registry.register_facility(
                String::from("FAC001"),
                String::from("Test Facility"),
                (100, 200),
                1000,
                vec![CultivationMethod::OpenPond],
            );
            assert!(result.is_ok());
            
            // Check facility is registered
            let facility = registry.get_facility(String::from("FAC001")).unwrap();
            assert_eq!(facility.name, String::from("Test Facility"));
            assert_eq!(facility.status, FacilityStatus::Pending);
            
            // Add an auditor
            let result = registry.add_auditor(accounts.bob);
            assert!(result.is_ok());
            
            // Check auditor was added
            assert!(registry.is_auditor(accounts.bob));
            
            // Facility count should be 1
            assert_eq!(registry.get_facilities_count(), 1);
        }
    }
}

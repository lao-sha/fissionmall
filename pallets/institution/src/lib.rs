#![cfg_attr(not(feature = "std"), no_std)]

/// 机构管理模块
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config + scale_info::TypeInfo {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// 机构ID最大长度
        #[pallet::constant]
        type MaxInstitutionIdLength: Get<u32>;
        
        /// 名称最大长度
        #[pallet::constant]
        type MaxNameLength: Get<u32>;
        
        /// 负责人信息最大长度
        #[pallet::constant]
        type MaxResponsiblePersonLength: Get<u32>;
        
        /// 经营范围最大长度
        #[pallet::constant]
        type MaxBusinessScopeLength: Get<u32>;
        
        /// 合约最大长度
        #[pallet::constant]
        type MaxContractLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 机构状态枚举
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum InstitutionStatus {
        Certified = 0,       // 已认证
        NotCertified = 1,    // 未认证
        Deactivated = 2,     // 注销
    }

    /// 机构信息结构
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct InstitutionInfo<T: Config> {
        pub institution_name: BoundedVec<u8, T::MaxNameLength>, // 机构名称
        pub status: InstitutionStatus,                         // 机构状态
        pub institution_full_name: BoundedVec<u8, T::MaxNameLength>, // 机构全名
        pub license_image_url: BoundedVec<u8, T::MaxNameLength>, // 营业执照图片URL
        pub responsible_person: BoundedVec<u8, T::MaxResponsiblePersonLength>, // 负责人
        pub business_scope: BoundedVec<u8, T::MaxBusinessScopeLength>, // 经营范围
        pub profit_contract: Option<BoundedVec<u8, T::MaxContractLength>>, // 分润合约
        pub created_date: frame_system::pallet_prelude::BlockNumberFor<T>, // 创建日期
        pub creator: T::AccountId,                             // 创建者
    }

    /// 机构存储映射
    #[pallet::storage]
    #[pallet::storage_prefix = "Institutions"]
    pub type Institutions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionIdLength>, // 主键：机构 ID
        InstitutionInfo<T>,                        // 值：机构信息
        OptionQuery,                               // 查询策略：如果键不存在，返回 None
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 机构已创建 [机构ID, 创建者]
        InstitutionCreated(BoundedVec<u8, T::MaxInstitutionIdLength>, T::AccountId),
        /// 机构状态已更新 [机构ID, 新状态(u8)]
        InstitutionStatusUpdated(BoundedVec<u8, T::MaxInstitutionIdLength>, u8),
        /// 机构信息已更新 [机构ID]
        InstitutionInfoUpdated(BoundedVec<u8, T::MaxInstitutionIdLength>),
        /// 机构已删除 [机构ID]
        InstitutionDeleted(BoundedVec<u8, T::MaxInstitutionIdLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 机构ID已存在
        InstitutionIdAlreadyExists,
        /// 机构不存在
        InstitutionNotFound,
        /// 无权操作此机构
        NotAuthorized,
        /// 字符串转换错误
        StringConversionError,
        /// 无效状态
        InvalidStatus,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新的机构
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_institution(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
            institution_name: Vec<u8>,
            institution_full_name: Vec<u8>,
            license_image_url: Vec<u8>,
            responsible_person: Vec<u8>,
            business_scope: Vec<u8>,
            profit_contract: Option<Vec<u8>>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查机构ID是否已存在
            ensure!(!Institutions::<T>::contains_key(&bounded_id), Error::<T>::InstitutionIdAlreadyExists);
            
            // 转换其他字段为边界向量
            let bounded_name = BoundedVec::<u8, T::MaxNameLength>::try_from(institution_name)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_full_name = BoundedVec::<u8, T::MaxNameLength>::try_from(institution_full_name)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_license_url = BoundedVec::<u8, T::MaxNameLength>::try_from(license_image_url)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_responsible = BoundedVec::<u8, T::MaxResponsiblePersonLength>::try_from(responsible_person)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_scope = BoundedVec::<u8, T::MaxBusinessScopeLength>::try_from(business_scope)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            // 处理可选的分润合约
            let bounded_contract = match profit_contract {
                Some(contract) => Some(BoundedVec::<u8, T::MaxContractLength>::try_from(contract)
                    .map_err(|_| Error::<T>::StringConversionError)?),
                None => None,
            };
            
            // 创建机构信息
            let institution = InstitutionInfo {
                institution_name: bounded_name,
                status: InstitutionStatus::NotCertified, // 默认为未认证
                institution_full_name: bounded_full_name,
                license_image_url: bounded_license_url,
                responsible_person: bounded_responsible,
                business_scope: bounded_scope,
                profit_contract: bounded_contract,
                created_date: frame_system::Pallet::<T>::block_number(),
                creator: who.clone(),
            };
            
            // 存储机构信息
            Institutions::<T>::insert(&bounded_id, institution);
            
            // 发出事件
            Self::deposit_event(Event::InstitutionCreated(bounded_id, who));
            
            Ok(())
        }
        
        /// 更新机构状态
        #[pallet::call_index(1)]
        #[pallet::weight(5_000)]
        pub fn update_institution_status(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
            status: u8,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新机构状态
            Institutions::<T>::try_mutate(&bounded_id, |maybe_institution| -> DispatchResult {
                let institution = maybe_institution.as_mut().ok_or(Error::<T>::InstitutionNotFound)?;
                
                // 检查权限（仅创建者可以更新）
                ensure!(institution.creator == who, Error::<T>::NotAuthorized);
                
                // 将 u8 转换为 InstitutionStatus
                let new_status = match status {
                    0 => InstitutionStatus::Certified,
                    1 => InstitutionStatus::NotCertified,
                    2 => InstitutionStatus::Deactivated,
                    _ => return Err(Error::<T>::InvalidStatus.into()),
                };
                
                // 更新状态
                institution.status = new_status;
                
                // 发出事件
                Self::deposit_event(Event::InstitutionStatusUpdated(bounded_id.clone(), status));
                
                Ok(())
            })
        }
        
        /// 更新机构信息
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn update_institution_info(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
            institution_name: Option<Vec<u8>>,
            institution_full_name: Option<Vec<u8>>,
            license_image_url: Option<Vec<u8>>,
            responsible_person: Option<Vec<u8>>,
            business_scope: Option<Vec<u8>>,
            profit_contract: Option<Vec<u8>>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新机构信息
            Institutions::<T>::try_mutate(&bounded_id, |maybe_institution| -> DispatchResult {
                let institution = maybe_institution.as_mut().ok_or(Error::<T>::InstitutionNotFound)?;
                
                // 检查权限（仅创建者可以更新）
                ensure!(institution.creator == who, Error::<T>::NotAuthorized);
                
                // 更新各字段（如果提供）
                if let Some(name) = institution_name {
                    let bounded_name = BoundedVec::<u8, T::MaxNameLength>::try_from(name)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                    institution.institution_name = bounded_name;
                }
                
                if let Some(full_name) = institution_full_name {
                    let bounded_full_name = BoundedVec::<u8, T::MaxNameLength>::try_from(full_name)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                    institution.institution_full_name = bounded_full_name;
                }
                
                if let Some(license_url) = license_image_url {
                    let bounded_license_url = BoundedVec::<u8, T::MaxNameLength>::try_from(license_url)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                    institution.license_image_url = bounded_license_url;
                }
                
                if let Some(responsible) = responsible_person {
                    let bounded_responsible = BoundedVec::<u8, T::MaxResponsiblePersonLength>::try_from(responsible)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                    institution.responsible_person = bounded_responsible;
                }
                
                if let Some(scope) = business_scope {
                    let bounded_scope = BoundedVec::<u8, T::MaxBusinessScopeLength>::try_from(scope)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                    institution.business_scope = bounded_scope;
                }
                
                if let Some(contract) = profit_contract {
                    let bounded_contract = BoundedVec::<u8, T::MaxContractLength>::try_from(contract)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                    institution.profit_contract = Some(bounded_contract);
                }
                
                // 发出事件
                Self::deposit_event(Event::InstitutionInfoUpdated(bounded_id.clone()));
                
                Ok(())
            })
        }
        
        /// 删除机构
        #[pallet::call_index(3)]
        #[pallet::weight(5_000)]
        pub fn delete_institution(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取机构并检查权限
            let institution = Institutions::<T>::get(&bounded_id)
                .ok_or(Error::<T>::InstitutionNotFound)?;
            
            ensure!(institution.creator == who, Error::<T>::NotAuthorized);
            
            // 删除机构
            Institutions::<T>::remove(&bounded_id);
            
            // 发出事件
            Self::deposit_event(Event::InstitutionDeleted(bounded_id));
            
            Ok(())
        }
    }
} 